//! 词法分析器——Iterator 实现。
//!
//! Lexer 实现 `Iterator<Item = Result<Token, SassError>>`，
//! 逐字符扫描源码，产出 token 流。扫描方法在 `scanner` 模块中。

pub mod token;
mod scanner;

pub use scanner::Lexer;
use token::Token;

use crate::error::{Result, SassError};

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
            '{' => { self.next_char(); Token::LBrace }
            '}' => { self.next_char(); Token::RBrace }
            '(' => { self.next_char(); Token::LParen }
            ')' => { self.next_char(); Token::RParen }
            '[' => { self.next_char(); Token::LBracket }
            ']' => { self.next_char(); Token::RBracket }
            ':' => { self.next_char(); Token::Colon }
            ';' => { self.next_char(); Token::Semicolon }
            ',' => { self.next_char(); Token::Comma }
            '+' => { self.next_char(); Token::Plus }
            '*' => { self.next_char(); Token::Star }
            '%' => { self.next_char(); Token::Percent }
            '&' => { self.next_char(); Token::Amp }
            '^' => { self.next_char(); Token::Caret }
            '~' => { self.next_char(); Token::Tilde }
            '|' => { self.next_char(); Token::Pipe }
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
            // 减号——可能是负数、标识符开头或减号运算符
            '-' => {
                let next = self.peek2();
                if next.is_some_and(|c| c.is_alphabetic() || c == '-' || c == '_' || !c.is_ascii()) {
                    self.next_char();
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
                    Some('/') => { self.next_char(); self.scan_line_comment() }
                    Some('*') => { self.next_char(); self.scan_block_comment() }
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
            // 反斜杠转义标识符
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
