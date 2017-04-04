use schema::pastes;
use chrono::{DateTime, UTC};


#[derive(Queryable, Debug)]
pub struct Paste {
    pub id: i32,
    pub key: String,
    pub content: String,
    pub date_created: DateTime<UTC>,
    pub date_viewed: DateTime<UTC>,
}


#[derive(Insertable)]
#[table_name="pastes"]
pub struct NewPaste<'a> {
    pub key: String,
    pub content: &'a str,
}
