use schema::pastes;
use chrono::{DateTime, UTC};


#[derive(Insertable)]
#[table_name="pastes"]
pub struct NewPaste<'a> {
    pub key: String,
    pub content: &'a str,
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
            "apache",
            "css",
            "http",
            "javascript",
            "ruby",
            "bash",
            "ini",
            "diff",
            "json",
            "sql",
            "markdown",
            "perl",
            "c++",
            "html",  // and xml?
            "java",
            "nginx",
            "python",
            "clojure",
            "ocaml",
            "scala",
            "vim",  // or vim-script?
            "go",
            "haskell",
            "swift",
            "elixer",
            "django",
            "dockerfile",
            "elm",
            "lisp",
            "rust",
            "yaml",
        ].iter()
         .map(|s| s.to_string())
         .collect::<Vec<_>>()
    }
}

