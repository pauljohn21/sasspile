//! sasspile — A Rust-native Sass compiler.
//!
//! Built from the official Sass specification and sass-spec test suite.
//! Does not reference dart-sass or any other Sass implementation.

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

/// Compile a SCSS source string with a virtual file system.
///
/// The VFS maps module names (without extension) to file content.
/// This enables `@use "plain"` to resolve to a virtual `plain.css` file.
#[instrument(name = "compile_with_files", skip_all, fields(stage = "compile"))]
pub fn compile_with_files(
    source: &str,
    vfs: &std::collections::HashMap<String, String>,
) -> Result<String, SassError> {
    let span = tracing::info_span!("compile_pipeline", stage = "compile");
    let _enter = span.enter();

    let tokens = tokenize(source)?;
    let ast = parse(tokens)?;
    let css_tree = eval::evaluate_with_vfs(ast, vfs)?;
    let output = serialize(&css_tree)?;

    tracing::info!(stage = "compile", output_len = output.len(), "compilation complete");
    Ok(output)
}
