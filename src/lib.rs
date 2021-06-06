//! Crate imports
//!
#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_json;
extern crate serde_urlencoded;
#[macro_use]
extern crate tera;
#[macro_use]
extern crate log;
#[macro_use]
extern crate rouille;

pub mod errors;
#[macro_use]
pub mod macros;

pub mod admin;
mod crypto;
pub mod handlers;
pub mod models;
pub mod service;

use errors::*;
use std::io::Read;

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

fn env_or(k: &str, default: &str) -> String {
    std::env::var(k).unwrap_or_else(|_| default.to_string())
}

#[derive(Clone)]
pub struct Config {
    pub version: String,

    // host to listen on, defaults to localhost
    pub host: String,
    pub port: u16,

    pub log_level: String,

    // key used for encrypting protected pastes
    pub encryption_key: String,
    // key used to derive signature of paste content
    pub signing_key: String,

    pub max_paste_bytes: usize,
    pub max_paste_age_seconds: i64,
}
impl Config {
    pub fn load() -> Self {
        let version = std::fs::File::open("commit_hash.txt")
            .map(|mut f| {
                let mut s = String::new();
                f.read_to_string(&mut s).expect("Error reading commit_hasg");
                s
            })
            .unwrap_or_else(|_| "unknown".to_string());
        Self {
            version,
            host: env_or("HOST", "localhost"),
            port: env_or("PORT", "3030").parse().expect("invalid port"),
            log_level: env_or("LOG_LEVEL", "INFO"),
            encryption_key: env_or("ENCRYPTION_KEY", "01234567890123456789012345678901"),
            signing_key: env_or("SIGNING_KEY", "01234567890123456789012345678901"),
            max_paste_bytes: env_or("MAX_PASTE_BYTES", "1000000")
                .parse()
                .unwrap_or_else(|e| panic!("invalid MAX_PASTE_BYTES {:?}", e)),
            // 60 * 60 * 24 * 30
            max_paste_age_seconds: env_or("MAX_PASTE_AGE_SECONDS", "2592000")
                .parse()
                .unwrap_or_else(|e| panic!("invalid MAX_PASTE_AGE_SECONDS {:?}", e)),
        }
    }

    pub fn host(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
