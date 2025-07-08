use std::{fmt, num::ParseIntError};

/// Errors that can occur when converting an ANSI string to HTML
#[derive(Debug)]
pub enum Error {
    /// Parsing a number was unsuccessful
    ParseInt(ParseIntError),

    /// The ANSI escape code is invalid
    InvalidAnsi { msg: String },
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Self::ParseInt(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseInt(err) => write!(f, "{err}"),
            Self::InvalidAnsi { msg } => write!(f, "Invalid ANSI: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub(crate) fn invalid_ansi(s: &'static str) -> impl Fn() -> Self {
        move || Error::InvalidAnsi { msg: s.to_string() }
    }
}
