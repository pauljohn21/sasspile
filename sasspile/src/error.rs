//! Error types for sasspile.

use thiserror::Error;

/// Top-level error type for the sasspile compiler.
#[derive(Debug, Error)]
pub enum SassError {
    /// IO error reading source files.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic compilation failure.
    #[error("compile error: {0}")]
    Compile(String),

    /// Generic error with context.
    #[error("{context}: {source}")]
    WithContext {
        /// Human-readable context description.
        context: String,
        /// Underlying cause.
        #[source]
        source: Box<SassError>,
    },
}

impl SassError {
    /// Wrap this error with additional context.
    pub fn context(self, ctx: impl Into<String>) -> Self {
        Self::WithContext {
            context: ctx.into(),
            source: Box::new(self),
        }
    }
}
