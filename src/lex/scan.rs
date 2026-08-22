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

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_n(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    /// 读取下一个 token。
    pub fn next_token(&mut self) -> Result<Option<Token>> {
        self.skip_whitespace();

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
            '@' => self.scan_at_rule()?,
            '#' => self.scan_hash()?,
            '$' => self.scan_variable()?,
            '"' => self.scan_string('"')?,
            '\'' => self.scan_string('\'')?,
            '/' => self.scan_slash()?,
            '!' => self.scan_bang()?,
            '.' => self.scan_dot()?,
            '+' => { self.bump(); Token::Plus }
            '-' => self.scan_minus()?,
            '*' => { self.bump(); Token::Star }
            '%' => { self.bump(); Token::Percent }
            '^' => { self.bump(); Token::Caret }
            '|' => { self.bump(); Token::Pipe }
            '=' => {
                self.bump();
                if self.peek() == Some('>') {
                    self.bump();
                    Token::Arrow
                } else {
                    Token::Eq
                }
            }
            '>' => {
                self.bump();
                if self.peek() == Some('=') { self.bump(); Token::Gte }
                else { Token::Gt }
            }
            '<' => {
                self.bump();
                if self.peek() == Some('=') { self.bump(); Token::Lte }
                else { Token::Lt }
            }
            '&' => { self.bump(); Token::Ampersand }
            '\\' => self.scan_escape_ident()?,
            c if c.is_ascii_digit() => self.scan_number()?,
            c if is_ident_start(c) => self.scan_ident()?,
            c if !c.is_ascii() => self.scan_ident()?,
            _ => {
                return Err(SassError::parse(format!(
                    "Unexpected character '{}' at position {}", c, self.pos
                )))
            }
        };

        Ok(Some(token))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn scan_at_rule(&mut self) -> Result<Token> {
        self.bump(); // @
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '-' {
                name.push(c);
                self.bump();
            } else if c == '\\' {
                self.bump(); // \
                if let Some(next) = self.peek() {
                    if next.is_ascii_hexdigit() {
                        let hex = self.scan_hex_escape()?;
                        if let Some(ch) = hex {
                            name.push(ch);
                        }
                    } else {
                        self.bump();
                        name.push(next);
                    }
                }
            } else {
                break;
            }
        }
        Ok(Token::AtRule(name))
    }

    fn scan_hash(&mut self) -> Result<Token> {
        self.bump(); // #
        if self.peek() == Some('{') {
            self.bump(); // {
            let s = self.scan_interp_body()?;
            Ok(Token::Interp(s))
        } else {
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

    /// 扫描插值体 `#{...}`——已消费 `#{`，扫描到匹配的 `}`。
    fn scan_interp_body(&mut self) -> Result<String> {
        let mut s = String::new();
        let mut depth = 1;
        while depth > 0 {
            match self.peek() {
                Some('{') => { depth += 1; s.push('{'); self.bump(); }
                Some('}') => {
                    depth -= 1;
                    if depth > 0 { s.push('}'); }
                    self.bump();
                }
                Some('"') | Some('\'') => {
                    let q = self.peek().unwrap();
                    let (content, _) = self.scan_string_content(q)?;
                    s.push(q);
                    s.push_str(&content);
                    s.push(q);
                }
                Some(c) => { s.push(c); self.bump(); }
                None => return Err(SassError::parse("Unterminated interpolation")),
            }
        }
        Ok(s)
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
        let (content, style) = self.scan_string_content(quote)?;
        Ok(Token::String(content, style))
    }

    /// 扫描字符串内容——已消费起始引号，返回 (content, quote_style)。
    fn scan_string_content(&mut self, quote: char) -> Result<(String, QuoteStyle)> {
        let mut s = String::new();
        loop {
            match self.peek() {
                Some('\\') => {
                    self.bump();
                    if let Some(next) = self.peek() {
                        if next.is_ascii_hexdigit() {
                            let hex = self.scan_hex_escape()?;
                            if let Some(ch) = hex {
                                s.push(ch);
                            }
                        } else {
                            self.bump();
                            s.push(next);
                        }
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
        Ok((s, style))
    }

    /// 扫描十六进制转义（1-6 位 hex digits + 可选尾部空白）。
    fn scan_hex_escape(&mut self) -> Result<Option<char>> {
        let mut hex = String::new();
        for _ in 0..6 {
            if let Some(h) = self.peek() {
                if h.is_ascii_hexdigit() {
                    hex.push(h);
                    self.bump();
                } else {
                    break;
                }
            }
        }
        // 跳过尾部一个空白字符
        if let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\n' {
                self.bump();
            }
        }
        if let Ok(code) = u32::from_str_radix(&hex, 16) {
            Ok(char::from_u32(code))
        } else {
            Ok(None)
        }
    }

    fn scan_slash(&mut self) -> Result<Token> {
        self.bump(); // /
        match self.peek() {
            Some('/') => {
                self.bump();
                let mut s = String::new();
                while let Some(c) = self.peek() {
                    if c == '\n' { break; }
                    s.push(c);
                    self.bump();
                }
                Ok(Token::SilentComment(s))
            }
            Some('*') => {
                self.bump();
                let mut s = String::new();
                loop {
                    match self.peek() {
                        Some('*') if self.peek_n(1) == Some('/') => {
                            self.bump();
                            self.bump();
                            break;
                        }
                        Some(c) => { s.push(c); self.bump(); }
                        None => return Err(SassError::parse("Unterminated block comment")),
                    }
                }
                Ok(Token::Comment(s))
            }
            _ => Ok(Token::Slash),
        }
    }

    fn scan_bang(&mut self) -> Result<Token> {
        self.bump(); // !
        if self.peek() == Some('=') {
            self.bump();
            Ok(Token::NotEq)
        } else {
            Ok(Token::Bang)
        }
    }

    fn scan_dot(&mut self) -> Result<Token> {
        self.bump(); // .
        if self.peek() == Some('.') && self.peek_n(1) == Some('.') {
            self.bump();
            self.bump();
            Ok(Token::DotDotDot)
        } else if let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                // 小数 .5
                let mut raw = String::from(".");
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        raw.push(c);
                        self.bump();
                    } else {
                        break;
                    }
                }
                let unit = self.scan_unit();
                Ok(Token::Number(raw, unit))
            } else {
                Ok(Token::Dot)
            }
        } else {
            Ok(Token::Dot)
        }
    }

    fn scan_minus(&mut self) -> Result<Token> {
        self.bump(); // -
        // -- 是 CSS 自定义属性前缀
        if self.peek() == Some('-') {
            self.bump();
            return Ok(Token::Ident("--".to_string()));
        }
        // 负数？
        if let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                let num = self.scan_number_raw()?;
                return Ok(Token::Number(format!("-{}", num.0), num.1));
            }
            if c == '.' && self.peek_n(1).is_some_and(|c| c.is_ascii_digit()) {
                let mut raw = String::from("-.");
                self.bump(); // .
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() { raw.push(c); self.bump(); }
                    else { break; }
                }
                let unit = self.scan_unit();
                return Ok(Token::Number(raw, unit));
            }
            // -ident（以 - 开头的标识符）
            if is_ident_start(c) || !c.is_ascii() {
                let mut s = String::from("-");
                while let Some(c) = self.peek() {
                    if is_ident_char(c) || !c.is_ascii() {
                        s.push(c);
                        self.bump();
                    } else {
                        break;
                    }
                }
                return Ok(keyword_or_ident(s));
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
            if c.is_ascii_digit() {
                raw.push(c);
                self.bump();
            } else {
                break;
            }
        }
        // 小数部分
        if self.peek() == Some('.') && self.peek_n(1).is_some_and(|c| c.is_ascii_digit()) {
            raw.push('.');
            self.bump();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() { raw.push(c); self.bump(); }
                else { break; }
            }
        }
        // 科学计数法 e/E
        if matches!(self.peek(), Some('e') | Some('E')) {
            let saved = self.pos;
            self.bump();
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.bump();
            }
            if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                raw.push('e');
                if let Some(c) = self.peek() { raw.push(c); self.bump(); }
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() { raw.push(c); self.bump(); }
                    else { break; }
                }
            } else {
                self.pos = saved;
            }
        }
        let unit = self.scan_unit();
        Ok((raw, unit))
    }

    fn scan_unit(&mut self) -> Option<String> {
        let mut unit = String::new();
        if self.peek() == Some('%') {
            unit.push('%');
            self.bump();
            return Some(unit);
        }
        while let Some(c) = self.peek() {
            if is_ident_char(c) {
                unit.push(c);
                self.bump();
            } else {
                break;
            }
        }
        if unit.is_empty() { None } else { Some(unit) }
    }

    fn scan_ident(&mut self) -> Result<Token> {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if is_ident_char(c) || !c.is_ascii() {
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
        Ok(keyword_or_ident(s))
    }

    /// 扫描反斜杠转义标识符（CSS 转义如 \: \. 等）。
    fn scan_escape_ident(&mut self) -> Result<Token> {
        self.bump(); // \
        let mut s = String::new();
        if let Some(next) = self.peek() {
            if next.is_ascii_hexdigit() {
                let ch = self.scan_hex_escape()?;
                if let Some(c) = ch {
                    s.push(c);
                }
            } else {
                self.bump();
                s.push(next);
            }
        }
        // 继续扫描后续标识符字符
        while let Some(c) = self.peek() {
            if is_ident_char(c) || !c.is_ascii() {
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
        Ok(keyword_or_ident(s))
    }
}

fn keyword_or_ident(s: String) -> Token {
    match s.as_str() {
        "true" => Token::True,
        "false" => Token::False,
        "null" => Token::Null,
        "and" => Token::And,
        "or" => Token::Or,
        "not" => Token::Not,
        _ => Token::Ident(s),
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '-'
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}
