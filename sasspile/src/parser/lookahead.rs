//! Token lookahead utility functions for the parser.
//!
//! These free functions accept a token slice and a position, performing
//! pure lookahead decisions without requiring a `&mut Parser` borrow.

use crate::lexer::{Token, TokenKind};

/// Returns `true` if the token stream at `pos` looks like a CSS declaration
/// (e.g. `color: red`, `--#{$name}: value`, `$var: 10px`).
///
/// Distinguishes declarations from selectors and nested rules by scanning
/// forward for a `Colon` terminator while skipping whitespace and interpolated
/// segments.
pub(crate) fn looks_like_declaration(tokens: &[Token], pos: usize) -> bool {
    let mut idx = pos;
    while idx < tokens.len() && matches!(tokens[idx].kind, TokenKind::Whitespace) {
        idx += 1;
    }
    match tokens.get(idx) {
        // CSS custom property: --#{$name}: red or --color: red
        Some(t) if matches!(t.kind, TokenKind::Minus) => {
            let mut j = idx + 1;
            // Consume -- and Interpolation/Ident segments
            while j < tokens.len() {
                match &tokens[j].kind {
                    TokenKind::Minus => j += 1,
                    TokenKind::Interpolation => {
                        j += 1;
                        // Consume interpolation inner tokens until RBrace
                        while j < tokens.len() && !matches!(tokens[j].kind, TokenKind::RBrace) {
                            j += 1;
                        }
                        if j < tokens.len() {
                            j += 1;
                        } // consume RBrace
                    }
                    TokenKind::Ident(_) => j += 1,
                    _ => break,
                }
            }
            // Skip whitespace
            while j < tokens.len() && matches!(tokens[j].kind, TokenKind::Whitespace) {
                j += 1;
            }
            matches!(tokens.get(j), Some(t) if matches!(t.kind, TokenKind::Colon))
        }
        Some(t) if matches!(t.kind, TokenKind::Ampersand | TokenKind::Dot | TokenKind::Hash) => {
            false
        }
        Some(t) if matches!(t.kind, TokenKind::Ident(_)) => {
            // Ident.Ident( pattern is a function call (e.g. map.merge(...), string.length())
            if matches!(tokens.get(idx + 1), Some(t) if matches!(t.kind, TokenKind::Dot))
                && matches!(tokens.get(idx + 2), Some(t) if matches!(t.kind, TokenKind::Ident(_)))
                && matches!(tokens.get(idx + 3), Some(t) if matches!(t.kind, TokenKind::LParen))
            {
                return false;
            }
            // Ident( pattern is also a function call
            if matches!(tokens.get(idx + 1), Some(t) if matches!(t.kind, TokenKind::LParen)) {
                return false;
            }
            let mut j = idx + 1;
            let mut paren_depth = 0;
            while j < tokens.len() {
                match &tokens[j].kind {
                    TokenKind::LParen => paren_depth += 1,
                    TokenKind::RParen => {
                        if paren_depth > 0 {
                            paren_depth -= 1;
                        }
                    }
                    TokenKind::Dot | TokenKind::Comma => {
                        return false;
                    } // Dot=element.class; Comma=selector list
                    TokenKind::Colon if paren_depth == 0 => return true,
                    TokenKind::LBrace | TokenKind::Semicolon | TokenKind::Eof => return false,
                    _ => {}
                }
                j += 1;
            }
            false
        }
        Some(t) if matches!(t.kind, TokenKind::Variable(_)) => {
            let mut j = idx + 1;
            while j < tokens.len() {
                match &tokens[j].kind {
                    TokenKind::Colon => return true,
                    TokenKind::LBrace | TokenKind::Semicolon | TokenKind::Eof => return false,
                    _ => j += 1,
                }
            }
            false
        }
        // Interpolation as property name: #{$var}: value
        Some(t) if matches!(t.kind, TokenKind::Interpolation) => {
            let mut j = idx + 1;
            // Skip contents of interpolation until RBrace
            while j < tokens.len() && !matches!(tokens[j].kind, TokenKind::RBrace) {
                j += 1;
            }
            if j < tokens.len() {
                j += 1;
            } // consume RBrace
            // Skip trailing parts (-ident, -#{...})
            while j < tokens.len() {
                match &tokens[j].kind {
                    TokenKind::Interpolation => {
                        j += 1;
                        while j < tokens.len() && !matches!(tokens[j].kind, TokenKind::RBrace) {
                            j += 1;
                        }
                        if j < tokens.len() {
                            j += 1;
                        }
                    }
                    TokenKind::Minus => {
                        let saved = j;
                        j += 1;
                        if matches!(
                            tokens.get(j).map(|t| &t.kind),
                            Some(TokenKind::Ident(_)) | Some(TokenKind::Interpolation)
                        ) {
                            continue;
                        }
                        j = saved;
                        break;
                    }
                    _ => break,
                }
            }
            // Skip whitespace
            while j < tokens.len() && matches!(tokens[j].kind, TokenKind::Whitespace) {
                j += 1;
            }
            // ::pseudo (::before, ::after) is part of a selector, not a declaration separator
            if matches!(tokens.get(j), Some(t) if matches!(t.kind, TokenKind::Colon))
                && matches!(tokens.get(j + 1), Some(t) if matches!(t.kind, TokenKind::Colon))
            {
                return false;
            }
            matches!(tokens.get(j), Some(t) if matches!(t.kind, TokenKind::Colon))
        }
        _ => false,
    }
}

