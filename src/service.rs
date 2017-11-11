//! Service
//!  - Initialize external and persistent services/structs
//!  - Initialize loggers
//!  - Mount url endpoints to `handlers` functions
//!  - Mount static file handler
//!
use std::io;
use std::env;
use std::path::{Path, PathBuf};
use std::sync;
use env_logger;

use chrono::Local;
use rusqlite::Connection;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::{Config, Pool};
use tera::Tera;
use rouille;
use migrant_lib;

use errors::*;
use handlers;
use {ToResponse};


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
    migrant_lib::search_for_config(&dir)
        .and_then(|p| migrant_lib::Config::load(&p).ok())
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


/// Logging workaround until custom logging api is available
/// https://github.com/tomaka/rouille/pull/158
struct MyLogger {
    buf: Vec<u8>,
}
impl MyLogger {
    fn new() -> Self {
        Self { buf: vec![] }
    }
}
impl io::Write for MyLogger {
    fn write(&mut self, buf: &[u8]) -> ::std::io::Result<usize> {
        self.buf.extend_from_slice(buf);
        if let Some(byte) = buf.last() {
            if *byte == '\n' as u8 {
                let s = ::std::str::from_utf8(&self.buf)
                    .map_err(|_| ::std::io::Error::new(::std::io::ErrorKind::InvalidData, "bad utf8"))?;
                info!("{}", s);
            }
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> ::std::io::Result<()> {
        panic!("unsupported")
    }
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

    info!(" ** Listening at {} **", host);
    rouille::start_server(&host, move |request| {
        let ctx = ctx.clone();
        // dispatch and handle errors
        rouille::log(request, MyLogger::new(), move || {
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
        (GET)   (/)     => { handlers::home(request, &ctx)? },
        (POST)  (/new)  => { handlers::new_paste(request, &ctx)? },
        (GET)   (/p/raw/{key: String}) => { handlers::view_paste_raw(request, &ctx, &key)? },
        (GET)   (/p/{key: String})   => {
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

