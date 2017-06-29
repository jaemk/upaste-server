//! Service
//!  - Initialize external and persistent services/structs
//!  - Initialize loggers
//!  - Mount url endpoints to `handlers` functions
//!  - Mount static file handler
//!
use std::env;
use std::path::Path;
use dotenv::dotenv;
use env_logger;

use chrono::Local;
use postgres::{self, Connection};
use r2d2_postgres::{self, PostgresConnectionManager};

use r2d2::{Config, Pool};

use tera::Tera;

use iron::prelude::*;
use iron::typemap::Key;
use iron::middleware::AfterMiddleware;
use iron::status;
use router::{Router, NoRoute};
use logger;
use persistent::{Write, Read};
use mount::Mount;
use staticfile::Static;

use routes;


/// Wrapped r2d2_pool/diesel-connection
type PgPool = Pool<PostgresConnectionManager>;


#[derive(Copy, Clone)]
/// Database pool wrapper type for iron request type-map
pub struct DB;
impl Key for DB { type Value = PgPool; }


#[derive(Copy, Clone)]
/// Tera templates wrapper type for iron request type-map
pub struct TERA;
impl Key for TERA { type Value = Tera; }


pub fn establish_connection(database_url: Option<&str>) -> Connection {
    let db_url = match database_url {
        Some(url) => url.into(),
        None => {
            dotenv().ok();
            env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set.")
        },
    };
    Connection::connect(db_url.clone(), postgres::TlsMode::None)
        .expect(&format!("Error connection to {}.", db_url))
}


fn establish_connection_pool(database_url: Option<&str>) -> PgPool {
    let db_url = match database_url {
        Some(url) => url.into(),
        None => {
            dotenv().ok();
            env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set.")
        },
    };
    let config = Config::default();
    let manager = PostgresConnectionManager::new(db_url, r2d2_postgres::TlsMode::None)
        .expect("failed to open pooled connection");
    Pool::new(config, manager).expect("Failed to create pool.")
}


static ERROR_404: &'static str = r##"
<html>
    <pre>
        Nothing to see here... <img src="https://badge-cache.kominick.com/badge/~(=^.^)-meow-yellow.svg?style=social"/>
    </pre>
</html>
"##;

/// Custom 404 Error handler/content
struct Error404;
impl AfterMiddleware for Error404 {
    fn catch(&self, _req: &mut Request, e: IronError) -> IronResult<Response> {
        if let Some(_) = e.error.downcast::<NoRoute>() {
            return Ok(Response::with((status::NotFound, mime!(Text/Html), ERROR_404)))
        }
        Err(e)
    }
}


pub fn start(host: &str, db: Option<&str>) {
    // get default host
    let host = if host.is_empty() { "localhost:3000" } else { host };

    // Set a custom logging format & change the env-var to "LOG"
    // e.g. LOG=info badge-cache serve
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

    // iron request-middleware loggers
    let format = logger::Format::new("[{request-time}] [{status}] {method} {uri}").unwrap();
    let (log_before, log_after) = logger::Logger::new(Some(format));

    // connect to our db
    let db_pool = establish_connection_pool(db);
    info!(" ** Established database connection pool **");

    // compile our template and initialize template engine
    let mut tera = compile_templates!("templates/**/*");
    tera.autoescape_on(vec!["html"]);

    // mount our url endpoints
    let mut router = Router::new();
    routes::mount(&mut router);

    // chain our router,
    // insert our mutable db_pool into request.typemap,
    // insert our template engine into request.typemap,
    // link our loggers if we're logging
    let mut chain = Chain::new(router);
    chain.link_before(log_before);
    chain.link_after(log_after);
    chain.link_after(Error404);
    chain.link(Write::<DB>::both(db_pool));
    chain.link(Read::<TERA>::both(tera));

    // mount our chain of services and a static file handler
    let mut mount = Mount::new();
    mount.mount("/", chain)
         .mount("/favicon.ico", Static::new(Path::new("static/favicon.ico")))
         .mount("/robots.txt", Static::new(Path::new("static/robots.txt")))
         .mount("/static/", Static::new(Path::new("static")));

    info!(" ** Serving at {} **", host);
    Iron::new(mount).http(host).unwrap();
}