/// Returns `true` if the token stream at `pos` is a top-level function call
/// (e.g. `map.merge(...)`, `calc(100% - 20px)`).
///
/// Top-level calls do not produce rule or declaration nodes; they are consumed
/// as no-ops during parsing.
pub(crate) fn is_top_level_expr_call(tokens: &[Token], pos: usize) -> bool {
    let mut idx = pos;
    while idx < tokens.len() && matches!(tokens[idx].kind, TokenKind::Whitespace) {
        idx += 1;
    }
    // Ident.Ident( or Ident( pattern
    let first_is_ident = matches!(
        tokens.get(idx),
        Some(t) if matches!(t.kind, TokenKind::Ident(_))
    );
    if !first_is_ident {
        return false;
    }
    // Skip trailing .Ident segments
    let mut j = idx + 1;
    while matches!(tokens.get(j), Some(t) if matches!(t.kind, TokenKind::Dot)) {
        j += 1;
        if matches!(tokens.get(j), Some(t) if matches!(t.kind, TokenKind::Ident(_))) {
            j += 1;
        } else {
            return false;
        }
    }
    // Final token must be (
    matches!(
        tokens.get(j),
        Some(t) if matches!(t.kind, TokenKind::LParen)
    )
}

/// Returns `true` if the token stream at `pos` looks like a list literal
/// (e.g. `a, b, c`).
///
/// Detects a comma after the first scalar token (ident, string, number, or
/// variable) to distinguish from parenthesized expressions.
pub(crate) fn is_list_syntax(tokens: &[Token], pos: usize) -> bool {
    let mut idx = pos;
    // Skip whitespace
    while idx < tokens.len() && matches!(tokens[idx].kind, TokenKind::Whitespace) {
        idx += 1;
    }
    // Simple heuristic: skip first token, look for comma
    if matches!(
        tokens.get(idx),
        Some(t) if matches!(
            t.kind,
            TokenKind::Ident(_)
                | TokenKind::String(_)
                | TokenKind::Number(_, _)
                | TokenKind::Variable(_)
        )
    ) {
        idx += 1;
        while idx < tokens.len() && matches!(tokens[idx].kind, TokenKind::Whitespace) {
            idx += 1;
        }
        return matches!(tokens.get(idx), Some(t) if matches!(t.kind, TokenKind::Comma));
    }
    false
}

/// Returns `true` if the token stream at `pos` looks like a map literal
/// (e.g. `key: value`).
///
/// Detects a `key: value` pattern where the key is a string, identifier, or
/// variable followed by a colon.
pub(crate) fn is_map_syntax(tokens: &[Token], pos: usize) -> bool {
    let mut idx = pos;
    // Skip whitespace
    while idx < tokens.len() && matches!(tokens[idx].kind, TokenKind::Whitespace) {
        idx += 1;
    }
    // Skip first key (string or ident)
    match tokens.get(idx) {
        Some(t) if matches!(t.kind, TokenKind::Ident(_) | TokenKind::String(_)) => idx += 1,
        Some(t) if matches!(t.kind, TokenKind::Variable(_)) => idx += 1,
        _ => return false,
    }
    // Skip whitespace
    while idx < tokens.len() && matches!(tokens[idx].kind, TokenKind::Whitespace) {
        idx += 1;
    }
    // Check for colon
    matches!(
        tokens.get(idx),
        Some(t) if matches!(t.kind, TokenKind::Colon)
    )
}
