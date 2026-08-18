//! Token types produced by the lexer.

use crate::error::SourcePos;

/// Token types produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Identifier or keyword
    Ident(String),
    /// Variable reference ($name)
    Variable(String),
    /// Number with optional unit
    Number(f64, Option<String>),
    /// Hex color (#fff, #aabbcc)
    HexColor(u32),
    /// Quoted string
    String(String, char),
    /// Unquoted string / identifier
    UnquotedString(String),
    /// Interpolation start #{
    InterpolationStart,
    /// Interpolation end }
    InterpolationEnd,
    /// Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,        // ==
    SingleEq,  // = (for CSS attribute selectors like [data-bs-theme="..."])
    NotEq,     // !=
    Lt,        // <
    LtEq,      // <=
    Gt,        // >
    GtEq,      // >=
    /// Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,
    Ampersand, // & parent selector
    Hash,      // # (for placeholder/interpolation prefix)
    Dot,       // . (for class selectors)
    /// Spread/rest operator ... (for variadic args)
    Spread,
    /// At-rule keywords
    AtRule(String),
    /// Block comment /* */ (preserved)
    BlockComment(String),
    /// Line comment // (not in output)
    LineComment(String),
    /// End of file
    Eof,
}

/// Source span for a token.
#[derive(Debug, Clone)]
pub struct TokenSpan {
    pub token: Token,
    pub pos: SourcePos,
}
