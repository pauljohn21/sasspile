//! Source location tracking.
//!
//! Provides `SourceSpan` and `SourcePosition` for error reporting
//! and source map generation.

mod position;
mod span;

pub use position::SourcePosition;
pub use span::SourceSpan;
