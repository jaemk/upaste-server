use chrono::{DateTime, Duration, TimeZone, Utc};
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
        .filter(|c| !matches!(*c, 'l' | '1' | 'i' | 'o' | '0'))
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
    pub fn insert(
        self,
        conn: &mut Connection,
        config: &crate::Config,
        ttl_seconds: Option<u32>,
        encryption_key: Option<&str>,
    ) -> Result<Paste> {
        let trans = conn.transaction()?;
        let key = get_new_key(&trans)?;
        let sig = crate::crypto::hmac_sign_with_key(&self.content, &config.signing_key);
        let (nonce, salt, content) = if let Some(enc_key) = encryption_key {
            let enc = crate::crypto::encrypt_with_key(&self.content, enc_key)?;
            (Some(enc.nonce), Some(enc.salt), enc.value)
        } else {
            (None, None, self.content)
        };

        let stmt = "insert into pastes (key, content, content_type, date_created, date_viewed, exp_date, nonce, salt, signature) values (?, ?, ?, ?, ?, ?, ?, ?, ?)";
        let now = Dt::now();
        let exp_date = ttl_seconds.map(|secs| {
            Dt(now
                .checked_add_signed(Duration::seconds(secs as i64))
                .expect("invalid date operation"))
        });
        let paste = try_insert_to_model!(
                [trans, stmt, &[&key as &dyn ToSql, &content, &self.content_type, &now, &now, &exp_date, &nonce, &salt, &sig]] ;
                Paste ;
                date_created: now.clone(), date_viewed: now,
                key: key, content: content, content_type: self.content_type, exp_date: exp_date,
                nonce: nonce, salt: salt, signature: Some(sig));
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
    pub exp_date: Option<Dt>,
    pub nonce: Option<String>,
    pub salt: Option<String>,
    pub signature: Option<String>,
}
impl Paste {
    #[inline]
    fn all_rows() -> &'static str {
        "id, key, content, content_type, date_created, date_viewed, exp_date, nonce, salt, signature"
    }

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
            exp_date: row.get(6).expect("row exp_date error"),
            nonce: row.get(7).expect("row nonce error"),
            salt: row.get(8).expect("row salt error"),
            signature: row.get(9).expect("row signature error"),
        })
    }

    pub fn exists(conn: &Connection, key: &str) -> Result<bool> {
        let stmt = "select exists(select 1 from pastes where key = $1)";
        Ok(try_query_row!([conn, stmt, &[&key]], u8) == 1)
    }

    pub fn count_outdated(conn: &Connection, date: &DateTime<Utc>) -> Result<i64> {
        let stmt = format!(
            "select count({}) from pastes where date_viewed < $1",
            Paste::all_rows()
        );
        Ok(try_query_row!([conn, &stmt, &[&date.timestamp()]], i64))
    }

    pub fn delete_outdated(
        conn: &Connection,
        max_cutoff: &DateTime<Utc>,
        now: &DateTime<Utc>,
    ) -> Result<i32> {
        let stmt =
            "delete from pastes where (exp_date is not null and exp_date < $1) or date_viewed < $2";
        Ok(conn.execute(stmt, &[&now.timestamp(), &max_cutoff.timestamp()])? as i32)
    }

    pub fn touch_and_get(
        conn: &mut Connection,
        key: &str,
        enc_key: Option<&str>,
        signing_key: &str,
    ) -> Result<Self> {
        let stmt_1 = "update pastes set date_viewed = ? where key = ?";
        let stmt_2 = format!("select {} from pastes where key = ?", Paste::all_rows());
        let trans = conn.transaction()?;
        trans.execute(stmt_1, &[&Dt::now() as &dyn ToSql, &key])?;
        let mut paste = trans
            .query_row(&stmt_2, &[&key], Self::from_row)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    format_err!(ErrorKind::DoesNotExist, "paste not found")
                }
                _ => ErrorKind::Sqlite(e),
            })?;
        if let Some(ref exp_date) = paste.exp_date {
            if exp_date.0 <= Utc::now() {
                trans.execute("delete from pastes where id = $1", &[&paste.id])?;
                return Err(ErrorKind::DoesNotExist("paste expired".to_string()).into());
            }
        }
        trans.commit()?;
        if matches!(
            (enc_key, paste.nonce.as_ref(), paste.salt.as_ref()),
            (Some(_), Some(_), Some(_))
        ) {
            let enc = crate::crypto::Enc {
                nonce: paste.nonce.clone().unwrap(),
                salt: paste.salt.clone().unwrap(),
                value: paste.content.clone(),
            };
            let dec = crate::crypto::decrypt_with_key(&enc, enc_key.unwrap())?;
            paste.content = dec;
        }
        if let Some(sig) = &paste.signature {
            if !crate::crypto::hmac_verify_with_key(&paste.content, &sig, &signing_key) {
                error!("decryption error, invalid signature");
                bail_fmt!(ErrorKind::DecryptionError, "decryption failure")
            }
        }
        Ok(paste)
    }
}

pub static CONTENT_TYPES: [&str; 147] = [
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
