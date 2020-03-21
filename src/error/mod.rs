use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum ErrorCode {
    ScTableCorruption = 0x01,
}

#[derive(Debug)]
pub struct Error {
    error_code: ErrorCode,
    reason: ErrorStr,
    fatal: bool
}

#[derive(Debug)]
pub enum ErrorStr {
    Owned(String),
    StaticBorrow(&'static str)
}

impl From<String> for ErrorStr {
    fn from(s: String) -> Self {
        ErrorStr::Owned(s)
    }
}

impl From<&'static str> for ErrorStr {
    fn from(s: &'static str) -> Self {
        ErrorStr::StaticBorrow(s)
    }
}

impl Error {
    pub(crate) fn sc_table_corrupt(reason: ErrorStr) -> Self {
        Self {
            error_code: ErrorCode::ScTableCorruption,
            reason,
            fatal: false
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        unimplemented!()
    }
}

impl std::error::Error for Error {}