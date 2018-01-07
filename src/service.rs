//! Service
//!  - Initialize external and persistent services/structs
//!  - Initialize loggers
//!  - Mount url endpoints to `handlers` functions
//!  - Mount static file handler
//!
use std::env;
use std::time;
use std::path::{Path, PathBuf};
use std::sync;
use std::thread;
use env_logger;

use chrono::{self, Local};
use rusqlite::Connection;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::{Config, Pool};
use tera::Tera;
use rouille;
use migrant_lib;

use errors::*;
use handlers;
use models;
use {ToResponse, MAX_PASTE_AGE_SECONDS};


// convenience wrapper types
pub type DbPool = Pool<SqliteConnectionManager>;
pub type Ctx = sync::Arc<Context>;


/// Request context with template and database access
pub struct Context {
    pub tera: sync::RwLock<Tera>,
    pub db: sync::Mutex<DbPool>,
}
impl Context {
    pub fn new(tera: Tera, db: DbPool) -> Self {
        Self {
            tera: sync::RwLock::new(tera),
            db: sync::Mutex::new(db),
        }
    }
}


/// Grab the database path from out migrant configuration file
pub fn migrant_database_path() -> Option<PathBuf> {
    let dir = env::current_dir()
        .expect("failed to get current directory");
    migrant_lib::search_for_settings_file(&dir)
        .and_then(|p| migrant_lib::Config::from_settings_file(&p).ok())
        .and_then(|config| config.database_path().ok())
}


pub fn establish_connection<T: AsRef<Path>>(database_path: T) -> Connection {
    Connection::open(database_path.as_ref())
        .expect(&format!("Error connection to {:?}.", database_path.as_ref()))
}


fn establish_connection_pool<T: AsRef<Path>>(database_path: T) -> DbPool {
    let config = Config::default();
    let manager = SqliteConnectionManager::file(database_path.as_ref());
    Pool::new(config, manager).expect("Failed to create pool.")
}


static ERROR_404: &'static str = r##"
<html>
    <pre>
        Nothing to see here... <img src="https://badge-cache.kominick.com/badge/~(=^.^)-meow-yellow.svg?style=social"/>
    </pre>
</html>
"##;


fn init_db_sweeper(ctx: Ctx) {
    thread::spawn(move || {
        loop {
            let cutoff = chrono::Utc::now()
                .checked_sub_signed(chrono::Duration::seconds(MAX_PASTE_AGE_SECONDS))
                .expect("Error calculating stale cutoff date");
            let deleted: Result<i32> = (|| {
                let conn = ctx.db.lock()
                    .map_err(|_| format_err!(ErrorKind::SyncPoison, "lock poisoned"))?
                    .get()?;
                Ok(models::Paste::delete_outdated(&conn, &cutoff)?)
            })();
            match deleted {
                Ok(count) => info!(" ** Cleaned out {} stale pastes **", count),
                Err(e) => panic!("Error cleaning stale pastes: {}", e),
            }
            thread::sleep(time::Duration::from_secs(60 * 10));
        }
    });
}


pub fn start(host: &str) -> Result<()> {
    // get default host
    let host = if host.is_empty() { "localhost:3000" } else { host };

    // Set a custom logging format & change the env-var to "LOG"
    // e.g. LOG=info upaste serve
    env_logger::LogBuilder::new()
        .format(|record| {
            format!("{} [{}] - [{}] -> {}",
                Local::now().format("%Y-%m-%d_%H:%M:%S"),
                record.level(),
                record.location().module_path(),
                record.args()
                )
            })
        .parse(&env::var("LOG").unwrap_or_default())
        .init()
        .expect("failed to initialize logger");

    // connect to our db
    let db = migrant_database_path()
        .ok_or_else(|| format_err!(ErrorKind::Msg, "Can't determine database path"))?;
    let db_pool = establish_connection_pool(&db);
    info!(" ** Established database connection pool **");

    // compile our template and initialize template engine
    let mut tera = compile_templates!("templates/**/*");
    tera.autoescape_on(vec!["html"]);

    let ctx = sync::Arc::new(Context::new(tera, db_pool));
    init_db_sweeper(ctx.clone());

    info!(" ** Listening at {} **", host);
    rouille::start_server(&host, move |request| {
        let ctx = ctx.clone();
        let now = Local::now().format("%Y-%m-%d %H:%M%S");
        let log_ok = |req: &rouille::Request, resp: &rouille::Response, elap: time::Duration| {
            let ms = (elap.as_secs() * 1_000) as f32 + (elap.subsec_nanos() as f32 / 1_000_000.);
            info!("[{}] {} {} -> {} ({}ms)", now, req.method(), req.raw_url(), resp.status_code, ms)
        };
        let log_err = |req: &rouille::Request, elap: time::Duration| {
            let ms = (elap.as_secs() * 1_000) as f32 + (elap.subsec_nanos() as f32 / 1_000_000.);
            info!("[{}] Handler Panicked: {} {} ({}ms)", now, req.method(), req.raw_url(), ms)
        };
        // dispatch and handle errors
        rouille::log_custom(request, log_ok, log_err, move || {
            let response = match route_request(request, ctx) {
                Ok(resp) => resp,
                Err(e) => {
                    use self::ErrorKind::*;
                    error!("Handler Error: {}", e);
                    match *e {
                        BadRequest(ref s) => {
                            // bad request
                            let body = json!({"error": s});
                            body.to_resp().unwrap().with_status_code(400)
                        }
                        DoesNotExist(_) => {
                            // not found
                            rouille::Response::html(ERROR_404).with_status_code(404)
                        }
                        UploadTooLarge(ref s) => {
                            // payload too large / request entity to large
                            let body = json!({"error": s});
                            body.to_resp().unwrap().with_status_code(413)
                        }
                        OutOfSpace(ref s) => {
                            // service unavailable
                            let body = json!({"error": s});
                            body.to_resp().unwrap().with_status_code(503)
                        }
                        _ => rouille::Response::text("Something went wrong").with_status_code(500),
                    }
                }
            };
            response
        })
    });
}


/// Route the request to appropriate handler
fn route_request(request: &rouille::Request, ctx: Ctx) -> Result<rouille::Response> {
    Ok(router!(request,
        (GET)   ["/"]               => { handlers::home(request, &ctx)? },
        (GET)   ["/favicon.ico"]    => { handlers::file("assets/favicon.ico")? },
        (GET)   ["/robots.txt"]     => { handlers::file("assets/robots.txt")? },
        (GET)   ["/appinfo"]        => { handlers::appinfo()? },
        (POST)  ["/new"]            => { handlers::new_paste(request, &ctx)? },
        (GET)   ["/raw/{key}", key: String] => { handlers::view_paste_raw(request, &ctx, &key)? },
        (GET)   ["/{key}", key: String]     => {
            // return a formatted paste, or show the default empty home page
            let paste_resp = handlers::view_paste(request, &ctx, &key);
            match paste_resp {
                Ok(resp) => resp,
                Err(e) => {
                    match *e {
                        ErrorKind::DoesNotExist(_) => {
                            info!("Paste not found: {}", key);
                            handlers::home(request, &ctx)?
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

