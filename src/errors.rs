error_chain! {
    foreign_links {
        Io(std::io::Error);
        Utf8(std::string::FromUtf8Error);
        Sqlite(rusqlite::Error);
        ParseInt(std::num::ParseIntError);
        Json(serde_json::Error);
        R2D2(r2d2::Error);
        Migrant(migrant_lib::Error);
    }
    errors {
        SyncPoison(s: String) {
            description("SyncPoison")
            display("SyncPoison Error: {}", s)
        }
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
        DecryptionError(s: String) {
            description("DecryptionError")
            display("DecryptionError Error: {}", s)
        }
    }
}
