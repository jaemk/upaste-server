use postgres::{self, Connection};
use chrono::{DateTime, UTC};

use errors::*;


pub struct NewPaste {
    pub key: String,
    pub content: String,
    pub content_type: String,
}

impl NewPaste {
    pub fn insert(self, conn: &Connection) -> Result<Paste> {
        let stmt = "insert into pastes (key, content, content_type) values ($1, $2, $3) \
                    returning id, date_created, date_viewed";
        try_insert_to_model!(conn.query(stmt, &[&self.key, &self.content, &self.content_type]) ;
                             Paste ;
                             id: 0, date_created: 1, date_viewed: 2 ;
                             key: self.key, content: self.content, content_type: self.content_type)
    }
}


#[derive(Debug)]
pub struct Paste {
    pub id: i32,
    pub key: String,
    pub content: String,
    pub content_type: String,
    pub date_created: DateTime<UTC>,
    pub date_viewed: DateTime<UTC>,
}
impl Paste {
    pub fn table_name() -> &'static str {
        "pastes"
    }

    pub fn from_row(row: postgres::rows::Row) -> Self {
        Self {
            id:             row.get(0),
            key:            row.get(1),
            content:        row.get(2),
            content_type:   row.get(3),
            date_created:   row.get(4),
            date_viewed:    row.get(5),
        }
    }

    pub fn exists(conn: &Connection, key: &str) -> Result<bool> {
        let stmt = "select exists(select 1 from pastes where key = $1)";
        try_query_aggregate!(conn.query(stmt, &[&key]), bool)
    }

    pub fn count_outdated(conn: &Connection, date: &DateTime<UTC>) -> Result<i64> {
        let stmt = "select count(*) from pastes where date_viewed < $1";
        try_query_aggregate!(conn.query(stmt, &[&date]), i64)
    }

    pub fn delete_outdated(conn: &Connection, date: &DateTime<UTC>) -> Result<i64> {
        let stmt = "with d as (delete from pastes where date_viewed < $1 returning true) \
                    select count(*) from d";
        try_query_aggregate!(conn.query(stmt, &[&date]), i64)
    }

    pub fn touch_and_get(key: &str, conn: &Connection) -> Result<Paste> {
        let stmt = "update pastes set date_viewed = NOW() where key = $1 \
                    returning id, key, content, content_type, date_created, date_viewed";
        try_query_one!(conn.query(stmt, &[&key]), Paste)
    }

    pub fn content_types() -> Vec<String> {
        [
            "auto",
            "text",
            "python",
            "rust",
            "javascript",
            "json",
            "bash",
            "perl",
            "clojure",
            "java",
            "css",
            "html/xml",
            "markdown",
            "ruby",
            "django",
            "ini",
            "diff",
            "sql",
            "c++",
            "apache",
            "nginx",
            "ocaml",
            "scala",
            "vim",
            "go",
            "haskell",
            "swift",
            "elixer",
            "dockerfile",
            "elm",
            "lisp",
            "yaml",
            "http",
        ].iter()
         .map(|s| s.to_string())
         .collect::<Vec<_>>()
    }
}

