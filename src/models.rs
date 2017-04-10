use schema::pastes;
use chrono::{DateTime, UTC};


#[derive(Insertable)]
#[table_name="pastes"]
pub struct NewPaste<'a> {
    pub key: String,
    pub content: &'a str,
    pub content_type: &'a str,
}


#[derive(Queryable, Debug)]
pub struct Paste {
    pub id: i32,
    pub key: String,
    pub content: String,
    pub content_type: String,
    pub date_created: DateTime<UTC>,
    pub date_viewed: DateTime<UTC>,
}
impl Paste {
    pub fn content_types() -> Vec<String> {
        [
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

