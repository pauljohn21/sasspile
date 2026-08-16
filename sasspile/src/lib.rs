//! sasspile — Pure Rust async SCSS compiler.
//!
//! Pipeline: Source → Lex → Parse → Semantic → Transform → Evaluate → Codegen
//!
//! # Architecture
//!
//! Each compilation stage is an independent Tokio task, connected via `mpsc` channels.
//! Immutable data flows through; `watch` channels propagate variable changes for
//! incremental recompilation.

pub mod builtin;
pub mod color;
pub mod css;
pub mod diagnostics;
pub mod error;
pub mod eval;
pub mod incremental;
pub mod lexer;
pub mod parser;
pub mod pipeline;
pub mod semantic;
pub mod source;
pub mod value;

pub use error::SassError;
pub use eval::{EvalContext, EvalError};
pub use pipeline::Compiler;
pub use pipeline::{Pipeline, PipelineInput, PipelineOutput};
pub use parser::*;
pub use diagnostics::Diagnostics;
pub use lexer::{tokenize, Token, TokenKind};
pub use semantic::{DefinitionRegistry, SymbolTable};
pub use value::{Separator, Value};

/// Library version string (from `CARGO_PKG_VERSION`).
///
/// # Examples
///
/// ```
/// use sasspile::VERSION;
///
/// assert!(!VERSION.is_empty());
/// println!("sasspile version: {VERSION}");
/// ```
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result type used throughout the crate.
pub type Result<T> = std::result::Result<T, SassError>;
