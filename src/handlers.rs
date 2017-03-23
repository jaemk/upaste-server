//! Handlers
//!  - Endpoint handlers
//!
use std::io::Read;
use uuid::Uuid;

use diesel;
use diesel::prelude::*;

use iron::prelude::*;
use iron::{status};
use router::Router;
use persistent::{Read as PerRead, Write};
use tera::Context;

use models;
use service::{DB, TERA};
use errors::*;


/// Macro to hide boiler plate of pulling a pooled db connection
/// out of the request typemap
macro_rules! get_dbconn {
    ($request:expr) => {
        {
            let mutex = $request.get::<Write<DB>>().unwrap();
            let pool = mutex.lock().unwrap().clone();
            pool.get().unwrap()
        }
    }
}


/// custom error for iron result
use std::error::Error;
use std::fmt::{self, Debug};
#[derive(Debug)]
struct StringError(String);
impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl Error for StringError {
    fn description(&self) -> &str { &*self.0 }
}
impl StringError {
    fn new(s: &str) -> StringError {
        StringError(s.to_string())
    }
}


/// Endpoint for creating a new paste record
pub fn new_paste(req: &mut Request) -> IronResult<Response> {
    use schema::pastes;
    use schema::pastes::dsl::*;
    let conn = get_dbconn!(req);

    let mut paste_content = String::new();
    let _ = req.body.read_to_string(&mut paste_content);

    // create a new paste.key, making sure it isn't already in use
    let mut new_key = Uuid::new_v4();
    while pastes.filter(key.eq(&new_key)).first::<models::Paste>(&*conn).is_ok() {
        new_key = Uuid::new_v4();
    }

    let new_paste = models::NewPaste { key: new_key, content: &paste_content};
    let new_paste = diesel::insert(&new_paste).into(pastes::table).get_result::<models::Paste>(&*conn);
    let new_paste = match new_paste {
        Ok(p) => p,
        // v -- return status::error response instead... just wanted to try to get this to work
        _ => return Err(IronError::new(StringError::new("Error saving new paste"), status::BadRequest)),
    };

    let resp = json!({
        "message": "success!",
        "key": &new_paste.key.simple().to_string()
    });
    Ok(Response::with((status::Ok, resp.to_string())))
}


fn get_paste(req: &mut Request) -> Result<models::Paste> {
    use schema::pastes::dsl::*;
    let req_key: Uuid;
    {
        let ref k = req.extensions.get::<Router>().unwrap().find("key").unwrap();
        req_key = Uuid::parse_str(k).unwrap();
    }
    let conn = get_dbconn!(req);
    let paste: models::Paste = pastes.filter(key.eq(&req_key)).first(&*conn)
        .chain_err(|| "Paste not found")?;
    Ok(paste)
}


pub fn get_paste_raw(req: &mut Request) -> IronResult<Response> {
    let paste = get_paste(req).unwrap();
    Ok(Response::with((status::Ok, paste.content)))
}


pub fn view_paste(req: &mut Request) -> IronResult<Response> {
    let paste = get_paste(req).unwrap();

    let arc = req.get::<PerRead<TERA>>().unwrap();
    let templates = arc.as_ref();
    let mut context = Context::new();
    context.add("content_type", &"text");
    context.add("content", &paste.content);
    let content = templates.render("core/paste.html", &context).unwrap();
    let content_type = mime!(Text/Html);
    Ok(Response::with((content_type, status::Ok, content)))
}


pub fn home(req: &mut Request) -> IronResult<Response> {
    let arc = req.get::<PerRead<TERA>>().unwrap();
    let templates = arc.as_ref();
    let context = Context::new();
    let content = templates.render("core/base.html", &context).unwrap();
    let content_type = mime!(Text/Html);
    Ok(Response::with((content_type, status::Ok, content)))
}
