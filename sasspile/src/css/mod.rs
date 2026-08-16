//! CSS generation — evaluated AST → CSS text.
//!
//! Converts the evaluated Sass AST to CSS output, supporting
//! expanded and compressed output styles.

pub mod ast;
pub mod atrules;
pub mod generator;
pub mod rules;

pub use ast::{CssAtRule, CssDeclaration, CssDocument, CssRule};
pub use generator::OutputStyle;

use crate::Result;

/// Generate CSS text from an evaluated stylesheet.
pub fn generate(
    stylesheet: &crate::parser::Stylesheet,
    style: OutputStyle,
) -> Result<String> {
    let mut doc = CssDocument::new();
    rules::expand_stylesheet(stylesheet, &mut doc);
    Ok(generator::format(&doc, style))
}
