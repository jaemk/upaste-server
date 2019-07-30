//! Service
//!  - Initialize external and persistent services/structs
//!  - Initialize loggers
//!  - Mount url endpoints to `handlers` functions
//!  - Mount static file handler
//!
use env_logger;
use std::env;
use std::io::Write;
use std::path::Path;
use std::sync;
use std::thread;
use std::time;

use chrono::{self, Local};
use migrant_lib::{Config, Settings};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rouille;
use rusqlite::Connection;
use tera::Tera;

use crate::errors::*;
use crate::handlers;
use crate::models;
use crate::{ToResponse, MAX_PASTE_AGE_SECONDS};

// convenience wrapper types
pub type DbPool = Pool<SqliteConnectionManager>;
pub type State = sync::Arc<Resources>;

/// Resources
/// template and database access
pub struct Resources {
    pub tera: Tera,
    pub db: DbPool,
}
impl Resources {
    pub fn new(tera: Tera, db: DbPool) -> Self {
        Self { tera: tera, db: db }
    }
}

pub fn migrant_config() -> Result<Config> {
    let dir = env::current_dir()?;
    let db_path = dir.join("db/upaste");
    let migration_dir = dir.join("migrations");
    let settings = Settings::configure_sqlite()
        .database_path(&db_path)?
        .migration_location(&migration_dir)?
        .build()?;
    Ok(Config::with_settings(&settings))
}

pub fn establish_connection<T: AsRef<Path>>(database_path: T) -> Connection {
    Connection::open(database_path.as_ref()).expect(&format!(
        "Error connection to {:?}.",
        database_path.as_ref()
    ))
}

fn establish_connection_pool<T: AsRef<Path>>(database_path: T) -> DbPool {
    let manager = SqliteConnectionManager::file(database_path.as_ref());
    Pool::new(manager).expect("Failed to create pool.")
}

static ERROR_404: &'static str = r##"
<html>
    <pre>
        Nothing to see here... <img src="https://badge-cache.kominick.com/badge/~(=^.^)-meow-yellow.svg?style=social"/>
    </pre>
</html>
"##;

fn init_db_sweeper(state: State) {
    thread::spawn(move || loop {
        let cutoff = chrono::Utc::now()
            .checked_sub_signed(chrono::Duration::seconds(MAX_PASTE_AGE_SECONDS))
            .expect("Error calculating stale cutoff date");
        let deleted: Result<i32> = (|| {
            let conn = state.db.get()?;
            Ok(models::Paste::delete_outdated(&conn, &cutoff)?)
        })();
        match deleted {
            Ok(count) => info!(" ** Cleaned out {} stale pastes **", count),
            Err(e) => panic!("Error cleaning stale pastes: {}", e),
        }
        thread::sleep(time::Duration::from_secs(60 * 10));
    });
}

pub fn start(host: &str) -> Result<()> {
    // get default host
    let host = if host.is_empty() {
        "localhost:3000"
    } else {
        host
    };

    // Set a custom logging format & change the env-var to "LOG"
    // e.g. LOG=info upaste serve
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - [{}] -> {}",
                Local::now().format("%Y-%m-%d_%H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or("<unknown>"),
                record.args()
            )
        })
        .parse(&env::var("LOG").unwrap_or_default())
        .init();

    // connect to our db
    let db = migrant_config()?
        .database_path()
        .chain_err(|| "Can't determine database path")?;
    let db_pool = establish_connection_pool(&db);
    info!(" ** Established database connection pool **");

    // compile our template and initialize template engine
    let mut tera = compile_templates!("templates/**/*");
    tera.autoescape_on(vec!["html"]);

    let state = sync::Arc::new(Resources::new(tera, db_pool));
    init_db_sweeper(state.clone());

    info!(" ** Listening at {} **", host);
    rouille::start_server(&host, move |request| {
        let state = state.clone();

        let now = Local::now().format("%Y-%m-%d %H:%M%S");
        let log_ok = |req: &rouille::Request, resp: &rouille::Response, elap: time::Duration| {
            let ms = (elap.as_secs() * 1_000) as f32 + (elap.subsec_nanos() as f32 / 1_000_000.);
            info!(
                "[{}] {} {} -> {} ({}ms)",
                now,
                req.method(),
                req.raw_url(),
                resp.status_code,
                ms
            )
        };
        let log_err = |req: &rouille::Request, elap: time::Duration| {
            let ms = (elap.as_secs() * 1_000) as f32 + (elap.subsec_nanos() as f32 / 1_000_000.);
            info!(
                "[{}] Handler Panicked: {} {} ({}ms)",
                now,
                req.method(),
                req.raw_url(),
                ms
            )
        };

        rouille::log_custom(request, log_ok, log_err, move || {
            match route_request(request, state) {
                Ok(resp) => rouille::content_encoding::apply(request, resp),
                Err(e) => {
                    use self::ErrorKind::*;
                    error!("Handler Error: {}", e);
                    match *e {
                        BadRequest(ref s) => {
                            let body = json!({ "error": s });
                            body.to_resp().unwrap().with_status_code(400)
                        }
                        DoesNotExist(_) => rouille::Response::html(ERROR_404).with_status_code(404),
                        UploadTooLarge(ref s) => {
                            // payload too large / request entity to large
                            let body = json!({ "error": s });
                            body.to_resp().unwrap().with_status_code(413)
                        }
                        OutOfSpace(ref s) => {
                            // service unavailable
                            let body = json!({ "error": s });
                            body.to_resp().unwrap().with_status_code(503)
                        }
                        _ => rouille::Response::text("Something went wrong").with_status_code(500),
                    }
                }
            }
        })
    });
}

/// Route the request to appropriate handler
fn route_request(request: &rouille::Request, state: State) -> Result<rouille::Response> {
    Ok(router!(request,
        (GET)   ["/"]               => { handlers::home(request, &state)? },
        (GET)   ["/favicon.ico"]    => { handlers::file("assets/favicon.ico")? },
        (GET)   ["/robots.txt"]     => { handlers::file("assets/robots.txt")? },
        (GET)   ["/appinfo"]        => { handlers::appinfo()? },
        (POST)  ["/new"]            => { handlers::new_paste(request, &state)? },
        (GET)   ["/raw/{key}", key: String] => { handlers::view_paste_raw(request, &state, &key)? },
        (GET)   ["/{key}", key: String]     => {
            // return a formatted paste, or show the default empty home page
            let paste_resp = handlers::view_paste(request, &state, &key);
            match paste_resp {
                Ok(resp) => resp,
                Err(e) => {
                    match *e {
                        ErrorKind::DoesNotExist(_) => {
                            info!("Paste not found: {}", key);
                            handlers::home(request, &state)?
                        }
                        _ => return Err(e),
                    }
                }
            }
        },
        _ => {
            // static files
            let static_resp = rouille::match_assets(&request, "assets");
            if static_resp.is_success() {
                static_resp
            } else {
                bail_fmt!(ErrorKind::DoesNotExist, "nothing here")
            }
        }
    ))
}
