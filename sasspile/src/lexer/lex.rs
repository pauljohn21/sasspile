//! Main SCSS lexer implementation.

use tracing::instrument;

use crate::source::SourceSpan;
use crate::diagnostics::{Diagnostic, Diagnostics};

use super::TokenKind;
use super::token::Token;

/// SCSS tokenizer.
pub struct Lexer<'src> {
    /// Source text.
    src: &'src str,
    /// Current position (byte offset).
    pos: usize,
    /// Collected diagnostics.
    diagnostics: Diagnostics,
}

impl<'src> Lexer<'src> {
    /// Create a new lexer for the given source.
    pub fn new(src: &'src str) -> Self {
        Self {
            src,
            pos: 0,
            diagnostics: Diagnostics::new(),
        }
    }

    /// Tokenize the entire source into a Vec of tokens.
    #[instrument(skip(self))]
    pub fn tokenize(mut self) -> (Vec<Token>, Diagnostics) {
        let mut tokens = Vec::new();
        while self.pos < self.src.len() {
            match self.next_token() {
                Some(token) => tokens.push(token),
                None => continue, // skip comments and whitespace-less tokens
            }
        }
        // Append EOF token.
        let eof_span = SourceSpan::new(self.pos as u32, self.pos as u32);
        tokens.push(Token::new(TokenKind::Eof, eof_span));
        (tokens, self.diagnostics)
    }

    /// Advance to the next token.
    fn next_token(&mut self) -> Option<Token> {
        let start = self.pos;
        let ch = self.current_char()?;

        // Skip whitespace (but record as token for some contexts).
        if ch.is_ascii_whitespace() {
            return self.scan_whitespace(start);
        }

        match ch {
            '$' => self.scan_variable(start),
            '#' => self.scan_hash(start),
            '@' => self.scan_at_keyword(start),
            '.' => self.scan_dot(start),
            '&' => self.single_char(TokenKind::Ampersand, start),
            '*' => self.single_char(TokenKind::Star, start),
            '+' => self.single_char(TokenKind::Plus, start),
            ',' => self.single_char(TokenKind::Comma, start),
            ';' => self.single_char(TokenKind::Semicolon, start),
            '%' => self.single_char(TokenKind::Percent, start),
            '^' => self.single_char(TokenKind::Caret, start),
            '~' => self.single_char(TokenKind::Tilde, start),
            '|' => self.single_char(TokenKind::Pipe, start),
            '`' => self.single_char(TokenKind::Backtick, start),
            ':' => self.scan_colon(start),
            '(' => self.single_char(TokenKind::LParen, start),
            ')' => self.single_char(TokenKind::RParen, start),
            '{' => self.single_char(TokenKind::LBrace, start),
            '}' => self.single_char(TokenKind::RBrace, start),
            '[' => self.single_char(TokenKind::LBracket, start),
            ']' => self.single_char(TokenKind::RBracket, start),
            '/' => self.scan_slash(start),
            '=' => self.scan_eq(start),
            '!' => self.scan_not(start),
            '<' => self.scan_less(start),
            '>' => self.scan_greater(start),
            '-' => self.scan_minus(start),
            '\\' => {
                // Backslash: skip it (CSS escape character)
                self.advance();
                None
            }
            '\'' | '"' => self.scan_string(start),
            c if c.is_ascii_digit() => self.scan_number(start),
            c if is_ident_start(c) => self.scan_ident(start),
            c => {
                self.diagnostics.push(
                    Diagnostic::error("L001", format!("unexpected character '{c}'"))
                        .with_span(SourceSpan::new(start as u32, self.pos as u32)),
                );
                self.advance();
                None
            }
        }
    }

    // ──────────────────── Helper methods ──────────────────────────

