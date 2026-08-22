//! Scanner — 逐字符扫描，产出 Token。

use crate::error::{Result, SassError};
use super::token::{Token, QuoteStyle};

/// 字符扫描器。
pub struct Scanner<'a> {
    chars: Vec<char>,
    pos: usize,
    _input: &'a str,
}

impl<'a> Scanner<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { chars: input.chars().collect(), pos: 0, _input: input }
    }

    /// 当前字符，None 表示 EOF。
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// 向前看 N 个字符。
    fn peek_n(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
    }

    /// 消费当前字符并前进。
    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    /// 剩余字符数。
    fn remaining(&self) -> usize {
        self.chars.len().saturating_sub(self.pos)
    }

    /// 读取下一个 token。
    pub fn next_token(&mut self) -> Result<Option<Token>> {
        self.skip_whitespace_and_comments()?;

        if self.pos >= self.chars.len() {
            return Ok(None);
        }

        let c = self.peek().expect("checked above");

        let token = match c {
            '{' => { self.bump(); Token::LBrace }
            '}' => { self.bump(); Token::RBrace }
            '(' => { self.bump(); Token::LParen }
            ')' => { self.bump(); Token::RParen }
            '[' => { self.bump(); Token::LBracket }
            ']' => { self.bump(); Token::RBracket }
            ':' => { self.bump(); Token::Colon }
            ';' => { self.bump(); Token::Semicolon }
            ',' => { self.bump(); Token::Comma }
            '.' => { self.bump(); Token::Dot }
            '@' => { self.bump(); Token::At }
            '&' => { self.bump(); Token::Ampersand }
            '#' => self.scan_hash()?,
            '$' => self.scan_variable()?,
            '"' => self.scan_string('"')?,
            '\'' => self.scan_string('\'')?,
            '/' => self.scan_slash()?,
            '+' => { self.bump(); Token::Plus }
            '-' => self.scan_minus()?,
            '*' => { self.bump(); Token::Star }
            '%' => { self.bump(); Token::Percent }
            '=' => {
                self.bump();
                if self.peek() == Some('>') {
                    self.bump();
                    Token::Arrow
                } else {
                    Token::Eq
                }
            }
            '!' => {
                self.bump();
                if self.peek() == Some('=') {
                    self.bump();
                    Token::NotEq
                } else {
                    Token::Ident("!".to_string())
                }
            }
            '>' => {
                self.bump();
                if self.peek() == Some('=') {
                    self.bump();
                    Token::Gte
                } else {
                    Token::Gt
                }
            }
            '<' => {
                self.bump();
                if self.peek() == Some('=') {
                    self.bump();
                    Token::Lte
                } else {
                    Token::Lt
                }
            }
            c if c.is_ascii_digit() => self.scan_number()?,
            c if is_ident_start(c) => self.scan_ident()?,
            _ => {
                return Err(SassError::parse(format!(
                    "Unexpected character '{}' at position {}", c, self.pos
                )))
            }
        };

        Ok(Some(token))
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<()> {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => { self.bump(); }
                Some('/') if self.peek_n(1) == Some('/') => {
                    self.bump(); self.bump();
                    let mut s = String::new();
                    while let Some(c) = self.peek() {
                        if c == '\n' { break; }
                        s.push(c);
                        self.bump();
                    }
                    // SilentComment 暂时跳过
                }
                Some('/') if self.peek_n(1) == Some('*') => {
                    self.bump(); self.bump();
                    let mut s = String::new();
                    loop {
                        match self.peek() {
                            Some('*') if self.peek_n(1) == Some('/') => {
                                self.bump(); self.bump();
                                break;
                            }
                            Some(c) => { s.push(c); self.bump(); }
                            None => return Err(SassError::parse("Unterminated block comment")),
                        }
                    }
                    // 保留块注释
                    // TODO: 返回 Comment token
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn scan_hash(&mut self) -> Result<Token> {
        self.bump(); // #
        if self.peek() == Some('{') {
            self.bump(); // {
            let mut s = String::new();
            let mut depth = 1;
            while depth > 0 {
                match self.peek() {
                    Some('{') => { depth += 1; s.push('{'); self.bump(); }
                    Some('}') => { depth -= 1; if depth > 0 { s.push('}'); } self.bump(); }
                    Some(c) => { s.push(c); self.bump(); }
                    None => return Err(SassError::parse("Unterminated interpolation")),
                }
            }
            Ok(Token::Interp(s))
        } else {
            // #hex color
            let mut s = String::new();
            while let Some(c) = self.peek() {
                if c.is_ascii_hexdigit() {
                    s.push(c);
                    self.bump();
                } else {
                    break;
                }
            }
            if s.is_empty() {
                Ok(Token::Hash)
            } else {
                Ok(Token::HexColor(s))
            }
        }
    }

    fn scan_variable(&mut self) -> Result<Token> {
        self.bump(); // $
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if is_ident_char(c) {
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
        Ok(Token::Variable(s))
    }

    fn scan_string(&mut self, quote: char) -> Result<Token> {
        self.bump(); // opening quote
        let mut s = String::new();
        loop {
            match self.peek() {
                Some('\\') => {
                    self.bump();
                    if let Some(c) = self.peek() {
                        s.push(c);
                        self.bump();
                    }
                }
                Some(c) if c == quote => {
                    self.bump();
                    break;
                }
                Some(c) => { s.push(c); self.bump(); }
                None => return Err(SassError::parse("Unterminated string")),
            }
        }
        let style = if quote == '"' { QuoteStyle::Double } else { QuoteStyle::Single };
        Ok(Token::String(s, style))
    }

    fn scan_slash(&mut self) -> Result<Token> {
        // 已知不是 // 或 /*（已 skip）
        self.bump();
        Ok(Token::Slash)
    }

    fn scan_minus(&mut self) -> Result<Token> {
        self.bump(); // -
        // -- 是 CSS 自定义属性前缀
        if self.peek() == Some('-') {
            self.bump();
            return Ok(Token::Ident("--".to_string()));
        }
        // 数字？
        if let Some(c) = self.peek() {
            if c.is_ascii_digit() || (c == '.') {
                // 负数
                let num = self.scan_number_raw()?;
                return Ok(Token::Number(format!("-{}", num.0), num.1));
            }
        }
        Ok(Token::Minus)
    }

    fn scan_number(&mut self) -> Result<Token> {
        let (raw, unit) = self.scan_number_raw()?;
        Ok(Token::Number(raw, unit))
    }

    fn scan_number_raw(&mut self) -> Result<(String, Option<String>)> {
        let mut raw = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                raw.push(c);
                self.bump();
            } else {
                break;
            }
        }
        // 单位
        let mut unit = String::new();
        while let Some(c) = self.peek() {
            if c == '%' {
                unit.push(c);
                self.bump();
                break;
            }
            if is_ident_char(c) {
                unit.push(c);
                self.bump();
            } else {
                break;
            }
        }
        let unit = if unit.is_empty() { None } else { Some(unit) };
        Ok((raw, unit))
    }

    fn scan_ident(&mut self) -> Result<Token> {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if is_ident_char(c) || c == '-' {
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
        Ok(Token::Ident(s))
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '-'
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}
