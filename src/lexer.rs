//! Lexer — tokenizes SCSS source into a token stream.

use crate::error::{SassError, SourcePos};
use crate::token::{Token, TokenSpan};
use tracing::instrument;

/// Tokenize SCSS source text.
///
/// `file` is the source file path used in error messages and trace spans.
/// Pass `"<string>"` for inline source without a file.
///
/// This is a pure function — no state, no side effects.
#[instrument(name = "tokenize", skip_all, fields(stage = "lexer", file = %file))]
pub fn tokenize(source: &str, file: &str) -> Result<Vec<TokenSpan>, SassError> {
    let span = tracing::info_span!("tokenize", stage = "lexer", file = %file, len = source.len());
    let _enter = span.enter();

    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    let mut line = 1usize;
    let mut col = 1usize;

    while let Some(&ch) = chars.peek() {
        let pos = SourcePos {
            file: file.to_string(),
            line,
            column: col,
        };

        // Skip whitespace
        if ch.is_whitespace() {
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            chars.next();
            continue;
        }

        // Line comment
        if ch == '/' && chars.clone().nth(1) == Some('/') {
            let comment = read_line_comment(&mut chars, &mut line, &mut col);
            tokens.push(TokenSpan { token: Token::LineComment(comment), pos });
            continue;
        }

        // Block comment
        if ch == '/' && chars.clone().nth(1) == Some('*') {
            let comment = read_block_comment(&mut chars, &mut line, &mut col);
            tokens.push(TokenSpan { token: Token::BlockComment(comment), pos });
            continue;
        }

        // Variable
        if ch == '$' {
            chars.next();
            col += 1;
            let name = read_ident_chars(&mut chars, &mut col);
            tokens.push(TokenSpan { token: Token::Variable(name), pos });
            continue;
        }

        // At-rule
        if ch == '@' {
            chars.next();
            col += 1;
            let name = read_atrule_chars(&mut chars, &mut col);
            tokens.push(TokenSpan { token: Token::AtRule(name), pos });
            continue;
        }

        // Interpolation start #{  (must check before hex color)
        if ch == '#' && chars.clone().nth(1) == Some('{') {
            chars.next();
            chars.next();
            col += 2;
            tokens.push(TokenSpan { token: Token::InterpolationStart, pos });
            continue;
        }

        // Hex color
        if ch == '#' && chars.clone().nth(1).map_or(false, |c| c.is_ascii_hexdigit()) {
            chars.next();
            col += 1;
            let (value, _) = read_hex(&mut chars, &mut col);
            tokens.push(TokenSpan { token: Token::HexColor(value), pos });
            continue;
        }

        // Number
        if ch.is_ascii_digit()
            || (ch == '.' && chars.clone().nth(1).map_or(false, |c| c.is_ascii_digit()))
        {
            let (value, unit) = read_number(&mut chars, &mut col);
            tokens.push(TokenSpan { token: Token::Number(value, unit), pos });
            continue;
        }

        // String (quoted)
        if ch == '"' || ch == '\'' {
            let quote = ch;
            chars.next();
            col += 1;
            let s = read_quoted_string(&mut chars, &mut line, &mut col, quote);
            tokens.push(TokenSpan { token: Token::String(s, quote), pos });
            continue;
        }

        // Identifier (including unquoted strings)
        // `-` starts an ident only if followed by a letter or `-` (e.g. `-webkit-` but not `1 - 2`)
        if ch.is_alphabetic() || ch == '_'
            || (ch == '-' && chars.clone().nth(1).map_or(false, |c| c.is_alphabetic() || c == '_' || c == '-'))
        {
            let name = read_ident(&mut chars, &mut col);
            tokens.push(TokenSpan { token: Token::Ident(name), pos });
            continue;
        }

        // Single-char and multi-char tokens
        let token = match ch {
            '+' => { chars.next(); col += 1; Token::Plus }
            '*' => { chars.next(); col += 1; Token::Star }
            '/' => { chars.next(); col += 1; Token::Slash }
            '%' => { chars.next(); col += 1; Token::Percent }
            '(' => { chars.next(); col += 1; Token::LParen }
            ')' => { chars.next(); col += 1; Token::RParen }
            '{' => { chars.next(); col += 1; Token::LBrace }
            '}' => { chars.next(); col += 1; Token::RBrace }
            '[' => { chars.next(); col += 1; Token::LBracket }
            ']' => { chars.next(); col += 1; Token::RBracket }
            ',' => { chars.next(); col += 1; Token::Comma }
            ':' => { chars.next(); col += 1; Token::Colon }
            ';' => { chars.next(); col += 1; Token::Semicolon }
            '&' => { chars.next(); col += 1; Token::Ampersand }
            '#' => { chars.next(); col += 1; Token::Hash }
            '.' => {
    // Check for spread operator ...
    if chars.clone().nth(1) == Some('.') && chars.clone().nth(2) == Some('.') {
        chars.next(); chars.next(); chars.next();
        col += 3;
        Token::Spread
    } else {
        chars.next(); col += 1; Token::Dot
    }
}
            '-' => { chars.next(); col += 1; Token::Minus }
            '~' => { chars.next(); col += 1; Token::Ident("~".to_string()) }
            '^' => { chars.next(); col += 1; Token::Ident("^".to_string()) }
            '<' => {
                chars.next(); col += 1;
                if chars.peek() == Some(&'=') { chars.next(); col += 1; Token::LtEq }
                else { Token::Lt }
            }
            '>' => {
                chars.next(); col += 1;
                if chars.peek() == Some(&'=') { chars.next(); col += 1; Token::GtEq }
                else { Token::Gt }
            }
            '=' => {
                chars.next(); col += 1;
                if chars.peek() == Some(&'=') { chars.next(); col += 1; Token::Eq }
                else { Token::SingleEq }
            }
            '!' => {
                chars.next(); col += 1;
                if chars.peek() == Some(&'=') { chars.next(); col += 1; Token::NotEq }
                else {
                    let name = read_bang_flag(&mut chars, &mut col);
                    Token::Ident(name)
                }
            }
            _ => return Err(SassError::lex(format!("unexpected character '{}'", ch), pos)),
        };

        tokens.push(TokenSpan { token, pos });
    }

    tokens.push(TokenSpan {
        token: Token::Eof,
        pos: SourcePos { file: file.to_string(), line, column: col },
    });

    tracing::debug!(stage = "lexer", token_count = tokens.len(), "tokenization complete");
    Ok(tokens)
}

