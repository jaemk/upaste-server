//! Handlers
//!  - Endpoint handlers
//!
use std::io::Read;
use rand::{self, Rng};

use serde_json;

use iron::prelude::*;
use iron::{status};
use iron::modifiers;
use router::Router;
use persistent::{Read as PerRead, Write};
use tera::Context;
use postgres::Connection;

use models;
use models::CONTENT_TYPES;
use service::{DB, TERA};
use errors::*;



/// Macro to pull a pooled db connection out a request typemap
macro_rules! get_dbconn {
    ($request:expr) => {
        {
            let mutex = $request.get::<Write<DB>>().unwrap();
            let pool = mutex.lock().unwrap().clone();
            pool.get().unwrap()
        }
    }
}


/// Macro to pull our template renderer out of a request typemap
macro_rules! get_templates {
    ($request:expr) => {
        {
            let arc = $request.get::<PerRead<TERA>>().unwrap();
            arc.clone()
        }
    }
}


#[derive(Serialize)]
struct NewPasteResponse<'a> {
    message: &'a str,
    key: &'a str,
}


/// Generate a new random key
fn gen_key(n_chars: usize) -> String {
    use std::ascii::AsciiExt;
    rand::thread_rng()
        .gen_ascii_chars()
        .filter(|c| match c.to_ascii_lowercase() {
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


/// Endpoint for creating a new paste record
pub fn new_paste(req: &mut Request) -> IronResult<Response> {
    use params::{Params, Value};
    let conn = get_dbconn!(req);

    let mut paste_content = String::new();
    let _ = req.body.read_to_string(&mut paste_content);

    let paste_type = match req.get_ref::<Params>() {
        Ok(map) => match map.find(&["type"]) {
            Some(&Value::String(ref name)) => Some(name.to_string()),
            _ => None,
        },
        _ => None,
    };
    let paste_type = paste_type.unwrap_or_else(|| "auto".to_string());
    let new_key = try_server_error!(get_new_key(&conn), "Unable to create new key");

    let new_paste = models::NewPaste { key: new_key, content: paste_content, content_type: paste_type};
    let new_paste = new_paste.insert(&conn);
    let new_paste = try_server_error!(new_paste, "Error creating post");

    let resp = NewPasteResponse { message: "success!", key: &new_paste.key };
    let resp = try_server_error!(serde_json::to_string(&resp), "Error serializing response");
    Ok(Response::with((status::Ok, resp)))
}


/// Helper for pulling out a paste
fn get_paste(req: &mut Request) -> Result<models::Paste> {
    let req_key = {
        let ref k = req.extensions.get::<Router>()
            .expect("failed to extract router params")
            .find("key")
            .expect("`key` router param missing");
        k.to_string()
    };
    let conn = get_dbconn!(req);
    let paste = models::Paste::touch_and_get(&req_key, &conn)
        .map_err(|e| format_err!("[key: {}]: {}", req_key, e))?;
    Ok(paste)
}


/// Endpoint for returning raw paste content
pub fn view_paste_raw(req: &mut Request) -> IronResult<Response> {
    let paste = get_paste(req).unwrap();
    Ok(Response::with((mime!(Text/Plain), status::Ok, paste.content)))
}


/// Endpoint for returning formatted paste content
pub fn view_paste(req: &mut Request) -> IronResult<Response> {
    let paste = match get_paste(req) {
        Ok(p) => p,
        Err(e) => {
            info!("Paste not found: {} -- Redirecting", e);
            return Ok(Response::with(
                    (status::Found, modifiers::Redirect(url_for!(req, "home")))))
        }
    };

    let arc = req.get::<PerRead<TERA>>().unwrap();
    let templates = arc.as_ref();
    let mut context = Context::new();
    context.add("paste_key", &paste.key);
    context.add("content", &paste.content);
    context.add("content_type", &paste.content_type);
    context.add("content_types", &&CONTENT_TYPES[..]);
    let content = templates.render("core/edit.html", &context).unwrap();
    let content_type = mime!(Text/Html);
    Ok(Response::with((content_type, status::Ok, content)))
}


/// Endpoint for returning landing page
pub fn home(req: &mut Request) -> IronResult<Response> {
    let templates = get_templates!(req);
    let mut context = Context::new();
    context.add("content_types", &&CONTENT_TYPES[..]);
    let content = templates.render("core/edit.html", &context).unwrap();
    let content_type = mime!(Text/Html);
    Ok(Response::with((content_type, status::Ok, content)))
}
