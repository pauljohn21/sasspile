//! Interpolation (`#{...}`) parsing.
//!
//! Handles `#{}` interpolation in selectors, property names, values,
//! and string contexts.

use crate::lexer::TokenKind;

use super::ast::Expr;

/// Detects if the current context is interpolation-active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationContext {
    /// Inside `#{...}` expression.
    Inside,
    /// Outside interpolation (normal token stream).
    Outside,
}

/// Extract interpolation expression from token stream.
///
/// Returns the expression extracted and the number of tokens consumed.
pub fn extract_interpolation(tokens: &[crate::lexer::Token]) -> Option<(Expr, usize)> {
    let mut depth = 0usize;
    let mut expr_tokens = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        match &token.kind {
            TokenKind::Interpolation => {
                depth += 1;
            }
            TokenKind::RBrace if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    if expr_tokens.is_empty() {
                        return Some((Expr::String(String::new()), i + 1));
                    }
                    return Some((Expr::String("TODO".to_string()), i + 1));
                }
            }
            TokenKind::RBrace => {
                return None;
            }
            _ => {
                expr_tokens.push(token.clone());
            }
        }
    }
    None
}

/// Check whether a token sequence starts with interpolation.
pub fn starts_with_interpolation(tokens: &[crate::lexer::Token]) -> bool {
    matches!(tokens.first(), Some(t) if matches!(t.kind, TokenKind::Interpolation))
}

/// Render interpolation prefix.
pub fn interpolation_prefix() -> &'static str {
    "#{"
}

/// Render interpolation suffix.
pub fn interpolation_suffix() -> &'static str {
    "}"
}