fn read_line_comment(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    _line: &mut usize,
    col: &mut usize,
) -> String {
    let mut comment = String::new();
    while let Some(&c) = chars.peek() {
        if c == '\n' { break; }
        comment.push(c);
        chars.next();
        *col += 1;
    }
    comment
}

fn read_block_comment(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    line: &mut usize,
    col: &mut usize,
) -> String {
    let mut comment = String::new();
    chars.next(); // /
    chars.next(); // *
    *col += 2;
    while let Some(c) = chars.next() {
        if c == '*' && chars.peek() == Some(&'/') {
            chars.next();
            *col += 2;
            break;
        }
        if c == '\n' { *line += 1; *col = 1; }
        else { *col += 1; }
        comment.push(c);
    }
    comment
}

fn read_ident_chars(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    col: &mut usize,
) -> String {
    let mut name = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            name.push(c);
            chars.next();
            *col += 1;
        } else { break; }
    }
    name
}

fn read_atrule_chars(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    col: &mut usize,
) -> String {
    let mut name = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphabetic() || c == '-' || c == '_' {
            name.push(c);
            chars.next();
            *col += 1;
        } else { break; }
    }
    name
}

fn read_hex(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    col: &mut usize,
) -> (u32, String) {
    let mut hex = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_hexdigit() {
            hex.push(c);
            chars.next();
            *col += 1;
        } else { break; }
    }
    let value = u32::from_str_radix(&hex, 16).unwrap_or(0);
    (value, hex)
}

fn read_number(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    col: &mut usize,
) -> (f64, Option<String>) {
    let mut num_str = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() || c == '.' {
            num_str.push(c);
            chars.next();
            *col += 1;
        } else { break; }
    }
    let value: f64 = num_str.parse().unwrap_or(0.0);
    let mut unit = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '%' {
            unit.push(c);
            chars.next();
            *col += 1;
        } else { break; }
    }
    let unit = if unit.is_empty() { None } else { Some(unit) };
    (value, unit)
}

fn read_quoted_string(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    line: &mut usize,
    col: &mut usize,
    quote: char,
) -> String {
    let mut s = String::new();
    while let Some(&c) = chars.peek() {
        if c == quote {
            chars.next();
            *col += 1;
            break;
        }
        if c == '\\' {
            chars.next();
            *col += 1;
            if let Some(&next) = chars.peek() {
                s.push(next);
                chars.next();
                *col += 1;
            }
            continue;
        }
        if c == '\n' { *line += 1; *col = 1; }
        else { *col += 1; }
        s.push(c);
        chars.next();
    }
    s
}

fn read_ident(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    col: &mut usize,
) -> String {
    let mut name = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            name.push(c);
            chars.next();
            *col += 1;
        } else { break; }
    }
    name
}

fn read_bang_flag(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    col: &mut usize,
) -> String {
    let mut name = String::from('!');
    while let Some(&c) = chars.peek() {
        if c.is_alphabetic() {
            name.push(c);
            chars.next();
            *col += 1;
        } else { break; }
    }
    name
}