    /// Get current character without advancing.
    fn current_char(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    /// Advance one character, returning it.
    fn advance(&mut self) -> Option<char> {
        let ch = self.current_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    /// Scan a single-character token.
    fn single_char(&mut self, kind: TokenKind, start: usize) -> Option<Token> {
        self.advance();
        Some(Token::new(kind, SourceSpan::new(start as u32, self.pos as u32)))
    }

    /// Scan a token starting with `$`.
    fn scan_variable(&mut self, start: usize) -> Option<Token> {
        self.advance(); // consume '$'
        let name = self.scan_ident_body();
        Some(Token::new(
            TokenKind::Variable(name),
            SourceSpan::new(start as u32, self.pos as u32),
        ))
    }

    /// Scan a token starting with `#`.
    fn scan_hash(&mut self, start: usize) -> Option<Token> {
        self.advance(); // consume '#'
        // Check for interpolation `#{
        if self.current_char() == Some('{') {
            self.advance();
            return Some(Token::new(
                TokenKind::Interpolation,
                SourceSpan::new(start as u32, self.pos as u32),
            ));
        }
        // Try hex color if followed by hex digits
        if self.current_char().is_some_and(|c| c.is_ascii_hexdigit()) {
            let color_start = self.pos;
            while self.current_char().is_some_and(|c| c.is_ascii_hexdigit()) {
                self.advance();
            }
            let hex_str = &self.src[color_start..self.pos];
            if let Ok(value) = u32::from_str_radix(hex_str, 16) {
                return Some(Token::new(
                    TokenKind::Color(value),
                    SourceSpan::new(start as u32, self.pos as u32),
                ));
            }
        }
        Some(Token::new(
            TokenKind::Hash,
            SourceSpan::new(start as u32, self.pos as u32),
        ))
    }

    /// Scan an at-keyword (`@use`, `@mixin`, etc.).
    fn scan_at_keyword(&mut self, start: usize) -> Option<Token> {
        self.advance(); // consume '@'
        let name = self.scan_ident_body();
        Some(Token::new(
            TokenKind::AtKeyword(name),
            SourceSpan::new(start as u32, self.pos as u32),
        ))
    }

    /// Scan identifier body starting from current position.
    fn scan_ident_body(&mut self) -> String {
        let mut result = String::new();
        while let Some(c) = self.current_char() {
            if is_ident_char(c) {
                result.push(c);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    /// Scan identifier or at-keyword body.
    #[instrument(skip(self), fields(ident = tracing::field::Empty, is_url = tracing::field::Empty))]
    fn scan_ident(&mut self, start: usize) -> Option<Token> {
        let name = self.scan_ident_body();
        let span = tracing::Span::current();
        span.record("ident", name.clone());
        // Check for url(...) — 必须整体 token 化，否则内部的 : / 会破坏解析
        if name == "url" && self.src[self.pos..].starts_with('(') {
            span.record("is_url", true);
            return self.scan_url(start);
        }
        span.record("is_url", false);
        // Check for keywords
        let kind = match name.as_str() {
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            _ => TokenKind::Ident(name),
        };
        Some(Token::new(kind, SourceSpan::new(start as u32, self.pos as u32)))
    }

    /// Scan `url(...)` as a single Url token, consuming until matching `)`.
    fn scan_url(&mut self, start: usize) -> Option<Token> {
        self.advance(); // consume '('
        let content_start = self.pos;
        let mut depth = 1i32;
        while self.pos < self.src.len() && depth > 0 {
            let c = self.current_char().unwrap();
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                '\'' | '"' => {
                    // skip string inside url(...)
                    self.advance();
                    let quote = c;
                    while let Some(ch) = self.current_char() {
                        if ch == quote { break; }
                        self.advance();
                    }
                }
                _ => {}
            }
            if depth > 0 {
                self.advance();
            }
        }
        let content = &self.src[content_start..self.pos].trim();
        self.advance(); // consume ')'
        Some(Token::new(
            TokenKind::Url(content.to_string()),
            SourceSpan::new(start as u32, self.pos as u32),
        ))
    }

    /// Scan a number (integer or float, with optional unit).
    fn scan_number(&mut self, start: usize) -> Option<Token> {
        let mut has_dot = false;
        while let Some(c) = self.current_char() {
            if c.is_ascii_digit() {
                self.advance();
            } else if c == '.' && !has_dot {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }
        let num_str = &self.src[start..self.pos];
        let value: f64 = num_str.parse().unwrap_or(0.0);

        // Check for unit (identifier or %)
        let mut unit = None;
        if self.current_char() == Some('%') {
            self.advance();
            unit = Some("%".to_string());
        } else if self.current_char().is_some_and(is_ident_start) {
            let unit_start = self.pos;
            while self.current_char().is_some_and(is_ident_char) {
                self.advance();
            }
            unit = Some(self.src[unit_start..self.pos].to_string());
        }

        Some(Token::new(
            TokenKind::Number(value, unit),
            SourceSpan::new(start as u32, self.pos as u32),
        ))
    }

    /// Scan a string (single or double quoted).
    fn scan_string(&mut self, start: usize) -> Option<Token> {
        let quote = self.advance()?; // consume opening quote
        let mut content = String::new();
        while let Some(c) = self.current_char() {
            if c == quote {
                self.advance(); // consume closing quote
                break;
            }
            if c == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char() {
                    content.push(escaped);
                    self.advance();
                }
            } else {
                content.push(c);
                self.advance();
            }
        }
        Some(Token::new(
            TokenKind::String(content),
            SourceSpan::new(start as u32, self.pos as u32),
        ))
    }

    /// Scan colon (single or combined).
    fn scan_colon(&mut self, start: usize) -> Option<Token> {
        self.advance();
        Some(Token::new(
            TokenKind::Colon,
            SourceSpan::new(start as u32, self.pos as u32),
        ))
    }

    /// Scan the `/` character (slash operator or comment).
    fn scan_slash(&mut self, start: usize) -> Option<Token> {
        self.advance();
        if self.current_char() == Some('/') {
            // Line comment: skip until newline
            while let Some(c) = self.current_char() {
                if c == '\n' {
                    break;
                }
                self.advance();
            }
            // Return None to skip line comments entirely
            return None;
        }
        if self.current_char() == Some('*') {
            // Block comment: skip until */
            self.advance(); // consume *
            while let Some(c) = self.current_char() {
                if c == '*' {
                    self.advance();
                    if self.current_char() == Some('/') {
                        self.advance();
                        break;
                    }
                } else {
                    self.advance();
                }
            }
            // Return None to skip block comments entirely
            return None;
        }
        Some(Token::new(
            TokenKind::Slash,
            SourceSpan::new(start as u32, self.pos as u32),
        ))
    }

    /// Scan `=` or `==`.
    fn scan_eq(&mut self, start: usize) -> Option<Token> {
        self.advance();
        if self.current_char() == Some('=') {
            self.advance();
            return Some(Token::new(
                TokenKind::Eq,
                SourceSpan::new(start as u32, self.pos as u32),
            ));
        }
        None // bare '=' is invalid in Sass
    }

    /// Scan `!` (handles `!=`).
    fn scan_not(&mut self, start: usize) -> Option<Token> {
        self.advance();
        if self.current_char() == Some('=') {
            self.advance();
            return Some(Token::new(
                TokenKind::NotEq,
                SourceSpan::new(start as u32, self.pos as u32),
            ));
        }
        Some(Token::new(
            TokenKind::Not,
            SourceSpan::new(start as u32, self.pos as u32),
        ))
    }

    /// Scan `<` or `<=`.
    fn scan_less(&mut self, start: usize) -> Option<Token> {
        self.advance();
        if self.current_char() == Some('=') {
            self.advance();
            return Some(Token::new(
                TokenKind::LessEq,
                SourceSpan::new(start as u32, self.pos as u32),
            ));
        }
        Some(Token::new(
            TokenKind::Less,
            SourceSpan::new(start as u32, self.pos as u32),
        ))
    }

    /// Scan `>` or `>=`.
    fn scan_greater(&mut self, start: usize) -> Option<Token> {
        self.advance();
        if self.current_char() == Some('=') {
            self.advance();
            return Some(Token::new(
                TokenKind::GreaterEq,
                SourceSpan::new(start as u32, self.pos as u32),
            ));
        }
        Some(Token::new(
            TokenKind::Greater,
            SourceSpan::new(start as u32, self.pos as u32),
        ))
    }

    /// Scan `-` (may start a number or identifier).
    fn scan_minus(&mut self, start: usize) -> Option<Token> {
        self.advance();
        // Check for negative number
        if self.current_char().is_some_and(|c| c.is_ascii_digit()) {
            return self.scan_number(start);
        }
        Some(Token::new(
            TokenKind::Minus,
            SourceSpan::new(start as u32, self.pos as u32),
        ))
    }

    /// Scan `.` (may start a number like `.5` or the dot operator).
    fn scan_dot(&mut self, start: usize) -> Option<Token> {
        // Check for `...`
        if self.src[self.pos..].starts_with("...") {
            self.pos += 3;
            return Some(Token::new(
                TokenKind::DotDotDot,
                SourceSpan::new(start as u32, self.pos as u32),
            ));
        }
        self.advance();
        // Check for number starting with dot (e.g., .5px)
        if self.current_char().is_some_and(|c| c.is_ascii_digit()) {
            return self.scan_number(start);
        }
        Some(Token::new(
            TokenKind::Dot,
            SourceSpan::new(start as u32, self.pos as u32),
        ))
    }

    /// Scan whitespace.
    fn scan_whitespace(&mut self, start: usize) -> Option<Token> {
        while self.current_char().is_some_and(|c| c.is_ascii_whitespace()) {
            self.advance();
        }
        Some(Token::new(
            TokenKind::Whitespace,
            SourceSpan::new(start as u32, self.pos as u32),
        ))
    }
}

/// Check if character can start an identifier.
fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || (c as u32) > 127
}

/// Check if character can appear in an identifier body.
fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || (c as u32) > 127
}
