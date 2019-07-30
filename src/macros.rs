/*!
Macros

For working with rust-postgres and iron
*/

// -------------
// error-chain
// -------------

/// Helper for formatting Errors that wrap strings
macro_rules! format_err {
    ($error:expr, $str:expr) => {
        $error(format!($str))
    };
    ($error:expr, $str:expr, $($arg:expr),*) => {
        $error(format!($str, $($arg),*))
    }
}

/// Helper for formatting strings with error-chain's `bail!` macro
macro_rules! bail_fmt {
    ($error:expr, $str:expr) => {
        bail!(format_err!($error, $str))
    };
    ($error:expr, $str:expr, $($arg:expr),*) => {
        bail!(format_err!($error, $str, $($arg),*))
    }
}

/// Attempts to execute an `insert`, using provided and returned columns
/// to return a populated instance of the associated model `struct`.
///
/// Returns a `Result` containing the given model
///
/// # macro syntax
///
/// ```rust,ignore
/// try_insert_to_model!(
///     query-expr-to-execute ;
///     model-type ;
///     struct-field: row-index, * ;
///     struct-field: value, *
/// )
/// ```
///
/// # Example
///
/// ```rust,ignore
/// impl NewPaste {
///     fn create(self, conn: &Connection) -> Result<Paste> {
///         let stmt = "insert into pastes (key, content, content_type, date_created, date_viewed) values (?, ?, ?, ?, ?)";
///         let now = Dt::now();
///         try_insert_to_model!([conn, stmt, &[&self.key, &self.content, &self.content_type, &now, &now]] ;
///                             Paste ;
///                             date_created: now.clone, date_viewed: now,
///                             key: self.key, content: self.content, content_type: self.content_type)
///     }
/// }
/// ```
macro_rules! try_insert_to_model {
    ([$conn:expr, $stmt:ident, $params:expr] ;
     $model:ident ;
     $($var:ident : $arg:expr),*) => {
        {
            let mut stmt = $conn.prepare($stmt)?;
            let row_id = stmt.insert($params)?;
            $model {
                id: row_id,
                $(
                    $var: $arg,
                )*
            }
        }
    }
}

/// Attempts to execute some statement that returns a single row
/// of some `type` that implements `FromSql`
///
/// # Example
///
/// ```rust,ignore
/// pub fn exists(conn: &Connection, key: &str) -> Result<bool> {
///     let stmt = "select exists(select 1 from pastes where key = $1)";
///     Ok(try_query_row!([conn, stmt, &[&key]], u8) == 1)
/// }
/// ```
macro_rules! try_query_row {
    ([$conn:expr, $stmt:expr, $params:expr], $row_type:ty) => {
        $conn.query_row($stmt, $params, |row| {
            let val: $row_type = row.get(0).unwrap();
            Ok(val)
        })?
    };
}
