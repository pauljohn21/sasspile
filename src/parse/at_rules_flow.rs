//! @规则解析 —— 流控制（@if/@for/@each/@while）。
//!
//! 从 `at_rules.rs` 拆分，包含所有流控制相关解析方法。

use super::ParseStream;
use super::ast::*;
use crate::error::{Result, SassError};
use crate::lex::token::Token;

impl<'tok> ParseStream<'tok> {
    pub(crate) fn parse_if(&mut self) -> Result<Node> {
        self.skip_ws();
        let cond = self.parse_value()?;
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        let mut branches = vec![(cond, body)];
        let mut else_body = None;
        // @else if / @else
        loop {
            self.skip_ws();
            match self.peek() {
                Some(Token::AtRule(n)) if n == "else" => {}
                _ => break,
            }
            self.advance(); // 消费 @else
            self.skip_ws();
            // @else if —— if 可能是 AtRule("if") 或 Ident("if")
            let is_else_if = match self.peek() {
                Some(Token::AtRule(n)) if n == "if" => true,
                Some(Token::Ident(n)) if n == "if" => true,
                _ => false,
            };
            match is_else_if {
                true => {
                self.advance(); // 消费 if
                self.skip_ws();
                let cond2 = self.parse_value()?;
                self.skip_ws();
                self.expect(&Token::LBrace)?;
                let body2 = self.parse_body()?;
                branches.push((cond2, body2));
                }
                false => {
                    self.skip_ws();
                    self.expect(&Token::LBrace)?;
                    else_body = Some(self.parse_body()?);
                    break;
                }
            }
        }
        Ok(Node::If {
            branches,
            else_body,
        })
    }

    pub(crate) fn parse_for(&mut self) -> Result<Node> {
        self.skip_ws();
        let var = match self.peek() {
            Some(Token::Dollar(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => {
                return Err(SassError::Parse {
                    expected: "$var".into(),
                    found: "other".into(),
                });
            }
        };
        self.skip_ws();
        self.expect_keyword("from")?;
        self.skip_ws();
        let from = self.parse_value()?;
        self.skip_ws();
        let inclusive = match self.peek_keyword("through") {
            true => {
                self.advance();
                true
            }
            false => {
                self.expect_keyword("to")?;
                false
            }
        };
        self.skip_ws();
        let to = self.parse_value()?;
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        Ok(Node::For {
            var,
            from,
            to,
            inclusive,
            body,
        })
    }

    pub(crate) fn parse_each(&mut self) -> Result<Node> {
        self.skip_ws();
        let mut vars = Vec::new();
        loop {
            self.skip_ws();
            match self.peek() {
                Some(Token::Dollar(n)) => {
                    vars.push(n.clone());
                    self.advance();
                }
                _ => break,
            }
            self.skip_ws();
            match self.peek() {
                Some(Token::Comma) => { self.advance(); }
                _ => break,
            }
        }
        self.skip_ws();
        self.expect_keyword("in")?;
        self.skip_ws();
        let list = self.parse_value()?;
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        Ok(Node::Each { vars, list, body })
    }

    pub(crate) fn parse_while(&mut self) -> Result<Node> {
        self.skip_ws();
        let cond = self.parse_value()?;
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        Ok(Node::While { cond, body })
    }
}
