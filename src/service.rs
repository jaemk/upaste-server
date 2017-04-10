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

use diesel::pg::PgConnection;
use r2d2::{Config, Pool};
use r2d2_diesel::ConnectionManager;

use tera::Tera;

use iron::prelude::*;
use iron::typemap::Key;
use iron::middleware::{BeforeMiddleware};
use router::Router;
use logger::Logger;
use persistent::{Write, Read};
use mount::Mount;
use staticfile::Static;

use routes;


/// Wrapped r2d2_pool/diesel-connection
type PgPool = Pool<ConnectionManager<PgConnection>>;


#[derive(Copy, Clone)]
/// Database pool wrapper type for iron request type-map
pub struct DB;
impl Key for DB { type Value = PgPool; }


#[derive(Copy, Clone)]
/// Tera templates wrapper type for iron request type-map
pub struct TERA;
impl Key for TERA { type Value = Tera; }


/// Custom logger to print out access info
pub struct InfoLog;
impl BeforeMiddleware for InfoLog {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        println!("[{:?}]: {}", req.method, req.url);
        Ok(())
    }
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<()> {
        Err(err)
    }
}


//fn establish_connection() -> PgConnection {
//    dotenv().ok();
//    let db_url = env::var("DATABASE_URL")
//        .expect("DATABASE_URL must be set.");
//    PgConnection::establish(&db_url)
//        .expect(&format!("Error connection to {}.", db_url))
//}


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
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    Pool::new(config, manager).expect("Failed to create pool.")
}


pub fn start(host: &str, db: Option<&str>, log: bool) {
    // get default host
    let host = if host.is_empty() { "localhost:3000" } else { host };

    // connect to our db
    let db_pool = establish_connection_pool(db);
    println!(" ** Established database connection pool **");

    // compile our template and initialize template engine
    let mut tera = compile_templates!("templates/**/*");
    tera.autoescape_on(vec!["html"]);

    // mount our url endpoints
    let mut router = Router::new();
    routes::mount(&mut router);

    // chain our router,
    // insert our mutable db_pool into request.typemap,
    // insert out template engine into request.typemap,
    // initialize and link our loggers if we're logging
    let mut chain = Chain::new(router);
    chain.link(Write::<DB>::both(db_pool));
    chain.link(Read::<TERA>::both(tera));
    if log {
        env_logger::init().unwrap();
        let (log_before, log_after) = Logger::new(None);
        chain.link_before(log_before);
        chain.link_before(InfoLog);
        chain.link_after(log_after);
    }

    // mount our chain of services and a static file handler
    let mut mount = Mount::new();
    mount.mount("/", chain)
         .mount("/static/", Static::new(Path::new("static")));

    println!(" ** Serving at {}", host);
    Iron::new(mount).http(host).unwrap();
}
