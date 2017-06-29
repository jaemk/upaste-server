//! Crate imports
//!
extern crate dotenv;
extern crate chrono;
extern crate time;
extern crate rand;
extern crate clap;


extern crate postgres;
extern crate r2d2_postgres;

extern crate r2d2;

extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

#[macro_use] extern crate tera;
#[macro_use] extern crate mime;
extern crate params;

// iron things
extern crate iron;
#[macro_use] extern crate router;
extern crate logger;
extern crate persistent;
extern crate staticfile;
extern crate mount;

extern crate env_logger;

extern crate migrant_lib;


#[macro_use] pub mod errors;
#[macro_use] pub mod macros;

pub mod service;
pub mod models;
pub mod routes;
pub mod handlers;
pub mod admin;

