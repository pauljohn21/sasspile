//! 语法分析器——递归下降 + Pratt 表达式解析。

pub mod ast;
mod ast_impl;

use ast::*;
use crate::error::{Result, SassError};
use crate::lex::token::Token;
use tracing::{instrument, trace, warn};

/// 语法分析器。
pub struct Parser<'tok> {
    tokens: &'tok [Token],
    pos: usize,
}

impl<'tok> Parser<'tok> {
    /// 创建新的 Parser。
    pub fn new(tokens: &'tok [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    /// 解析入口。
    pub fn parse(tokens: &'tok [Token]) -> Result<Ast> {
        let mut p = Self::new(tokens);
        let mut nodes = Vec::new();
        while !p.at_end() {
            p.skip_ws();
            if p.at_end() {
                break;
            }
            nodes.push(p.parse_node()?);
        }
        Ok(Ast { nodes })
    }

    // —— 基础操作 ——
    fn peek(&self) -> Option<&Token> { self.tokens.get(self.pos) }
    fn peek_n(&self, n: usize) -> Option<&Token> { self.tokens.get(self.pos + n) }
    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        if !matches!(t, Some(Token::Eof) | None) { self.pos += 1; }
        t
    }
    fn at_end(&self) -> bool {
        let mut i = self.pos;
        while matches!(self.tokens.get(i), Some(Token::Whitespace)) { i += 1; }
        matches!(self.tokens.get(i), None | Some(Token::Eof))
    }
    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(Token::Whitespace) | Some(Token::Comment(_, _))) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, tok: &Token) -> Result<()> {
        self.skip_ws();
        if self.peek() == Some(tok) {
            self.advance();
            Ok(())
        } else {
            let found = self.peek().map(|t| t.to_string()).unwrap_or("EOF".into());
            warn!(expected = %tok, found = %found, pos = self.pos, "expect failed");
            Err(SassError::Parse {
                expected: tok.to_string(),
                found,
            })
        }
    }
}

mod nodes;
mod at_rules;
mod expr;
