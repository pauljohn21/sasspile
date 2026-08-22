//! Parser — 语法分析，Token → AST。

use crate::error::{Result, SassError};
use crate::lex::{Lexed, Token};
use std::path::PathBuf;

pub(crate) mod ast;
mod expr;
mod rule;
mod decl;
mod variable;
mod body;
mod at_rules;

pub use ast::*;

/// 语法分析完成。
pub(crate) struct Parsed {
    pub(crate) ast: Ast,
    pub(crate) base_path: Option<PathBuf>,
    pub(crate) load_paths: Vec<PathBuf>,
}

impl TryFrom<Lexed> for Parsed {
    type Error = SassError;

    fn try_from(lexed: Lexed) -> Result<Self> {
        let mut parser = Parser::new(lexed.tokens);
        let ast = parser.parse_body()?;
        Ok(Self {
            ast,
            base_path: lexed.base_path,
            load_paths: lexed.load_paths,
        })
    }
}

impl Parsed {
    /// 求值——Parsed → Evaluated。
    pub fn evaluate(self) -> Result<crate::eval::Evaluated> {
        crate::eval::Evaluated::try_from(self)
    }
}

/// 语法分析器。
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// 当前 token。
    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    /// 向前看 n 个。
    fn peek_n(&self, n: usize) -> &Token {
        self.tokens.get(self.pos + n).unwrap_or(&Token::Eof)
    }

    /// 消费当前 token。
    fn bump(&mut self) -> Token {
        let t = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        t
    }

    /// 消费并匹配特定 token。
    fn eat(&mut self, expected: &Token) -> Result<()> {
        if std::mem::discriminant(self.peek()) == std::mem::discriminant(expected) {
            self.bump();
            Ok(())
        } else {
            Err(SassError::parse(format!("Expected {:?}, got {:?}", expected, self.peek())))
        }
    }

    /// 跳过空白 token（分号等）。
    fn skip_ws(&self) {
        // 在 SCSS 中分号是语句分隔符，不是空白
        // 这里不做任何跳过
    }

    /// 是否到达末尾。
    fn is_eof(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    /// 跳过静默注释（// ...）。
    fn skip_silent_comments(&mut self) {
        while matches!(self.peek(), Token::SilentComment(_)) {
            self.bump();
        }
    }

    /// 跳过注释（静默 + 块注释）。
    fn skip_comments(&mut self) {
        while matches!(self.peek(), Token::SilentComment(_) | Token::Comment(_)) {
            self.bump();
        }
    }

    /// 解析语句序列。
    pub fn parse_body(&mut self) -> Result<Vec<Node>> {
        let mut nodes = Vec::new();
        while !self.is_eof() && !matches!(self.peek(), Token::RBrace) {
            self.skip_silent_comments();
            if self.is_eof() || matches!(self.peek(), Token::RBrace) {
                break;
            }
            if let Some(node) = self.parse_statement()? {
                nodes.push(node);
            }
        }
        Ok(nodes)
    }

    /// 解析单条语句。
    fn parse_statement(&mut self) -> Result<Option<Node>> {
        match self.peek() {
            Token::Semicolon => { self.bump(); Ok(None) }
            Token::Comment(s) => {
                let s = s.clone();
                self.bump();
                Ok(Some(Node::Comment(s)))
            }
            Token::AtRule(_) => self.parse_at_rule().map(Some),
            Token::Variable(_) => variable::parse_variable_decl(self).map(Some),
            Token::LBrace => {
                Err(SassError::parse("Unexpected {"))
            }
            _ => self.parse_rule_or_decl().map(Some),
        }
    }

    /// 解析规则或声明（根据是否有 { 判断）。
    fn parse_rule_or_decl(&mut self) -> Result<Node> {
        rule::parse_rule_or_decl(self)
    }
}
