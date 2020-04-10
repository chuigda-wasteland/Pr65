use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone)]
pub enum Error {
    ScTableCorrupt { reason: ErrorStr },
    ScSplitCorrupt { reason: ErrorStr },
    IOError { reason: ErrorStr, file: String },
    RequiresExplode
}

#[derive(Debug, Clone)]
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
        Error::ScTableCorrupt { reason }
    }

    pub(crate) fn sc_split_corrupt(reason: ErrorStr) -> Self {
        Error::ScSplitCorrupt { reason }
    }

    pub(crate) fn io_error(reason: ErrorStr, file: String) -> Self {
        Error::IOError { reason, file }
    }

    pub(crate) fn requires_explode() -> Self {
        Error::RequiresExplode
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        unimplemented!()
    }
}

impl std::error::Error for Error {}
