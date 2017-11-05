//! Service
//!  - Initialize external and persistent services/structs
//!  - Initialize loggers
//!  - Mount url endpoints to `handlers` functions
//!  - Mount static file handler
//!
use std::env;
use std::path::Path;
use std::time;
use env_logger;

use chrono::Local;
use rusqlite::{self, Connection};
use r2d2_sqlite::{self, SqliteConnectionManager};
use r2d2::{Config, Pool};
use tera::Tera;
use rouille;
use migrant_lib;

use errors::*;
use {ToResponse};


type DbPool = Pool<SqliteConnectionManager>;


fn migrant_connect_string() -> Option<String> {
    let dir = env::current_dir()
        .expect("failed to get current directory");
    migrant_lib::search_for_config(&dir)
        .and_then(|p| migrant_lib::Config::load(&p).ok())
        .and_then(|config| config.connect_string().ok())
}


pub fn establish_connection(database_path: &str) -> Connection {
    Connection::open(database_path)
        .expect(&format!("Error connection to {}.", database_path))
}


fn establish_connection_pool(database_path: &str) -> DbPool {
    let config = Config::default();
    let manager = SqliteConnectionManager::file(database_path);
    Pool::new(config, manager).expect("Failed to create pool.")
}


static ERROR_404: &'static str = r##"
<html>
    <pre>
        Nothing to see here... <img src="https://badge-cache.kominick.com/badge/~(=^.^)-meow-yellow.svg?style=social"/>
    </pre>
</html>
"##;


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
    let db = migrant_connect_string()
        .ok_or_else(|| format_err!(ErrorKind::Msg, "Can't determine database connection string"))?;
    let db_pool = establish_connection_pool(&db);
    info!(" ** Established database connection pool **");

    // compile our template and initialize template engine
    let mut tera = compile_templates!("templates/**/*");
    tera.autoescape_on(vec!["html"]);

    rouille::start_server(&host, move |request| {
        let db_pool = db_pool.clone();
        let start = time::Instant::now();

        // dispatch and handle errors
        let response = match route_request(request, db_pool) {
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
                    DoesNotExist(ref s) => {
                        // not found
                        let body = json!({"error": s});
                        body.to_resp().unwrap().with_status_code(404)
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

        let elapsed = start.elapsed();
        let elapsed = (elapsed.as_secs() * 1_000) as f32 + (elapsed.subsec_nanos() as f32 / 1_000_000.);
        info!("[{}] {} {:?} {}ms", request.method(), response.status_code, request.url(), elapsed);
        response
    });
}


/// Route the request to appropriate handler
fn route_request(request: &rouille::Request, db_pool: DbPool) -> Result<rouille::Response> {
    Ok(router!(request,
        (GET)   (/)    => { json!({"message": "hey!"}).to_resp()? },
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

