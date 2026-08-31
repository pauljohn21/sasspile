//! 词法分析器——迭代器实现。
//!
//! Lexer 实现 `Iterator<Item = Result<Token, SassError>>`，
//! 逐字符扫描源码，产出 token 流。

pub mod token;

use token::Token;

use crate::error::{Result, SassError};

/// 词法分析器——惰性扫描源码字符串。
pub struct Lexer<'src> {
    /// 源码引用。
    source: &'src str,
    /// 字符迭代器。
    chars: std::iter::Peekable<std::str::Chars<'src>>,
    /// 当前字节位置。
    pos: usize,
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
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// 查看下下个字符（不消费）——直接从源码字符串索引，避免克隆迭代器。
    fn peek2(&self) -> Option<char> {
        let remainder = &self.source[self.pos..];
        let mut iter = remainder.chars();
        iter.next(); // 跳过第一个字符
        iter.next() // 返回第二个字符
    }

    /// 消费下一个字符并前进。
    fn next_char(&mut self) -> Option<char> {
        let c = self.chars.next();
        if let Some(ch) = c {
            self.pos += ch.len_utf8();
        }
        c
    }

    /// 扫描标识符或关键字（支持 Unicode）。
    fn scan_ident(&mut self) -> Token {
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
    fn scan_number(&mut self) -> Token {
        let start = self.pos;
        // 整数部分
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.next_char();
            } else {
                break;
            }
        }
        // 小数部分
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
        // 单位
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
    fn scan_string(&mut self, quote: char) -> Result<Token> {
        self.next_char(); // 消费起始引号
        let mut content = String::new();
        while let Some(c) = self.peek() {
            if c == '\\' {
                self.next_char(); // 消费 \
                if let Some(next) = self.peek() {
                    if next.is_ascii_hexdigit() {
                        // 十六进制转义：\XXXX（1-6 位十六进制）
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
                        // 跳过尾部一个空白字符
                        if self.peek().is_some_and(|c| c == ' ' || c == '\t' || c == '\n') {
                            self.next_char();
                        }
                        if let Ok(code) = u32::from_str_radix(&hex, 16)
                            && let Some(ch) = char::from_u32(code) {
                                content.push(ch);
                            }
                    } else {
                        // 非十六进制转义：\X → X（如 \" → ", \\ → \）
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
            self.next_char(); // 消费结束引号
        }
        Ok(Token::String(content, quote))
    }

    /// 扫描反斜杠转义标识符（CSS 转义，如 \: \. 等）。
    fn scan_escape_ident(&mut self) -> Result<Token> {
        let mut text = String::new();
        self.next_char();
        if let Some(next) = self.peek() {
            if next.is_ascii_hexdigit() {
                let mut hex = String::new();
                for _ in 0..6 {
                    if let Some(h) = self.peek().filter(|c| c.is_ascii_hexdigit()) {
                        hex.push(h);
                        self.next_char();
                    } else { break; }
                }
                if self.peek().is_some_and(|c| c == ' ' || c == '\t' || c == '\n') { self.next_char(); }
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
            } else { self.next_char(); text.push(next); }
        }
        // 继续扫描后续标识符字符
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

    /// 扫描 @规则（支持反斜杠转义，如 `@u\\73 e` → `@use`）。
    fn scan_at(&mut self) -> Token {
        self.next_char(); // 消费 @
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '-' {
                name.push(c);
                self.next_char();
            } else if c == '\\' {
                self.next_char(); // 消费 \
                if let Some(next) = self.peek() {
                    if next.is_ascii_hexdigit() {
                        // 十六进制转义：\73 → 's'
                        let mut hex = String::new();
                        for _ in 0..6 {
                            if let Some(h) = self.peek().filter(|c| c.is_ascii_hexdigit()) {
                                hex.push(h);
                                self.next_char();
                            } else {
                                break;
                            }
                        }
                        // 跳过尾部一个空白字符
                        if self.peek().is_some_and(|c| c == ' ' || c == '\t' || c == '\n') {
                            self.next_char();
                        }
                        if let Ok(code) = u32::from_str_radix(&hex, 16)
                            && let Some(ch) = char::from_u32(code)
                        {
                            name.push(ch);
                        }
                    } else {
                        // 非十六进制转义：\X → X
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
    fn scan_dollar(&mut self) -> Token {
        self.next_char(); // 消费 $
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
    fn scan_hash(&mut self) -> Result<Token> {
        self.next_char(); // 消费 #
        if self.peek() == Some('{') {
            self.next_char(); // 消费 {
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
                    Some('"') | Some('\'') => {
                        let q = self.peek().unwrap();
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
    fn scan_line_comment(&mut self) -> Token {
        // 已消费两个 /，跳过到行尾
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
    fn scan_block_comment(&mut self) -> Token {
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

impl<'src> Iterator for Lexer<'src> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.peek()?;
        let token = match c {
            // 空白
            ' ' | '\t' | '\n' | '\r' => {
                self.next_char();
                Token::Whitespace
            }
            // 标识符/关键字
            'a'..='z' | 'A'..='Z' | '_' => self.scan_ident(),
            // 数字
            '0'..='9' => self.scan_number(),
            // 符号
            '{' => {
                self.next_char();
                Token::LBrace
            }
            '}' => {
                self.next_char();
                Token::RBrace
            }
            '(' => {
                self.next_char();
                Token::LParen
            }
            ')' => {
                self.next_char();
                Token::RParen
            }
            '[' => {
                self.next_char();
                Token::LBracket
            }
            ']' => {
                self.next_char();
                Token::RBracket
            }
            ':' => {
                self.next_char();
                Token::Colon
            }
            ';' => {
                self.next_char();
                Token::Semicolon
            }
            ',' => {
                self.next_char();
                Token::Comma
            }
            '+' => {
                self.next_char();
                Token::Plus
            }
            '*' => {
                self.next_char();
                Token::Star
            }
            '%' => {
                self.next_char();
                Token::Percent
            }
            '&' => {
                self.next_char();
                Token::Amp
            }
            '^' => {
                self.next_char();
                Token::Caret
            }
            '~' => {
                self.next_char();
                Token::Tilde
            }
            '|' => {
                self.next_char();
                Token::Pipe
            }
            // 点——可能是小数开始或 Dot
            '.' => {
                if self.peek2().is_some_and(|c| c.is_ascii_digit()) {
                    self.scan_number()
                } else if self.peek2() == Some('.') && self.source[self.pos..].starts_with("...") {
                    self.next_char();
                    self.next_char();
                    self.next_char();
                    Token::DotDotDot
                } else {
                    self.next_char();
                    Token::Dot
                }
            }
            // 减号——可能是负数、标识符开头（如 -real-channel）或减号运算符
            '-' => {
                // 如果下一个字符是标识符字符（非数字），则扫描为以 - 开头的标识符
                let next = self.peek2();
                if next.is_some_and(|c| c.is_alphabetic() || c == '-' || c == '_' || !c.is_ascii()) {
                    self.next_char(); // 消费 -
                    let start = self.pos;
                    while let Some(c) = self.peek() {
                        if c.is_alphanumeric() || c == '-' || c == '_' || !c.is_ascii() {
                            self.next_char();
                        } else {
                            break;
                        }
                    }
                    let text = format!("-{}", &self.source[start..self.pos]);
                    match text.as_str() {
                        "true" => Token::True,
                        "false" => Token::False,
                        "null" => Token::Null,
                        "and" => Token::And,
                        "or" => Token::Or,
                        "not" => Token::Not,
                        _ => Token::Ident(text),
                    }
                } else {
                    self.next_char();
                    Token::Minus
                }
            }
            // 斜杠——可能是注释或除法
            '/' => {
                self.next_char();
                match self.peek() {
                    Some('/') => {
                        self.next_char();
                        self.scan_line_comment()
                    }
                    Some('*') => {
                        self.next_char();
                        self.scan_block_comment()
                    }
                    _ => Token::Slash,
                }
            }
            // 感叹号
            '!' => {
                self.next_char();
                if self.peek() == Some('=') {
                    self.next_char();
                    Token::NotEq
                } else {
                    Token::Bang
                }
            }
            // 等号/比较
            '=' => {
                self.next_char();
                if self.peek() == Some('=') {
                    self.next_char();
                    Token::Eq
                } else {
                    Token::Assign
                }
            }
            '<' => {
                self.next_char();
                if self.peek() == Some('=') {
                    self.next_char();
                    Token::LessEq
                } else {
                    Token::Less
                }
            }
            '>' => {
                self.next_char();
                if self.peek() == Some('=') {
                    self.next_char();
                    Token::GreaterEq
                } else {
                    Token::Greater
                }
            }
            // 字符串
            '"' | '\'' => return Some(self.scan_string(c)),
            // 反斜杠转义标识符（CSS 转义，如 \: \. 等）
            '\\' => return Some(self.scan_escape_ident()),
            // @规则
            '@' => self.scan_at(),
            // $变量
            '$' => self.scan_dollar(),
            // # hash 或插值
            '#' => return Some(self.scan_hash()),
            // 非 ASCII——作为标识符
            c if !c.is_ascii() => self.scan_ident(),
            // 未知
            _ => {
                self.next_char();
                return Some(Err(SassError::Lex {
                    message: format!("无效字符: '{c}'"),
                    pos: self.pos,
                }));
            }
        };
        Some(Ok(token))
    }
}
