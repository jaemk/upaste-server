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
        try_insert_to_model!([conn, stmt, &[&self.key, &self.content, &self.content_type]] ;
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
        try_query_aggregate!([conn, stmt, &[&key]], bool)
    }

    pub fn count_outdated(conn: &Connection, date: &DateTime<UTC>) -> Result<i64> {
        let stmt = "select count(*) from pastes where date_viewed < $1";
        try_query_aggregate!([conn, stmt, &[&date]], i64)
    }

    pub fn delete_outdated(conn: &Connection, date: &DateTime<UTC>) -> Result<i64> {
        let stmt = "with d as (delete from pastes where date_viewed < $1 returning true) \
                    select count(*) from d";
        try_query_aggregate!([conn, stmt, &[&date]], i64)
    }

    pub fn touch_and_get(key: &str, conn: &Connection) -> Result<Paste> {
        let stmt = "update pastes set date_viewed = NOW() where key = $1 \
                    returning id, key, content, content_type, date_created, date_viewed";
        try_query_one!([conn, stmt, &[&key]], Paste)
    }
}


pub static CONTENT_TYPES: [&'static str; 147] = [
    "text",
    "abap",
    "abc",
    "actionscript",
    "ada",
    "apache_conf",
    "applescript",
    "asciidoc",
    "assembly_x86",
    "autohotkey",
    "batchfile",
    "bro",
    "c9search",
    "c_cpp",
    "cirru",
    "clojure",
    "cobol",
    "coffee",
    "coldfusion",
    "csharp",
    "css",
    "curly",
    "dart",
    "diff",
    "django",
    "d",
    "dockerfile",
    "dot",
    "drools",
    "eiffel",
    "ejs",
    "elixir",
    "elm",
    "erlang",
    "forth",
    "fortran",
    "ftl",
    "gcode",
    "gherkin",
    "gitignore",
    "glsl",
    "gobstones",
    "golang",
    "graphqlschema",
    "groovy",
    "haml",
    "handlebars",
    "haskell_cabal",
    "haskell",
    "haxe",
    "hjson",
    "html_elixir",
    "html",
    "html_ruby",
    "ini",
    "io",
    "jack",
    "jade",
    "java",
    "javascript",
    "jsoniq",
    "json",
    "jsp",
    "jsx",
    "julia",
    "kotlin",
    "latex",
    "lean",
    "less",
    "liquid",
    "lisp",
    "live_script",
    "livescript",
    "logiql",
    "lsl",
    "lua",
    "luapage",
    "lucene",
    "makefile",
    "markdown",
    "mask",
    "matlab",
    "maze",
    "mel",
    "mips_assembler",
    "mipsassembler",
    "mushcode",
    "mysql",
    "nix",
    "nsis",
    "objectivec",
    "ocaml",
    "pascal",
    "perl",
    "pgsql",
    "php",
    "pig",
    "plain_text",
    "powershell",
    "praat",
    "prolog",
    "properties",
    "protobuf",
    "python",
    "razor",
    "rdoc",
    "rhtml",
    "r",
    "rst",
    "ruby",
    "rust",
    "sass",
    "scad",
    "scala",
    "scheme",
    "scss",
    "sh",
    "sjs",
    "smarty",
    "snippets",
    "soy_template",
    "space",
    "sparql",
    "sql",
    "sqlserver",
    "stylus",
    "svg",
    "swift",
    "swig",
    "tcl",
    "tex",
    "textile",
    "text",
    "toml",
    "tsx",
    "turtle",
    "twig",
    "typescript",
    "vala",
    "vbscript",
    "velocity",
    "verilog",
    "vhdl",
    "wollok",
    "xml",
    "xquery",
    "yaml",
];
