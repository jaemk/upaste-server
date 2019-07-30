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
use crate::{FromRequestQuery, ToResponse, MAX_PASTE_BYTES};

#[derive(Debug, Deserialize)]
pub struct NewPasteQueryParams {
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

/// Endpoint for creating a new paste record
pub fn new_paste(req: &Request, state: &State) -> Result<Response> {
    let paste_type = req.parse_query_params::<NewPasteQueryParams>()?.type_;
    let paste_type = paste_type.unwrap_or_else(|| "auto".to_string());

    let mut content = match req.header("content-length") {
        Some(ct_len) => {
            let ct_len = ct_len.parse::<usize>()?;
            if ct_len > MAX_PASTE_BYTES {
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
        if byte_count > MAX_PASTE_BYTES {
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
        let new_paste = new_paste.insert(&mut conn)?;
        new_paste
    };

    Ok(json!({"message": "success", "key": &new_paste.key}).to_resp()?)
}

fn get_paste(state: &State, key: &str) -> Result<models::Paste> {
    let mut conn = state.db.get()?;
    models::Paste::touch_and_get(&mut conn, key)
}

/// Endpoint for returning raw paste content
pub fn view_paste_raw(_req: &Request, state: &State, key: &str) -> Result<Response> {
    let paste = get_paste(state, &key)?;
    Ok(Response::text(paste.content))
}

/// Endpoint for returning formatted paste content
pub fn view_paste(_req: &Request, state: &State, key: &str) -> Result<Response> {
    let paste = get_paste(state, &key)?;
    let mut context = Context::new();
    context.add("paste_key", &paste.key);
    context.add("content", &paste.content);
    context.add("content_type", &paste.content_type);
    context.add("content_types", &&CONTENT_TYPES[..]);
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
pub fn appinfo() -> Result<Response> {
    Ok(json!({
        "version": env!("CARGO_PKG_VERSION"),
    })
    .to_resp()?)
}
