//! Error types for the reactive grass compiler.

use std::fmt;

/// Compiler error type — covers all stages of the pipeline.
#[derive(Debug)]
pub enum Error {
    /// Lexer error (invalid character, unterminated string, etc.)
    LexerError(String),
    /// Parser error (invalid syntax, unexpected token, etc.)
    ParserError(String),
    /// Lowering error (undefined variable, type mismatch, etc.)
    LoweringError(String),
    /// Evaluator error (runtime evaluation failure, @error directive, etc.)
    EvalError(String),
    /// Serializer error (invalid CSS output state, etc.)
    SerializerError(String),
    /// Generic message error (convenience variant)
    Msg(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LexerError(msg) => write!(f, "lexer error: {msg}"),
            Self::ParserError(msg) => write!(f, "parser error: {msg}"),
            Self::LoweringError(msg) => write!(f, "lowering error: {msg}"),
            Self::EvalError(msg) => write!(f, "eval error: {msg}"),
            Self::SerializerError(msg) => write!(f, "serializer error: {msg}"),
            Self::Msg(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {}

/// Convenience result type alias.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a generic message error.
    pub fn msg(msg: impl Into<String>) -> Self {
        Self::Msg(msg.into())
    }

    /// Create a lexer error.
    pub fn lexer(msg: impl Into<String>) -> Self {
        Self::LexerError(msg.into())
    }

    /// Create a parser error.
    pub fn parser(msg: impl Into<String>) -> Self {
        Self::ParserError(msg.into())
    }

    /// Create a lowering error.
    pub fn lowering(msg: impl Into<String>) -> Self {
        Self::LoweringError(msg.into())
    }

    /// Create an eval error.
    pub fn eval(msg: impl Into<String>) -> Self {
        Self::EvalError(msg.into())
    }

    /// Create a serializer error.
    pub fn serializer(msg: impl Into<String>) -> Self {
        Self::SerializerError(msg.into())
    }
}
