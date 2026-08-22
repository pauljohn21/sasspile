//! 错误类型——全管线统一。

use std::fmt;

/// SCSS 编译错误。
#[derive(Debug)]
pub struct SassError {
    pub message: String,
    pub kind: ErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Parse,
    Eval,
    Module,
    Io,
    PlainCss,
}

pub type Result<T> = std::result::Result<T, SassError>;

impl fmt::Display for SassError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::Parse => write!(f, "Error: {}", self.message),
            ErrorKind::Eval => write!(f, "Error: {}", self.message),
            ErrorKind::Module => write!(f, "Error: {}", self.message),
            ErrorKind::Io => write!(f, "Error: {}", self.message),
            ErrorKind::PlainCss => write!(f, "Error: {}", self.message),
        }
    }
}

impl std::error::Error for SassError {}

impl From<std::io::Error> for SassError {
    fn from(e: std::io::Error) -> Self {
        Self { message: e.to_string(), kind: ErrorKind::Io }
    }
}

impl SassError {
    pub fn parse(msg: impl Into<String>) -> Self {
        Self { message: msg.into(), kind: ErrorKind::Parse }
    }
    pub fn eval(msg: impl Into<String>) -> Self {
        Self { message: msg.into(), kind: ErrorKind::Eval }
    }
    pub fn module(msg: impl Into<String>) -> Self {
        Self { message: msg.into(), kind: ErrorKind::Module }
    }
    pub fn plain_css(msg: impl Into<String>) -> Self {
        Self { message: msg.into(), kind: ErrorKind::PlainCss }
    }
}
