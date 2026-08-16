//! Parser — produces AST from token stream.
//!
//! Recursive descent parser supporting all SCSS/Sass constructs:
//! selectors, declarations, at-rules, expressions.

mod ast;
mod at_rules;
mod core;
mod expr;
mod selector;

mod lookahead;
pub mod interpolation;
pub mod recovery;

pub use ast::*;
pub use core::Parser;

use crate::diagnostics::Diagnostics;
use crate::lexer;

/// Convenience function: tokenize and parse source.
#[tracing::instrument(skip(source), fields(source_len = source.len()))]
pub fn parse(source: &str) -> (Stylesheet, Diagnostics) {
    let (tokens, _lex_diags) = lexer::tokenize(source);
    let parser = Parser::new(&tokens);
    let result = parser.parse();
    tracing::info!(nodes = result.0.nodes.len(), errors = result.1.errors().len(), "parse complete");
    result
}
