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
        if let Some(ch) = c {
            self.pos += ch.len_utf8();
        }
        c
    }

    /// 扫描标识符或关键字（支持 Unicode）。
    pub(crate) fn scan_ident(&mut self) -> Token {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' || !c.is_ascii() {
                self.next_char();
            } else {
                break;
            }
        }
        let text = &self.source[start..self.pos];
        match text {
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
            if c.is_ascii_digit() {
                self.next_char();
            } else {
                break;
            }
        }
        if self.peek() == Some('.') && self.peek2().is_some_and(|c| c.is_ascii_digit()) {
            self.next_char();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.next_char();
                } else {
                    break;
                }
            }
        }
        if self.source[start..self.pos]
            .bytes()
            .any(|b| b.is_ascii_digit())
        {
            while let Some(c) = self.peek() {
                if c.is_ascii_alphabetic() || c == '%' {
                    self.next_char();
                } else {
                    break;
                }
            }
        }
        let text = &self.source[start..self.pos];
        if text == "." {
            Token::Dot
        } else {
            Token::Number(text.to_string())
        }
    }

    /// 扫描引号字符串（处理转义序列）。
    pub(crate) fn scan_string(&mut self, quote: char) -> Result<Token> {
        self.next_char();
        let mut content = String::new();
        while let Some(c) = self.peek() {
            if c == '\\' {
                self.next_char();
                if let Some(next) = self.peek() {
                    if next.is_ascii_hexdigit() {
                        let mut hex = String::new();
                        for _ in 0..6 {
                            if let Some(h) = self.peek() {
                                if h.is_ascii_hexdigit() {
                                    hex.push(h);
                                    self.next_char();
                                } else {
                                    break;
                                }
                            }
                        }
                        if self
                            .peek()
                            .is_some_and(|c| c == ' ' || c == '\t' || c == '\n')
                        {
                            self.next_char();
                        }
                        if let Ok(code) = u32::from_str_radix(&hex, 16)
                            && let Some(ch) = char::from_u32(code)
                        {
                            content.push(ch);
                        }
                    } else {
                        self.next_char();
                        content.push(next);
                    }
                }
                continue;
            }
            if c == quote {
                break;
            }
            content.push(c);
            self.next_char();
        }
        if self.peek().is_some() {
            self.next_char();
        }
        Ok(Token::String(content, quote))
    }

    /// 扫描反斜杠转义标识符（CSS 转义，如 \: \. 等）。
    pub(crate) fn scan_escape_ident(&mut self) -> Result<Token> {
        let mut text = String::new();
        self.next_char();
        if let Some(next) = self.peek() {
            if next.is_ascii_hexdigit() {
                let mut hex = String::new();
                for _ in 0..6 {
                    if let Some(h) = self.peek().filter(char::is_ascii_hexdigit) {
                        hex.push(h);
                        self.next_char();
                    } else {
                        break;
                    }
                }
                if self
                    .peek()
                    .is_some_and(|c| c == ' ' || c == '\t' || c == '\n')
                {
                    self.next_char();
                }
                if let Ok(code) = u32::from_str_radix(&hex, 16) {
                    if let Some(ch) = char::from_u32(code) {
                        text.push(ch);
                    } else {
                        return Err(SassError::Lex {
                            message: "Invalid Unicode code point.".into(),
                            pos: self.pos,
                        });
                    }
                }
            } else {
                self.next_char();
                text.push(next);
            }
        }
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' || !c.is_ascii() {
                self.next_char();
                text.push(c);
            } else {
                break;
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
            if c.is_ascii_alphanumeric() || c == '-' {
                name.push(c);
                self.next_char();
            } else if c == '\\' {
                self.next_char();
                if let Some(next) = self.peek() {
                    if next.is_ascii_hexdigit() {
                        let mut hex = String::new();
                        for _ in 0..6 {
                            if let Some(h) = self.peek().filter(char::is_ascii_hexdigit) {
                                hex.push(h);
                                self.next_char();
                            } else {
                                break;
                            }
                        }
                        if self
                            .peek()
                            .is_some_and(|c| c == ' ' || c == '\t' || c == '\n')
                        {
                            self.next_char();
                        }
                        if let Ok(code) = u32::from_str_radix(&hex, 16)
                            && let Some(ch) = char::from_u32(code)
                        {
                            name.push(ch);
                        }
                    } else {
                        self.next_char();
                        name.push(next);
                    }
                }
            } else {
                break;
            }
        }
        Token::AtRule(name)
    }

    /// 扫描 $变量。
    pub(crate) fn scan_dollar(&mut self) -> Token {
        self.next_char();
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                self.next_char();
            } else {
                break;
            }
        }
        let name = &self.source[start..self.pos];
        Token::Dollar(name.to_string())
    }

    /// 扫描 #hash 或 #插值。
    pub(crate) fn scan_hash(&mut self) -> Result<Token> {
        self.next_char();
        if self.peek() == Some('{') {
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
                        if depth == 0 {
                            break;
                        }
                        self.next_char();
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
            if self.peek() == Some('}') {
                self.next_char();
            }
            return Ok(Token::Interp(content));
        }
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                self.next_char();
            } else {
                break;
            }
        }
        let value = &self.source[start..self.pos];
        Ok(Token::Hash(value.to_string()))
    }

    /// 扫描 // 单行注释。
    pub(crate) fn scan_line_comment(&mut self) -> Token {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.next_char();
        }
        let text = self.source[start..self.pos].trim().to_string();
        Token::Comment(text, true)
    }

    /// 扫描 /* */ 多行注释。
    pub(crate) fn scan_block_comment(&mut self) -> Token {
        let start = self.pos;
        while self.peek().is_some() {
            if self.source[self.pos..].starts_with("*/") {
                self.next_char();
                self.next_char();
                break;
            }
            self.next_char();
        }
        let text = self.source[start..self.pos.saturating_sub(2)]
            .trim()
            .to_string();
        Token::Comment(text, false)
    }
}
