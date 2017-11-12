//! Handlers
//!  - Endpoint handlers
//!
use std::io::{self, BufRead};

use rand::{self, Rng};
use rouille::{Request, Response};
use tera::Context;
use rusqlite::Connection;

use errors::*;
use service::Ctx;
use models::{self, CONTENT_TYPES};
use {ToResponse, FromRequestQuery, MAX_PASTE_BYTES};


/// Generate a new random key
fn gen_key(n_chars: usize) -> String {
    use std::ascii::AsciiExt;
    rand::thread_rng()
        .gen_ascii_chars()
        .map(|c| c.to_ascii_lowercase())
        .filter(|c| match *c {
            'l' | '1' | 'i' | 'o' | '0' => false,
            _ => true,
        })
        .take(n_chars)
        .collect::<String>()
}


/// Create a new paste.key, making sure it isn't already in use
fn get_new_key(conn: &Connection) -> Result<String> {
    let mut n_chars = 5;
    let mut new_key = gen_key(n_chars);
    while models::Paste::exists(&conn, &new_key)? {
        n_chars += 1;
        new_key = gen_key(n_chars);
    }
    Ok(new_key)
}


#[derive(Debug, Deserialize)]
pub struct NewPasteQueryParams {
    #[serde(rename="type")]
    pub type_: Option<String>,
}


/// Endpoint for creating a new paste record
pub fn new_paste(req: &Request, ctx: &Ctx) -> Result<Response> {
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
        if n == 0 { break; }

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
                if n == 0 || over >= 10_000 { break; }
            }
            bail_fmt!(ErrorKind::UploadTooLarge, "Upload too large")
        }
    }
    let paste_content = String::from_utf8(content)?;

    let new_paste = {
        let mut conn = ctx.db.lock()
            .map_err(|_| format_err!(ErrorKind::SyncPoison, "lock poisoned"))?
            .get()?;
        let trans = conn.transaction()?;
        let new_key = get_new_key(&trans)?;
        let new_paste = models::NewPaste { key: new_key, content: paste_content, content_type: paste_type };
        let new_paste = new_paste.insert(&trans)?;
        trans.commit()?;
        new_paste
    };

    Ok(json!({"message": "success", "key": &new_paste.key}).to_resp()?)
}


fn get_paste(ctx: &Ctx, key: &str) -> Result<models::Paste> {
    let mut conn = ctx.db.lock()
        .map_err(|_| format_err!(ErrorKind::SyncPoison, "lock poisoned"))?
        .get()?;
    models::Paste::touch_and_get(&mut conn, key)
}


/// Endpoint for returning raw paste content
pub fn view_paste_raw(_req: &Request, ctx: &Ctx, key: &str) -> Result<Response> {
    let paste = get_paste(ctx, &key)?;
    Ok(Response::text(paste.content))
}


/// Endpoint for returning formatted paste content
pub fn view_paste(_req: &Request, ctx: &Ctx, key: &str) -> Result<Response> {
    let paste = get_paste(ctx, &key)?;
    let templates = ctx.tera.read()
        .map_err(|_| format_err!(ErrorKind::SyncPoison, "lock poisoned"))?;
    let mut context = Context::new();
    context.add("paste_key", &paste.key);
    context.add("content", &paste.content);
    context.add("content_type", &paste.content_type);
    context.add("content_types", &&CONTENT_TYPES[..]);
    let content = templates.render("core/edit.html", &context).unwrap();
    Ok(Response::html(content))
}


/// Endpoint for returning landing page
pub fn home(_req: &Request, ctx: &Ctx) -> Result<Response> {
    let templates = ctx.tera.read()
        .map_err(|_| format_err!(ErrorKind::SyncPoison, "lock poisoned"))?;
    let mut context = Context::new();
    context.add("content_types", &&CONTENT_TYPES[..]);
    let content = templates.render("core/edit.html", &context).unwrap();
    Ok(Response::html(content))
}


pub fn appinfo() -> Result<Response> {
    Ok(json!({
        "version": env!("CARGO_PKG_VERSION"),
    }).to_resp()?)
}

