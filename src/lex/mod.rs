//! 词法分析器——迭代器实现。
//!
//! Lexer 自身实现 `Iterator<Item = Result<Token, SassError>>`，
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
            // 支持 ASCII 字母数字、连字符、下划线，以及非 ASCII 字符（中文等）
            if c.is_alphanumeric() || c == '-' || c == '_' {
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
        if self.peek() == Some('.') {
            self.next_char();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.next_char();
                } else {
                    break;
                }
            }
        }
        // 单位（仅在后面跟数字时才有意义）
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
        // 如果只有 "." 没有数字，返回 Dot
        if text == "." {
            Token::Dot
        } else {
            Token::Number(text.to_string())
        }
    }

    /// 扫描引号字符串。
    fn scan_string(&mut self, quote: char) -> Result<Token> {
        self.next_char(); // 消费起始引号
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == quote {
                break;
            }
            self.next_char();
        }
        let content = &self.source[start..self.pos];
        self.next_char(); // 消费结束引号
        Ok(Token::String(content.to_string()))
    }

    /// 扫描 @规则。
    fn scan_at(&mut self) -> Token {
        self.next_char(); // 消费 @
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '-' {
                self.next_char();
            } else {
                break;
            }
        }
        let name = &self.source[start..self.pos];
        Token::AtRule(name.to_string())
    }

    /// 扫描 $变量（支持 Unicode）。
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

    /// 扫描 // 单行注释。
    fn scan_single_line_comment(&mut self) -> crate::error::Result<Token> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.next_char();
        }
        let text = &self.source[start..self.pos];
        Ok(Token::Comment(text.to_string()))
    }

    /// 扫描 /* ... */ 注释。
    fn scan_comment(&mut self) -> crate::error::Result<Token> {
        let start = self.pos;
        while self.peek().is_some() {
            if self.source[self.pos..].starts_with("*/") {
                self.next_char(); // 消费 *
                self.next_char(); // 消费 /
                break;
            }
            self.next_char();
        }
        let text = &self.source[start..self.pos];
        Ok(Token::Comment(text.to_string()))
    }

    /// 扫描 #hash（颜色或 ID）。
    fn scan_hash(&mut self) -> Token {
        self.next_char(); // 消费 #
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() {
                self.next_char();
            } else {
                break;
            }
        }
        let value = &self.source[start..self.pos];
        Token::Hash(value.to_string())
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = crate::error::Result<Token>;

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
            // 数字/点
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
            '-' => {
                self.next_char();
                Token::Minus
            }
            '*' => {
                self.next_char();
                Token::Star
            }
            '%' => {
                self.next_char();
                Token::Percent
            }
            '!' => {
                self.next_char();
                Token::Bang
            }
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
            '.' => self.scan_number(),
            '/' => {
                self.next_char();
                // 检查注释
                if self.peek() == Some('*') {
                    self.next_char(); // 消费 *
                    return Some(self.scan_comment());
                }
                // 单行注释 //
                if self.peek() == Some('/') {
                    self.next_char(); // 消费第二个 /
                    return Some(self.scan_single_line_comment());
                }
                Token::Slash
            }
            '"' | '\'' => match self.scan_string(c) {
                Ok(t) => return Some(Ok(t)),
                Err(e) => return Some(Err(e)),
            },
            '@' => self.scan_at(),
            '$' => self.scan_dollar(),
            '#' => self.scan_hash(),
            // 非 ASCII 字符——作为标识符的一部分（支持中文等）
            c if !c.is_ascii() => self.scan_ident(),
            _ => {
                return Some(Err(SassError::LexError {
                    message: format!("无效字符: '{c}'"),
                    position: self.pos,
                }));
            }
        };

        Some(Ok(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_ident() {
        let tokens: Vec<_> = Lexer::new("color")
            .collect::<crate::error::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::Ident("color".to_string())]);
    }

    #[test]
    fn test_lex_number() {
        let tokens: Vec<_> = Lexer::new("16px")
            .collect::<crate::error::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::Number("16px".to_string())]);
    }

    #[test]
    fn test_lex_variable() {
        let tokens: Vec<_> = Lexer::new("$color")
            .collect::<crate::error::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::Dollar("color".to_string())]);
    }

    #[test]
    fn test_lex_at_rule() {
        let tokens: Vec<_> = Lexer::new("@import")
            .collect::<crate::error::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::AtRule("import".to_string())]);
    }

    #[test]
    fn test_lex_string() {
        let tokens: Vec<_> = Lexer::new("\"hello\"")
            .collect::<crate::error::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::String("hello".to_string())]);
    }

    #[test]
    fn test_lex_media_query() {
        let input = "@media (min-width: 576px)";
        let tokens: Vec<_> = Lexer::new(input)
            .collect::<crate::error::Result<Vec<_>>>()
            .unwrap();
        // 验证完整 token 序列（包含 Whitespace）
        assert_eq!(
            tokens,
            vec![
                Token::AtRule("media".to_string()),
                Token::Whitespace,
                Token::LParen,
                Token::Ident("min-width".to_string()),
                Token::Colon,
                Token::Whitespace,
                Token::Number("576px".to_string()),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_lex_symbols() {
        let tokens: Vec<_> = Lexer::new("{}();:.")
            .filter(|t| !matches!(t.as_ref(), Ok(Token::Whitespace)))
            .collect::<crate::error::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LBrace,
                Token::RBrace,
                Token::LParen,
                Token::RParen,
                Token::Semicolon,
                Token::Colon,
                Token::Dot,
            ]
        );
    }

    #[test]
    fn test_lex_keywords() {
        let tokens: Vec<_> = Lexer::new("true false null")
            .filter(|t| !matches!(t.as_ref(), Ok(Token::Whitespace)))
            .collect::<crate::error::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::True, Token::False, Token::Null]);
    }

    #[test]
    fn test_lex_error_on_invalid() {
        let result: crate::error::Result<Vec<_>> = Lexer::new("\x00").collect();
        assert!(result.is_err());
    }
}
