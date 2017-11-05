use std;
use rusqlite;
use serde_json;
use migrant_lib;


error_chain! {
    foreign_links {
        Io(std::io::Error);
        Sqlite(rusqlite::Error);
        ParseInt(std::num::ParseIntError);
        Json(serde_json::Error);
        Migrant(migrant_lib::Error);
    }
    errors {
        BadRequest(s: String) {
            description("BadRequest")
            display("BadRequest Error: {}", s)
        }
        DoesNotExist(s: String) {
            description("DoesNotExist")
            display("DoesNotExist Error: {}", s)
        }
        MultipleRecords(s: String) {
            description("MultipleRecords")
            display("MultipleRecords Error: {}", s)
        }
        UploadTooLarge(s: String) {
            description("UploadTooLarge")
            display("UploadTooLarge Error: {}", s)
        }
        OutOfSpace(s: String) {
            description("OutOfSpace")
            display("OutOfSpace Error: {}", s)
        }
    }
}

