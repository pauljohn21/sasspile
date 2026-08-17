//! Sass error types with source position tracking.

use std::fmt;

/// Error category for classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Lexer error: invalid token or character
    Lex,
    /// Parser error: unexpected token, syntax violation
    Parse,
    /// Evaluation error: undefined variable, type mismatch, etc.
    Eval,
    /// Type error: incompatible value types for an operation
    Type,
    /// User-triggered error via @error directive
    UserError,
    /// Import/load error: file not found, module resolution failure
    Import,
}

/// Source position in the original SCSS file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourcePos {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for SourcePos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// The main error type for all Sass compilation errors.
#[derive(Debug, Clone)]
pub struct SassError {
    pub message: String,
    pub pos: SourcePos,
    pub category: ErrorCategory,
}

impl SassError {
    pub fn new(message: impl Into<String>, pos: SourcePos, category: ErrorCategory) -> Self {
        Self {
            message: message.into(),
            pos,
            category,
        }
    }

    pub fn lex(message: impl Into<String>, pos: SourcePos) -> Self {
        Self::new(message, pos, ErrorCategory::Lex)
    }

    pub fn parse(message: impl Into<String>, pos: SourcePos) -> Self {
        Self::new(message, pos, ErrorCategory::Parse)
    }

    pub fn eval(message: impl Into<String>, pos: SourcePos) -> Self {
        Self::new(message, pos, ErrorCategory::Eval)
    }

    pub fn type_err(message: impl Into<String>, pos: SourcePos) -> Self {
        Self::new(message, pos, ErrorCategory::Type)
    }

    pub fn user_error(message: impl Into<String>, pos: SourcePos) -> Self {
        Self::new(message, pos, ErrorCategory::UserError)
    }

    pub fn import(message: impl Into<String>, pos: SourcePos) -> Self {
        Self::new(message, pos, ErrorCategory::Import)
    }
}

impl fmt::Display for SassError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {} at {}", self.message, self.pos)
    }
}

impl std::error::Error for SassError {}

impl From<std::fmt::Error> for SassError {
    fn from(e: fmt::Error) -> Self {
        SassError::eval(e.to_string(), SourcePos::default())
    }
}
