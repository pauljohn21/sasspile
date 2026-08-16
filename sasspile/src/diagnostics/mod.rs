//! Compilation diagnostics with source location tracking.
//!
//! Provides structured error reporting with source spans,
//! multiple severity levels, and best-effort error recovery.

mod diagnostic;
mod level;

pub use diagnostic::{Diagnostic, Diagnostics};
pub use level::Level;
