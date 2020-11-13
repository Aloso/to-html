use std::num::ParseIntError;

/// Errors that can occur when converting an ANSI string to HTML
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Parsing a number was unsuccessful
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),

    /// The ANSI escape code is invalid
    #[error("Invalid ANSI: {msg}")]
    InvalidAnsi { msg: String },
}

impl Error {
    pub(crate) fn invalid_ansi(s: &'static str) -> impl Fn() -> Self {
        move || Error::InvalidAnsi { msg: s.to_string() }
    }
}
