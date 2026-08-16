//! Token types for SCSS/Sass lexing.

use std::fmt;

use crate::source::SourceSpan;

/// Token with source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Token kind (type + value).
    pub kind: TokenKind,
    /// Source span (byte range).
    pub span: SourceSpan,
}

impl Token {
    /// Create a new token.
    pub fn new(kind: TokenKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

/// Token kind enumeration.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ── Literals ─────────────────────────────────────────
    /// Identifier (e.g., `color`, `flex-direction`).
    Ident(String),
    /// Number with optional unit (e.g., `16px`, `1.5`).
    Number(f64, Option<String>),
    /// String literal (content without quotes).
    String(String),
    /// URL token.
    Url(String),
    /// Hex color (e.g., `#ff0000`).
    Color(u32),

    // ── Operators ────────────────────────────────────────
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `^` (attribute selector prefix)
    Caret,
    /// `~` (general sibling or attribute selector)
    Tilde,
    /// `|` (namespace separator)
    Pipe,
    /// `` ` `` (backtick, used in some CSS contexts)
    Backtick,
    /// `==`
    Eq,
    /// `!=`
    NotEq,
    /// `>`
    Greater,
    /// `<`
    Less,
    /// `>=`
    GreaterEq,
    /// `<=`
    LessEq,
    /// `and`
    And,
    /// `or`
    Or,
    /// `not`
    Not,

    // ── Delimiters ───────────────────────────────────────
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `;`
    Semicolon,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `...`
    DotDotDot,

    // ── Special ──────────────────────────────────────────
    /// `#{` (interpolation start).
    Interpolation,
    /// `@` keyword (e.g., `@use`, `@mixin`), carries keyword name.
    AtKeyword(String),
    /// `#` standalone (for ID selectors).
    Hash,
    /// `&` parent selector reference.
    Ampersand,
    /// `$` variable prefix marker.
    Dollar,
    /// `$variable` carries the variable name.
    Variable(String),

    // ── Sass-specific ────────────────────────────────────
    /// Indent in .sass files.
    Indent,
    /// Dedent in .sass files.
    Dedent,

    // ── Other ────────────────────────────────────────────
    /// Whitespace (may be significant in some contexts).
    Whitespace,
    /// End of file.
    Eof,
}
