//! Service
//!  - Initialize external and persistent services/structs
//!  - Initialize loggers
//!  - Mount url endpoints to `handlers` functions
//!  - Mount static file handler
//!
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
use rusqlite::Connection;
use tera::Tera;

use crate::errors::*;
use crate::handlers;
use crate::models;
use crate::ToResponse;

// convenience wrapper types
pub type DbPool = Pool<SqliteConnectionManager>;
pub type State = sync::Arc<Resources>;

/// Resources
/// template and database access
pub struct Resources {
    pub tera: Tera,
    pub db: DbPool,
    pub config: crate::Config,
}
impl Resources {
    pub fn new(tera: Tera, db: DbPool, config: crate::Config) -> Self {
        Self { tera, db, config }
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
    Connection::open(database_path.as_ref())
        .unwrap_or_else(|_| panic!("Error connection to {:?}.", database_path.as_ref()))
}

fn establish_connection_pool<T: AsRef<Path>>(database_path: T) -> DbPool {
    let manager = SqliteConnectionManager::file(database_path.as_ref());
    Pool::new(manager).expect("Failed to create pool.")
}

static ERROR_404: &str = r##"
<html>
    <pre>
        Nothing to see here... <img src="https://badge-cache.kominick.com/badge/~(=^.^)-meow-yellow.svg?style=social"/>
    </pre>
</html>
"##;

fn init_db_sweeper(state: State) {
    thread::spawn(move || loop {
        let deleted: Result<i32> = (|| {
            let cutoff = chrono::Utc::now()
                .checked_sub_signed(chrono::Duration::seconds(
                    state.config.max_paste_age_seconds,
                ))
                .chain_err(|| "Error calculating stale cutoff date")?;
            let conn = state.db.get()?;
            models::Paste::delete_outdated(&conn, &cutoff, &chrono::Utc::now())
        })();
        match deleted {
            Ok(count) => info!(" ** Cleaned out {} stale pastes **", count),
            Err(e) => error!("Error cleaning stale pastes: {}", e),
        }
        thread::sleep(time::Duration::from_secs(20));
    });
}

pub fn start() -> Result<()> {
    let config = crate::Config::load();
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
        .parse(&config.log_level)
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

    let state = sync::Arc::new(Resources::new(tera, db_pool, config.clone()));
    init_db_sweeper(state.clone());

    let host = config.host();
    info!(" ** Listening at {} **", &host);
    rouille::start_server_with_pool(&host, None, move |request| {
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
                    match e.kind() {
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

fn _handle_key(request: &rouille::Request, state: &State, key: &str) -> Result<rouille::Response> {
    // return a formatted paste, or show the default empty home page
    let paste_resp = handlers::view_paste(request, &state, &key);
    Ok(match paste_resp {
        Ok(resp) => resp,
        Err(e) => match e.kind() {
            ErrorKind::DoesNotExist(_) => {
                info!("Paste not found: {}", key);
                handlers::home(request, &state)?
            }
            _ => return Err(e),
        },
    })
}

/// Route the request to appropriate handler
fn route_request(request: &rouille::Request, state: State) -> Result<rouille::Response> {
    Ok(router!(request,
        (GET)   ["/"]               => { handlers::home(request, &state)? },
        (GET)   ["/favicon.ico"]    => { handlers::file("assets/favicon.ico")? },
        (GET)   ["/robots.txt"]     => { handlers::file("assets/robots.txt")? },
        (GET)   ["/status"]         => { handlers::status()? },
        (POST)  ["/new"]            => { handlers::new_paste(request, &state)? },
        (GET)   ["/raw/{key}", key: String] =>  { handlers::view_paste_raw(request, &state, &key)? },
        (GET)   ["/json/{key}", key: String] => { handlers::view_paste_json(request, &state, &key)? },
        (GET)   ["/{key}", key: String]     =>  { _handle_key(request, &state, &key)? },
        (POST)  ["/{key}", key: String]     =>  { _handle_key(request, &state, &key)? },
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
