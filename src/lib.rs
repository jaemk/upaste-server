//! Crate imports
//!
#![recursion_limit = "1024"]
#[macro_use] extern crate error_chain;
extern crate chrono;
extern crate time;
extern crate rand;
extern crate clap;
extern crate rusqlite;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate serde_urlencoded;
#[macro_use] extern crate tera;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate migrant_lib;
#[macro_use] extern crate rouille;

pub mod errors;
#[macro_use] pub mod macros;

pub mod service;
pub mod handlers;
pub mod models;
pub mod admin;

use std::io::Read;
use errors::*;

// ------------------------------------------------
// Traits for augmenting `rouille`
// ------------------------------------------------

/// Trait for parsing `json` from `rouille::Request` bodies into some type `T`
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct PostData {
///     name: String,
///     age: u32,
/// }
///```
///
/// For a request with a body containing `json`
///
/// ```rust,ignore
/// let post_data = request.parse_json_body::<PostData>()?;
/// println!("{}", post_data.name);
/// ```
pub trait FromRequestBody {
    fn parse_json_body<T: serde::de::DeserializeOwned>(&self) -> Result<T>;
}

impl FromRequestBody for rouille::Request {
    fn parse_json_body<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let mut body = self.data().expect("Can't read request body twice");
        let mut s = String::new();
        body.read_to_string(&mut s)?;
        let data = serde_json::from_str::<T>(&s)
            .map_err(|_| format_err!(ErrorKind::BadRequest, "malformed data"))?;
        Ok(data)
    }
}


/// Trait for parsing query string parameters from `rouille::Request` urls into some type `T`
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct PostData {
///     name: String,
///     age: u32,
/// }
///```
///
/// For a request with url query parameters
///
/// ```rust,ignore
/// let param_data = request.parse_query_params::<PostData>()?;
/// println!("{}", post_data.name);
/// ```
pub trait FromRequestQuery {
    fn parse_query_params<T: serde::de::DeserializeOwned>(&self) -> Result<T>;
}
impl FromRequestQuery for rouille::Request {
    fn parse_query_params<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let qs = self.raw_query_string();
        let params = serde_urlencoded::from_str::<T>(qs)
            .map_err(|_| format_err!(ErrorKind::BadRequest, "malformed data"))?;
        Ok(params)
    }
}


/// Trait for constructing `rouille::Response`s from other types
pub trait ToResponse {
    fn to_resp(&self) -> Result<rouille::Response>;
}
impl ToResponse for serde_json::Value {
    fn to_resp(&self) -> Result<rouille::Response> {
        let s = serde_json::to_string(self)?;
        let resp = rouille::Response::from_data("application/json", s.as_bytes());
        Ok(resp)
    }
}

