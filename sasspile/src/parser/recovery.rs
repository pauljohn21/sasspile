//! Error recovery strategies for the parser.
//!
//! When encountering invalid syntax, attempt to synchronize at known
//! continuation points to report multiple errors in a single pass.

use crate::lexer::TokenKind;

/// Synchronization points in SCSS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncPoint {
    /// Next `;` — end of statement.
    Semicolon,
    /// Next `}` — end of block.
    BlockEnd,
    /// Next `{` — start of block.
    BlockStart,
    /// End of file.
    Eof,
}

/// Find the next synchronization token to resume parsing.
pub fn find_sync_point(tokens: &[crate::lexer::Token], start: usize, point: SyncPoint) -> usize {
    let target = match point {
        SyncPoint::Semicolon => TokenKind::Semicolon,
        SyncPoint::BlockEnd => TokenKind::RBrace,
        SyncPoint::BlockStart => TokenKind::LBrace,
        SyncPoint::Eof => TokenKind::Eof,
    };

    for (i, token) in tokens.iter().enumerate().skip(start) {
        if std::mem::discriminant(&token.kind) == std::mem::discriminant(&target) {
            return i;
        }
    }
    // Fallback: end of tokens
    tokens.len().saturating_sub(1)
}

/// Tune recovery strategy based on error type.
pub fn recovery_strategy(error_kind: &str) -> SyncPoint {
    match error_kind {
        "missing_semicolon" => SyncPoint::Semicolon,
        "missing_brace" => SyncPoint::BlockEnd,
        "invalid_value" => SyncPoint::Semicolon,
        _ => SyncPoint::BlockEnd,
    }
}
