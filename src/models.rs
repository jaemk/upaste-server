use schema::pastes;
use uuid::Uuid;
use chrono::{DateTime, UTC};

#[derive(Queryable, Debug)]
pub struct Paste {
    pub id: i32,
    pub key: Uuid,
    pub content: String,
    pub date_created: DateTime<UTC>,
    pub date_viewed: DateTime<UTC>,
}

#[derive(Insertable)]
#[table_name="pastes"]
pub struct NewPaste<'a> {
    pub key: Uuid,
    pub content: &'a str,
}
