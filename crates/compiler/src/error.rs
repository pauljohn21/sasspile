//! Error types for the reactive grass compiler.

use std::fmt;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create an error with a message.
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }

    /// Convenience alias matching `crate::Error::Msg(...)` usage.
    pub fn msg(msg: impl Into<String>) -> Self {
        Self::new(msg)
    }
}
