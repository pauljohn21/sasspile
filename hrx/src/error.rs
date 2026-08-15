//! Error types for HRX parsing and writing.

use thiserror::Error;

/// Errors that can occur when parsing or writing HRX archives.
#[derive(Debug, Error)]
pub enum HrxError {
    /// The input does not begin with a valid HRX boundary.
    #[error("invalid HRX format: expected '<==>:' or '<===>' as first non-comment line, got: {0:?}")]
    InvalidFormat(String),

    /// A line exceeds the maximum allowed length (1MB per the HRX spec).
    #[error("line too long: {0} bytes exceeds maximum of 1048576")]
    LineTooLong(usize),

    /// The boundary marker is malformed.
    #[error("invalid boundary marker: {0:?}")]
    InvalidBoundary(String),

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A path entry is invalid (e.g., empty, absolute, or contains `..`).
    #[error("invalid path: {0:?}")]
    InvalidPath(String),
}

/// Result type alias for HRX operations.
pub type Result<T> = std::result::Result<T, HrxError>;
