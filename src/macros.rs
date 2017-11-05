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
///         let stmt = "insert into pastes (key, content, content_type) values ($1, $2, $3) \
///                     returning id, date_created, date_viewed";
///         try_insert_to_model!(conn.query(stmt, &[&self.key, &self.content, &self.content_type]) ;
///                              Paste ;
///                              id: 0, date_created: 1, date_viewed: 2 ;
///                              key: self.key, content: self.content, content_type: self.content_type)
///     }
/// }
/// ```
macro_rules! try_insert_to_model {
    ($query:expr ;
     $model:ident ;
     $($rowvar:ident : $rowindex:expr),* ;
     $($var:ident : $arg:expr),*) => {
        match $query {
            Ok(rows) => {
                match rows.iter().next() {
                    Some(row) => Ok($model {
                        $(
                            $rowvar: row.get($rowindex),
                         )*
                        $(
                            $var : $arg,
                         )*
                    }),
                    None => bail!(DoesNotExist; "No rows returned from table: {}", $model::table_name()),
                }
            }
            Err(e) => {
                Err(Error::from(e))
            }
        }
    }
}


/// Attempts to execute a `select`, taking the first row returned and
/// converting it into the associated model type. If more than one row
/// is returned, returns an `Error::MultipleRecords`.
///
/// Returns a `Result<Option<T>>` containing the given model
///
/// # Example
///
/// ```rust,ignore
/// fn touch_and_get(key: &str, conn: &Connection) -> Result<Paste> {
///     let stmt = "update pastes set date_viewed = NOW() where key = $1 \
///                 returning id, key, content, content_type, date_created, date_viewed";
///     try_query_one!(conn.query(stmt, &[&key]), Paste)
/// }
/// ```
macro_rules! try_query_one {
    ([$conn:expr, $stmt:expr, $args:expr], $model:ident) => {
        {
            let prepared = $conn.prepare($stmt)?;
            match prepared.query($args)? {
                Err(e) => {
                    Err(Error::from(e))
                }
                Ok(rows) => {
                    let mut rows = rows.iter();
                    let record = match rows.next() {
                        None => bail!(DoesNotExist; "No rows returned from table: {}", $model::table_name()),
                        Some(row) => Ok($model::from_row(row)),
                    };
                    match rows.next() {
                        None => record,
                        Some(_) => bail!(MultipleRecords; "Multiple rows returned from table: {}, expected one", $model::table_name()),
                    }
                }
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
/// fn exists(conn: &Connection, key: &str) -> Result<bool> {
///     let stmt = "select exists(select 1 from pastes where key = $1)";
///     try_query_aggregate!(conn.query(stmt, &[&key]), bool)
/// }
/// ```
macro_rules! try_query_aggregate {
    ($query:expr, $row_type:ty) => {
        match $query {
            Err(e) => {
                Err(Error::from(e))
            }
            Ok(rows) => {
                match rows.iter().next() {
                    None => bail!(DoesNotExist; "No rows returned"),
                    Some(row) => {
                        let val: $row_type = row.get(0);
                        Ok(val)
                    }
                }
            }
        }
    }
}


/// Attempts to unwrap a `Result`, returning an `iron::Response`
/// in the case of an `Err` and logging the error with `$msg`, if provided.
///
/// # Examples
///
/// ```rust,ignore
/// # returns an `Response` with `status::InternalServerError` and body `fmt::Disaply::fmt(err)`
/// try_server_error!(result)
///
/// ```rust,ignore
/// # returns an `Response` with `status::InternalServerError` and body `"error message"`
/// try_server_error!(result, "error message")
///
/// ```rust,ignore
/// # returns an `Response` with `status::NotImplemented` and body `"error message"`
/// try_server_error!(result, status::NotImplemented, "error message")
/// ```
macro_rules! try_server_error {
    ( $exp:expr ) => {
        match $exp {
            Ok(ok) => ok,
            Err(err) => {
                error!("Encountered: {}", err);
                return Ok(
                    Response::with(
                        (status::InternalServerError, format!("[ERROR] {}", err))
                    )
                );
            }
        }
    };
    ( $exp:expr, $msg:expr ) => {
        match $exp {
            Ok(ok) => ok,
            Err(err) => {
                error!("{} - from error: {}", $msg, err);
                return Ok(
                    Response::with(
                        (status::InternalServerError, $msg)
                    )
                );
            }
        }
    };
    ( $exp:expr ; $error_status:expr, $msg:expr ) => {
        match $exp {
            Ok(ok) => ok,
            Err(err) => {
                error!("{} - from error: {}", $msg, err);
                return Ok(
                    Response::with(
                        ($error_status, $msg)
                    )
                );
            }
        }
    }
}
