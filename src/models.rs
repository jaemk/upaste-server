use chrono::{DateTime, TimeZone, Utc};
use rand::{self, Rng};
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{self, Connection};
use std::ops;

use crate::errors::*;

/// Generate a new random key
fn gen_key(n_chars: usize) -> String {
    #[allow(unused_imports)]
    #[allow(deprecated)]
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
    while Paste::exists(&conn, &new_key)? {
        n_chars += 1;
        new_key = gen_key(n_chars);
    }
    Ok(new_key)
}

#[derive(Debug, Clone)]
pub struct Dt(DateTime<Utc>);
impl Dt {
    pub fn now() -> Self {
        Dt(Utc::now())
    }
}
impl ops::Deref for Dt {
    type Target = DateTime<Utc>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ops::DerefMut for Dt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromSql for Dt {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        value
            .as_i64()
            .map(|timestamp| Dt(Utc.timestamp(timestamp, 0)))
    }
}
impl ToSql for Dt {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        Ok(self.timestamp().into())
    }
}

pub struct NewPaste {
    pub content: String,
    pub content_type: String,
}

impl NewPaste {
    pub fn insert(self, conn: &mut Connection) -> Result<Paste> {
        let trans = conn.transaction()?;
        let key = get_new_key(&trans)?;
        let stmt = "insert into pastes (key, content, content_type, date_created, date_viewed) values (?, ?, ?, ?, ?)";
        let now = Dt::now();
        let paste = try_insert_to_model!(
                [trans, stmt, &[&key as &ToSql, &self.content, &self.content_type, &now, &now]] ;
                Paste ;
                date_created: now.clone(), date_viewed: now,
                key: key, content: self.content, content_type: self.content_type);
        trans.commit()?;
        Ok(paste)
    }
}

#[derive(Debug)]
pub struct Paste {
    pub id: i64,
    pub key: String,
    pub content: String,
    pub content_type: String,
    pub date_created: Dt,
    pub date_viewed: Dt,
}
impl Paste {
    pub fn table_name() -> &'static str {
        "pastes"
    }

    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0).expect("row id error"),
            key: row.get(1).expect("row key error"),
            content: row.get(2).expect("row content error"),
            content_type: row.get(3).expect("row content_type error"),
            date_created: row.get(4).expect("row date_created error"),
            date_viewed: row.get(5).expect("row date_viewed error"),
        })
    }

    pub fn exists(conn: &Connection, key: &str) -> Result<bool> {
        let stmt = "select exists(select 1 from pastes where key = $1)";
        Ok(try_query_row!([conn, stmt, &[&key]], u8) == 1)
    }

    pub fn count_outdated(conn: &Connection, date: &DateTime<Utc>) -> Result<i64> {
        let stmt = "select count(*) from pastes where date_viewed < $1";
        Ok(try_query_row!([conn, stmt, &[&date.timestamp()]], i64))
    }

    pub fn delete_outdated(conn: &Connection, date: &DateTime<Utc>) -> Result<i32> {
        let stmt = "delete from pastes where date_viewed < ?";
        Ok(conn.execute(stmt, &[&date.timestamp()])? as i32)
    }

    pub fn touch_and_get(conn: &mut Connection, key: &str) -> Result<Self> {
        let stmt_1 = "update pastes set date_viewed = ? where key = ?";
        let stmt_2 = "select * from pastes where key = ?";
        let trans = conn.transaction()?;
        trans.execute(stmt_1, &[&Dt::now() as &ToSql, &key])?;
        let paste = trans
            .query_row(stmt_2, &[&key], Self::from_row)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    format_err!(ErrorKind::DoesNotExist, "paste not found")
                }
                _ => ErrorKind::Sqlite(e),
            })?;
        trans.commit()?;
        Ok(paste)
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
