//! Error types for sasspile.

use thiserror::Error;

/// Top-level error type for the sasspile compiler.
///
/// Errors are created via the [`Error`](std::error::Error) trait and can carry
/// additional context using the [`context`](SassError::context) method.
///
/// # Examples
///
/// ```
/// use sasspile::SassError;
///
/// // Create a compile error
/// let err = SassError::Compile("unexpected token".into());
///
/// // Add context
/// let wrapped = err.context("while parsing foo.scss");
/// println!("{wrapped}");
/// ```
///
/// ## Error variants
///
/// - [`SassError::Io`] — wraps `std::io::Error` for file read failures
/// - [`SassError::Compile`] — generic compilation failure with message
/// - [`SassError::WithContext`] — wraps another error with additional context
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
