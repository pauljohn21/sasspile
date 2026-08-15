//! sasslipe — Pure Rust async SCSS compiler.
//!
//! Pipeline: Source → Lex → Parse → Semantic → Transform → Evaluate → Codegen
//!
//! # Architecture
//!
//! Each compilation stage is an independent Tokio task, connected via `mpsc` channels.
//! Immutable data flows through; `watch` channels propagate variable changes for
//! incremental recompilation.

pub mod error;
pub mod pipeline;

pub use error::SassError;
pub use pipeline::Compiler;

/// Library version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result type used throughout the crate.
pub type Result<T> = std::result::Result<T, SassError>;
