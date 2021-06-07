//! Handlers
//!  - Endpoint handlers
//!
use std::fs;
use std::io::{self, BufRead};
use std::path;

use rouille::{self, Request, Response};
use tera::Context;

use crate::errors::*;
use crate::models::{self, CONTENT_TYPES};
use crate::service::State;
use crate::{FromRequestBody, FromRequestQuery, ToResponse};

#[derive(Debug, serde::Deserialize)]
pub struct NewPasteQueryParams {
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub ttl_seconds: Option<u32>,
}

/// Endpoint for creating a new paste record
pub fn new_paste(req: &Request, state: &State) -> Result<Response> {
    let paste_params = req.parse_query_params::<NewPasteQueryParams>()?;
    let paste_type = paste_params.type_.unwrap_or_else(|| "auto".to_string());
    let paste_ttl_seconds = paste_params.ttl_seconds;
    let encryption_key = req.header("x-upaste-encryption-key");

    let mut content = match req.header("content-length") {
        Some(ct_len) => {
            let ct_len = ct_len.parse::<usize>()?;
            if ct_len > state.config.max_paste_bytes {
                bail_fmt!(ErrorKind::UploadTooLarge, "Upload too large")
            }
            Vec::with_capacity(ct_len)
        }
        None => vec![],
    };

    let mut byte_count = 0;
    let mut stream = io::BufReader::new(req.data().expect("Unable to read request body"));
    loop {
        let n = {
            let buf = stream.fill_buf()?;
            content.extend_from_slice(&buf);
            buf.len()
        };
        stream.consume(n);
        if n == 0 {
            break;
        }

        byte_count += n;
        if byte_count > state.config.max_paste_bytes {
            error!("Paste too large");
            // See if we can drain the rest of the stream and send a real response
            // before we kill the connection
            let mut over = 0;
            loop {
                let n = {
                    let buf = stream.fill_buf()?;
                    buf.len()
                };
                stream.consume(n);
                over += n;
                if n == 0 || over >= 10_000 {
                    break;
                }
            }
            bail_fmt!(ErrorKind::UploadTooLarge, "Upload too large")
        }
    }
    let paste_content = String::from_utf8(content)?;

    let new_paste = {
        let mut conn = state.db.get()?;
        let new_paste = models::NewPaste {
            content: paste_content,
            content_type: paste_type,
        };
        new_paste.insert(&mut conn, &state.config, paste_ttl_seconds, encryption_key)?
    };

    json!({"message": "success", "key": &new_paste.key}).to_resp()
}

fn get_paste(state: &State, key: &str, enc_key: Option<&str>) -> Result<models::Paste> {
    let mut conn = state.db.get()?;
    models::Paste::touch_and_get(&mut conn, key, enc_key, &state.config.signing_key)
}

#[derive(serde::Serialize)]
struct PasteContent {
    pub key: String,
    pub content: String,
    pub content_type: String,
}

pub fn view_paste_json(req: &Request, state: &State, key: &str) -> Result<Response> {
    let enc_key = req.header("x-upaste-encryption-key");
    let paste = get_paste(state, &key, enc_key)?;
    let content = PasteContent {
        key: paste.key,
        content: paste.content,
        content_type: paste.content_type,
    };
    json!({ "paste": content }).to_resp()
}

/// Endpoint for returning raw paste content
pub fn view_paste_raw(req: &Request, state: &State, key: &str) -> Result<Response> {
    let enc_key = req.header("x-upaste-encryption-key");
    match get_paste(state, &key, enc_key) {
        Ok(paste) => Ok(Response::text(paste.content)),
        Err(e) => match e.kind() {
            ErrorKind::DecryptionError(_) => json!({
                "error": "decryption_key_required",
                "message": "x-upaste-encryption-key header is required"
            })
            .to_resp()
            .map(|r| r.with_status_code(400)),
            _ => Err(e),
        },
    }
}

#[derive(serde::Deserialize)]
struct ViewParams {
    encryption_key: Option<String>,
}

/// Endpoint for returning formatted paste content
pub fn view_paste(req: &Request, state: &State, key: &str) -> Result<Response> {
    let mut enc_key = req.header("x-upaste-encryption-key").map(String::from);
    if enc_key.is_none() && req.method() == "POST" {
        let params = req.parse_json_body::<ViewParams>()?;
        enc_key = params.encryption_key;
    }
    let mut context = Context::new();
    match get_paste(state, &key, enc_key.as_deref()) {
        Ok(paste) => {
            context.add("paste_key", &paste.key);
            context.add("content", &paste.content);
            context.add("content_type", &paste.content_type);
            context.add("content_types", &&CONTENT_TYPES[..]);
        }
        Err(e) => match e.kind() {
            ErrorKind::DecryptionError(_) => {
                context.add("paste_key", &key);
                context.add("content", &"");
                context.add("content_type", &"");
                context.add("content_types", &&CONTENT_TYPES[..]);
                context.add("encryption_key_required", &true);
            }
            _ => return Err(e),
        },
    }
    let content = state.tera.render("core/edit.html", &context).unwrap();
    Ok(Response::html(content))
}

/// Endpoint for returning landing page
pub fn home(_req: &Request, state: &State) -> Result<Response> {
    let mut context = Context::new();
    context.add("content_types", &&CONTENT_TYPES[..]);
    let content = state.tera.render("core/edit.html", &context).unwrap();
    Ok(Response::html(content))
}

/// Endpoint for returning a file from a given path
pub fn file(path: &str) -> Result<Response> {
    let path = path::Path::new(path);
    let ext = path
        .extension()
        .and_then(::std::ffi::OsStr::to_str)
        .unwrap_or("");
    let f = fs::File::open(&path)?;
    Ok(Response::from_file(rouille::extension_to_mime(ext), f))
}

/// Return appinfo/health-check
pub fn status() -> Result<Response> {
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "hash": include_str!("../commit_hash.txt").trim(),
    })
    .to_resp()
}
