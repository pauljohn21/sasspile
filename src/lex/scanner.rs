//! Lexer 扫描方法——标识符/数字/字符串/注释/插值/转义等。

use super::token::Token;
use crate::error::{Result, SassError};

/// 词法分析器——惰性扫描源码字符串。
pub struct Lexer<'src> {
    /// 源码引用。
    pub(crate) source: &'src str,
    /// 字符迭代器。
    pub(crate) chars: std::iter::Peekable<std::str::Chars<'src>>,
    /// 当前字节位置。
    pub(crate) pos: usize,
}

impl<'src> Lexer<'src> {
    /// 创建新的 Lexer。
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            pos: 0,
        }
    }

    /// 查看下一个字符（不消费）。
    pub(crate) fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// 查看下下个字符（不消费）——直接从源码字符串索引。
    pub(crate) fn peek2(&self) -> Option<char> {
        let remainder = &self.source[self.pos..];
        let mut iter = remainder.chars();
        iter.next();
        iter.next()
    }

    /// 消费下一个字符并前进。
    pub(crate) fn next_char(&mut self) -> Option<char> {
        let c = self.chars.next();
        match c {
            Some(ch) => {
                self.pos += ch.len_utf8();
            }
            None => {}
        }
        c
    }

    /// 扫描标识符或关键字（支持 Unicode）。
    pub(crate) fn scan_ident(&mut self) -> Token {
        let start = self.pos;
        while let Some(c) = self.peek() {
            match c.is_alphanumeric() || c == '-' || c == '_' || !c.is_ascii() {
                true => { self.next_char(); }
                false => break,
            }
        }
        let text = &self.source[start..self.pos];
        match text.to_lowercase().as_str() {
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            _ => Token::Ident(text.to_string()),
        }
    }

    /// 扫描数字（整数或小数，含单位）。
    pub(crate) fn scan_number(&mut self) -> Token {
        let start = self.pos;
        while let Some(c) = self.peek() {
            match c.is_ascii_digit() {
                true => { self.next_char(); }
                false => break,
            }
        }
        match self.peek() == Some('.') && self.peek2().is_some_and(|c| c.is_ascii_digit()) {
            true => {
                self.next_char();
                while let Some(c) = self.peek() {
                    match c.is_ascii_digit() {
                        true => { self.next_char(); }
                        false => break,
                    }
                }
            }
            false => {}
        }
        match self.source[start..self.pos].bytes().any(|b| b.is_ascii_digit()) {
            true => {
                while let Some(c) = self.peek() {
                    match c.is_ascii_alphabetic() || c == '%' {
                        true => { self.next_char(); }
                        false => break,
                    }
                }
            }
            false => {}
        }
        let text = &self.source[start..self.pos];
        match text {
            "." => Token::Dot,
            _ => Token::Number(text.to_string()),
        }
    }

    /// 扫描引号字符串（处理转义序列）。
    pub(crate) fn scan_string(&mut self, quote: char) -> Result<Token> {
        self.next_char();
        let mut content = String::new();
        while let Some(c) = self.peek() {
            match c {
                '\\' => {
                    self.next_char();
                    match self.peek() {
                        Some(next) => {
                            match next.is_ascii_hexdigit() {
                                true => {
                                    let mut hex = String::new();
                                    for _ in 0..6 {
                                        match self.peek() {
                                            Some(h) if h.is_ascii_hexdigit() => {
                                                hex.push(h);
                                                self.next_char();
                                            }
                                            _ => break,
                                        }
                                    }
                                    match self
                                        .peek()
                                        .is_some_and(|c| c == ' ' || c == '\t' || c == '\n')
                                    {
                                        true => { self.next_char(); }
                                        false => {}
                                    }
                                    match u32::from_str_radix(&hex, 16)
                                        .ok()
                                        .and_then(char::from_u32)
                                    {
                                        Some(ch) => { content.push(ch); }
                                        None => {}
                                    }
                                }
                                false => {
                                    self.next_char();
                                    content.push(next);
                                }
                            }
                        }
                        None => {}
                    }
                    continue;
                }
                _ if c == quote => break,
                _ => {
                    content.push(c);
                    self.next_char();
                }
            }
        }
        match self.peek().is_some() {
            true => { self.next_char(); }
            false => {}
        }
        Ok(Token::String(content, quote))
    }

    /// 扫描反斜杠转义标识符（CSS 转义，如 \: \. 等）。
    pub(crate) fn scan_escape_ident(&mut self) -> Result<Token> {
        let mut text = String::new();
        self.next_char();
        match self.peek() {
            Some(next) => {
                match next.is_ascii_hexdigit() {
                    true => {
                        let mut hex = String::new();
                        for _ in 0..6 {
                            match self.peek().filter(char::is_ascii_hexdigit) {
                                Some(h) => {
                                    hex.push(h);
                                    self.next_char();
                                }
                                None => break,
                            }
                        }
                        match self
                            .peek()
                            .is_some_and(|c| c == ' ' || c == '\t' || c == '\n')
                        {
                            true => { self.next_char(); }
                            false => {}
                        }
                        match u32::from_str_radix(&hex, 16) {
                            Ok(code) => {
                                match char::from_u32(code) {
                                    Some(ch) => { text.push(ch); }
                                    None => {
                                        return Err(SassError::Lex {
                                            message: "Invalid Unicode code point.".into(),
                                            pos: self.pos,
                                        });
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    false => {
                        self.next_char();
                        text.push(next);
                    }
                }
            }
            None => {}
        }
        while let Some(c) = self.peek() {
            match c.is_alphanumeric() || c == '-' || c == '_' || !c.is_ascii() {
                true => {
                    self.next_char();
                    text.push(c);
                }
                false => break,
            }
        }
        Ok(match text.as_str() {
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            _ => Token::Ident(text),
        })
    }

    /// 扫描 @规则（支持反斜杠转义）。
    pub(crate) fn scan_at(&mut self) -> Token {
        self.next_char();
        let mut name = String::new();
        while let Some(c) = self.peek() {
            match c {
                _ if c.is_ascii_alphanumeric() || c == '-' => {
                    name.push(c);
                    self.next_char();
                }
                '\\' => {
                    self.next_char();
                    match self.peek() {
                        Some(next) => {
                            match next.is_ascii_hexdigit() {
                                true => {
                                    let mut hex = String::new();
                                    for _ in 0..6 {
                                        match self.peek().filter(char::is_ascii_hexdigit) {
                                            Some(h) => {
                                                hex.push(h);
                                                self.next_char();
                                            }
                                            None => break,
                                        }
                                    }
                                    match self
                                        .peek()
                                        .is_some_and(|c| c == ' ' || c == '\t' || c == '\n')
                                    {
                                        true => { self.next_char(); }
                                        false => {}
                                    }
                                    match u32::from_str_radix(&hex, 16)
                                        .ok()
                                        .and_then(char::from_u32)
                                    {
                                        Some(ch) => { name.push(ch); }
                                        None => {}
                                    }
                                }
                                false => {
                                    self.next_char();
                                    name.push(next);
                                }
                            }
                        }
                        None => {}
                    }
                }
                _ => break,
            }
        }
        Token::AtRule(name)
    }

    /// 扫描 $变量。
    pub(crate) fn scan_dollar(&mut self) -> Token {
        self.next_char();
        let start = self.pos;
        while let Some(c) = self.peek() {
            match c.is_alphanumeric() || c == '-' || c == '_' {
                true => { self.next_char(); }
                false => break,
            }
        }
        let name = &self.source[start..self.pos];
        Token::Dollar(name.to_string())
    }

    /// 扫描 #hash 或 #插值。
    pub(crate) fn scan_hash(&mut self) -> Result<Token> {
        self.next_char();
        match self.peek() {
            Some('{') => {
                self.next_char();
                let start = self.pos;
                let mut depth = 1;
                while depth > 0 {
                    match self.peek() {
                        Some('{') => {
                            depth += 1;
                            self.next_char();
                        }
                        Some('}') => {
                            depth -= 1;
                            match depth == 0 {
                                true => break,
                                false => { self.next_char(); }
                            }
                        }
                        Some('"' | '\'') => {
                            let q = self.peek().expect("peek is Some in quote branch");
                            let _ = self.scan_string(q)?;
                        }
                        Some(_) => {
                            self.next_char();
                        }
                        None => break,
                    }
                }
                let content = self.source[start..self.pos].to_string();
                match self.peek() {
                    Some('}') => { self.next_char(); }
                    _ => {}
                }
                Ok(Token::Interp(content))
            }
            _ => {
                let start = self.pos;
                while let Some(c) = self.peek() {
                    match c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                        true => { self.next_char(); }
                        false => break,
                    }
                }
                let value = &self.source[start..self.pos];
                Ok(Token::Hash(value.to_string()))
            }
        }
    }

    /// 扫描 // 单行注释。
    pub(crate) fn scan_line_comment(&mut self) -> Token {
        let start = self.pos;
        while let Some(c) = self.peek() {
            match c {
                '\n' => break,
                _ => { self.next_char(); }
            }
        }
        let text = self.source[start..self.pos].trim().to_string();
        Token::Comment(text, true)
    }

    /// 扫描 /* */ 多行注释。
    pub(crate) fn scan_block_comment(&mut self) -> Token {
        let start = self.pos;
        while self.peek().is_some() {
            match self.source[self.pos..].starts_with("*/") {
                true => {
                    self.next_char();
                    self.next_char();
                    break;
                }
                false => { self.next_char(); }
            }
        }
        let text = self.source[start..self.pos.saturating_sub(2)]
            .trim()
            .to_string();
        Token::Comment(text, false)
    }
}
