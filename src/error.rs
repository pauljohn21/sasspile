//! Unified error type using thiserror.
//!
//! Errors carry source position info (Span) for precise location.

use thiserror::Error;

/// Source span.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Span {
    /// Start byte offset.
    pub start: usize,
    /// End byte offset.
    pub end: usize,
}

impl Span {
    /// Create a new Span.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Single-position Span.
    pub fn at(pos: usize) -> Self {
        Self {
            start: pos,
            end: pos + 1,
        }
    }
}

/// sasspile error type.
#[derive(Debug, Error)]
pub enum SassError {
    /// Lex error — invalid character during scanning.
    #[error("Lex error: {message} (pos {pos})")]
    Lex {
        /// Error description.
        message: String,
        /// Byte offset where the error occurred.
        pos: usize,
    },

    /// Parse error — structure mismatch during parsing.
    #[error("Parse error: expected {expected}, found {found}")]
    Parse {
        /// Expected token or structure description.
        expected: String,
        /// Actual token or structure encountered.
        found: String,
    },

    /// Evaluation error — runtime issue.
    #[error("{0}")]
    Eval(String),

    /// Type error — type mismatch.
    #[error("Type error: expected {expected}, got {actual}")]
    Type {
        /// Expected type description.
        expected: String,
        /// Actual type description.
        actual: String,
    },

    /// Unit error — incompatible unit operation.
    #[error("Unit error: {0}")]
    Unit(String),

    /// Undefined variable.
    #[error("Undefined variable: ${0}")]
    UndefinedVariable(String),

    /// Undefined mixin.
    #[error("Undefined mixin: {0}")]
    UndefinedMixin(String),

    /// Undefined function.
    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    /// Division by zero.
    #[error("Division by zero")]
    DivideByZero,

    /// Module loading error.
    #[error("Module error: {0}")]
    Module(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias.
pub type Result<T> = std::result::Result<T, SassError>;
