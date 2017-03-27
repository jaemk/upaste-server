//! Crate imports
//!
extern crate dotenv;
extern crate uuid;
extern crate chrono;
extern crate rand;

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate r2d2;
extern crate r2d2_diesel;

extern crate serde;
#[macro_use] extern crate serde_json;

#[macro_use] extern crate tera;
#[macro_use] extern crate mime;

// iron things
extern crate iron;
extern crate router;
extern crate logger;
extern crate persistent;
extern crate staticfile;
extern crate mount;

extern crate env_logger;

#[macro_use] extern crate error_chain;
pub mod errors {
    error_chain! {
        foreign_links {
            Diesel(::diesel::result::Error);
        }
    }
}

pub mod schema;
pub mod models;
pub mod service;
pub mod routes;
pub mod handlers;

