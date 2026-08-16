//! SCSS lexer — tokenizes source code into a stream of tokens.
//!
//! Supports both SCSS (braces/semicolons) and Sass (indented) syntax.

pub mod lex;
pub mod sass_syntax;
mod token;

pub use lex::Lexer;
pub use token::{Token, TokenKind};

use crate::diagnostics::Diagnostics;

/// Convenience function: tokenize source string.
pub fn tokenize(src: &str) -> (Vec<Token>, Diagnostics) {
    Lexer::new(src).tokenize()
}
