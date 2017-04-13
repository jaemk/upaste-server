//! Handlers
//!  - Endpoint handlers
//!
use std::io::Read;
use chrono::UTC;
use rand::{self, Rng};

use diesel;
use diesel::prelude::*;

use iron::prelude::*;
use iron::{status};
use iron::modifiers;
use router::Router;
use persistent::{Read as PerRead, Write};
use tera::Context;

use models;
use service::{DB, TERA};
use errors::*;


lazy_static! {
    static ref CONTENT_TYPES: Vec<String> = {
        models::Paste::content_types()
    };
}


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


/// Generate a new random key
fn gen_key(n_chars: usize) -> String {
    rand::thread_rng()
        .gen_ascii_chars()
        .take(n_chars)
        .collect::<String>()
}


/// Endpoint for creating a new paste record
pub fn new_paste(req: &mut Request) -> IronResult<Response> {
    use params::{Params, Value};
    use schema::pastes;
    use schema::pastes::dsl::*;
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
    let paste_type = paste_type.unwrap_or("text".to_string());

    // create a new paste.key, making sure it isn't already in use
    let mut n_chars = 8;
    let mut new_key = gen_key(n_chars);
    while pastes.filter(key.eq(&new_key)).first::<models::Paste>(&*conn).is_ok() {
        n_chars += 1;
        new_key = gen_key(n_chars);
    }

    let new_paste = models::NewPaste { key: new_key, content: &paste_content, content_type: &paste_type};
    let new_paste = diesel::insert(&new_paste).into(pastes::table).get_result::<models::Paste>(&*conn);
    let new_paste = match new_paste {
        Ok(p) => p,
        _ => return Ok(Response::with((status::InternalServerError, "Error creating post"))),
    };

    let resp = json!({
        "message": "success!",
        "key": &new_paste.key,
    });
    Ok(Response::with((status::Ok, resp.to_string())))
}


/// Helper for pulling out a paste
fn get_paste(req: &mut Request) -> Result<models::Paste> {
    use schema::pastes::dsl::*;
    let req_key: String;
    {
        let ref k = req.extensions.get::<Router>().unwrap().find("key").unwrap();
        req_key = k.to_string();
    }
    let conn = get_dbconn!(req);
    let paste: models::Paste = diesel::update(pastes.filter(key.eq(&req_key)))
        .set(date_viewed.eq(UTC::now()))
        .get_result(&*conn)
        .chain_err(|| "Paste not found")?;
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
        _ => return Ok(Response::with((status::Found,
                                       modifiers::Redirect(url_for!(req, "home"))))),
    };

    let arc = req.get::<PerRead<TERA>>().unwrap();
    let templates = arc.as_ref();
    let mut context = Context::new();
    context.add("paste_key", &paste.key);
    context.add("content", &paste.content);
    context.add("content_type", &paste.content_type);
    context.add("content_types", &*CONTENT_TYPES);
    let content = templates.render("core/edit.html", &context).unwrap();
    let content_type = mime!(Text/Html);
    Ok(Response::with((content_type, status::Ok, content)))
}


/// Endpoint for returning landing page
pub fn home(req: &mut Request) -> IronResult<Response> {
    let templates = get_templates!(req);
    let mut context = Context::new();
    context.add("content_types", &*CONTENT_TYPES);
    let content = templates.render("core/edit.html", &context).unwrap();
    let content_type = mime!(Text/Html);
    Ok(Response::with((content_type, status::Ok, content)))
}
