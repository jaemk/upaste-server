use std;
use postgres;
use migrant_lib;


pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Msg(String),
    DoesNotExist(String),
    MultipleRecords(String),
    Io(std::io::Error),
    Pg(postgres::error::Error),
    ParseInt(std::num::ParseIntError),
    Migrant(migrant_lib::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use self::Error::*;
        match *self {
            Msg(ref s)              => write!(f, "Error: {}", s),
            DoesNotExist(ref s)     => write!(f, "DoesNotExist Error: {}", s),
            MultipleRecords(ref s)  => write!(f, "MultipleRecords Error: {}", s),
            Io(ref e)               => write!(f, "Io Error: {}", e),
            Pg(ref e)               => write!(f, "Postgres Error: {}", e),
            ParseInt(ref e)         => write!(f, "ParseInt Error: {}", e),
            Migrant(ref e)          => write!(f, "Migrant Error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "upaste Error"
    }

    fn cause(&self) -> Option<&std::error::Error> {
        use self::Error::*;
        Some(match *self {
            Io(ref e)       => e,
            Pg(ref e)       => e,
            ParseInt(ref e) => e,
            Migrant(ref e)  => e,
            _ => return None
        })
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<postgres::error::Error> for Error {
    fn from(e: postgres::error::Error) -> Error {
        Error::Pg(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Error {
        Error::ParseInt(e)
    }
}

impl From<migrant_lib::Error> for Error {
    fn from(e: migrant_lib::Error) -> Error {
        Error::Migrant(e)
    }
}


macro_rules! format_err {
    ($literal:expr) => {
        Error::Msg(format!($literal))
    };
    ($literal:expr, $($arg:expr),*) => {
        Error::Msg(format!($literal, $($arg),*))
    };
    ($e_type:expr ; $literal:expr) => {
        $e_type(format!($literal))
    };
    ($e_type:expr ; $literal:expr, $($arg:expr),*) => {
        $e_type(format!($literal, $($arg),*))
    };
}


macro_rules! bail {
    ($msg:expr) => {
        return Err(format_err!(Error::Msg ; $msg))
    };
    ($literal:expr, $($arg:expr),*) => {
        return Err(format_err!(Error::Msg ; $literal, $($arg),*))
    };
    (Msg; $msg:expr) => {
        return Err(format_err!(Error::Msg ; $msg))
    };
    (Msg; $literal:expr, $($arg:expr),*) => {
        return Err(format_err!(Error::Msg ; $literal, $($arg),*))
    };
    (DoesNotExist; $msg:expr) => {
        return Err(format_err!(Error::DoesNotExist ; $msg))
    };
    (DoesNotExist; $literal:expr, $($arg:expr),*) => {
        return Err(format_err!(Error::DoesNotExist ; $literal, $($arg),*))
    };
    (MultipleRecords; $msg:expr) => {
        return Err(format_err!(Error::MultipleRecords ; $msg))
    };
    (MultipleRecords; $literal:expr, $($arg:expr),*) => {
        return Err(format_err!(Error::MultipleRecords ; $literal, $($arg),*))
    };
}
