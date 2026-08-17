//! # sasspile — A Rust-native Sass/SCSS compiler
//!
//! Built from the official Sass specification and sass-spec test suite.
//! Does not reference dart-sass or any other Sass implementation.
//!
//! ## Overview
//!
//! sasspile is a from-scratch Sass/SCSS compiler written in pure Rust. It implements
//! the full compilation pipeline — lexer, parser, evaluator, and serializer — and
//! supports the majority of Sass language features.
//!
//! ## Quick Start
//!
//! ```rust
//! use sasspile::compile;
//!
//! let scss = r#"
//!     $primary: #336699;
//!
//!     .button {
//!         color: $primary;
//!         &:hover { color: lighten($primary, 10%); }
//!     }
//! "#;
//!
//! let css = compile(scss).unwrap();
//! ```
//!
//! ## File-based compilation
//!
//! For `@use "module"` resolution from the filesystem:
//!
//! ```rust,no_run
//! use sasspile::compile_file;
//!
//! let css = compile_file("src/main.scss").unwrap();
//! ```
//!
//! ## Pipeline
//!
//! The compilation pipeline follows four stages:
//!
//! 1. **Lexer** — Tokenizes SCSS source into a `Token` stream
//! 2. **Parser** — Builds an `AST` from tokens
//! 3. **Evaluator** — Evaluates the `AST` into a `CssTree`
//! 4. **Serializer** — Converts the `CssTree` into a CSS string
//!
//! Each stage is instrumented with `tracing` spans for observability.

pub mod error;
pub mod token;
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod env;
pub mod eval;
pub mod serialize;
pub mod selector;
pub mod value;
pub mod operators;
pub mod builtins;
pub mod raw_css;

use tracing::instrument;

pub use error::SassError;
pub use token::{Token, TokenSpan};
pub use lexer::tokenize;
pub use ast::{Stmt, Expr};
pub use parser::parse;
pub use eval::evaluate;
pub use eval::evaluate_with_dir;
pub use serialize::{serialize, serialize_with_style, OutputStyle};

/// Compile a SCSS source string to CSS.
///
/// This is the main entry point for the compiler.
/// It orchestrates the full pipeline: Lexer → Parser → Evaluator → Serializer.
#[instrument(name = "compile", skip_all, fields(stage = "compile"))]
pub fn compile(source: &str) -> Result<String, SassError> {
    let span = tracing::info_span!("compile_pipeline", stage = "compile");
    let _enter = span.enter();

    let tokens = tokenize(source)?;
    let ast = parse(tokens)?;
    let css_tree = evaluate(ast)?;
    let output = serialize(&css_tree)?;

    tracing::info!(stage = "compile", output_len = output.len(), "compilation complete");
    Ok(output)
}

/// Compile a SCSS file from the filesystem.
///
/// Sets `base_dir` so that `@use` / `@import` can resolve relative files.
/// This is the preferred entry point for compiling real projects.
#[instrument(name = "compile_file", skip_all, fields(stage = "compile"))]
pub fn compile_file(path: impl AsRef<std::path::Path>) -> Result<String, SassError> {
    let path = path.as_ref();
    let span = tracing::info_span!("compile_pipeline", stage = "compile", file = %path.display());
    let _enter = span.enter();

    let source = std::fs::read_to_string(path).map_err(|e| {
        SassError::parse(
            format!("Failed to read {}: {}", path.display(), e),
            crate::error::SourcePos { file: String::new(), line: 0, column: 0 },
        )
    })?;
    let base_dir = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();

    let tokens = tokenize(&source)?;
    let ast = parse(tokens)?;
    let css_tree = eval::evaluate_with_dir(ast, base_dir)?;
    let output = serialize(&css_tree)?;

    tracing::info!(stage = "compile", output_len = output.len(), "compilation complete");
    Ok(output)
}
