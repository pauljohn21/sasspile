//! 模块系统 @规则解析：@use / @forward / @import。

use super::Parser;
use super::ast::*;
use crate::error::{Result, SassError};
use crate::lex::token::Token;

impl<'tok> Parser<'tok> {
    pub(crate) fn parse_use(&mut self) -> Result<Node> {
        self.skip_ws();
        let url = self.parse_string_value()?;
        let mut namespace = None;
        let mut star = false;
        let mut config = Vec::new();
        self.skip_ws();
        if self.peek_keyword("as") {
            self.advance();
            self.skip_ws();
            if self.peek() == Some(&Token::Star) {
                self.advance();
                star = true;
            } else {
                namespace = Some(self.parse_ident_name()?);
            }
        }
        self.skip_ws();
        if self.peek_keyword("with") {
            self.advance();
            self.skip_ws();
            self.expect(&Token::LParen)?;
            config = self.parse_config()?;
        }
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Use {
            url,
            namespace,
            star,
            config,
        })
    }

    pub(crate) fn parse_forward(&mut self) -> Result<Node> {
        if self.in_body {
            return Err(SassError::Eval("This at-rule is not allowed here.".into()));
        }
        if self.saw_other_rule {
            return Err(SassError::Eval("@forward rules must be written before any other rules.".into()));
        }
        self.skip_ws();
        let url = match self.peek() {
            Some(Token::String(s, _)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return Err(SassError::Eval("Expected string.".into())),
        };
        let mut show = Vec::new();
        let mut hide = Vec::new();
        let mut prefix = None;
        self.skip_ws();
        if self.peek_keyword("as") {
            self.advance();
            self.skip_ws();
            match self.peek() {
                Some(Token::Ident(s)) => {
                    prefix = Some(s.clone());
                    self.advance();
                }
                _ => return Err(SassError::Eval("Expected identifier.".into())),
            }
            self.skip_ws();
            if self.peek() == Some(&Token::Star) {
                self.advance();
            } else {
                return Err(SassError::Eval("expected \"*\".".into()));
            }
            self.skip_ws();
        }
        if self.peek_keyword("show") {
            self.advance();
            show = self.parse_member_list()?;
            if show.is_empty() {
                return Err(SassError::Eval("Expected variable, mixin, or function name".into()));
            }
            self.skip_ws();
            if self.peek_keyword("hide") {
                return Err(SassError::Eval("expected \";\".".into()));
            }
        } else if self.peek_keyword("hide") {
            self.advance();
            hide = self.parse_member_list()?;
            if hide.is_empty() {
                return Err(SassError::Eval("Expected variable, mixin, or function name".into()));
            }
            self.skip_ws();
            if self.peek_keyword("show") {
                return Err(SassError::Eval("expected \";\".".into()));
            }
        }
        self.skip_ws();
        let mut config = Vec::new();
        if self.peek_keyword("with") {
            self.advance();
            self.skip_ws();
            self.expect(&Token::LParen)?;
            config = self.parse_config()?;
            self.skip_ws();
            if self.peek_keyword("as") || self.peek_keyword("show") || self.peek_keyword("hide") {
                return Err(SassError::Eval("expected \";\".".into()));
            }
        }
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Forward {
            url,
            show,
            hide,
            prefix,
            config,
        })
    }

    pub(crate) fn parse_import(&mut self) -> Result<Node> {
        let mut urls = Vec::new();
        loop {
            self.skip_ws();
            let url = self.parse_string_value()?;
            urls.push(url);
            self.skip_ws();
            if self.peek() == Some(&Token::Comma) {
                self.advance();
                continue;
            }
            break;
        }
        self.skip_ws();
        let modifier = if !matches!(self.peek(), Some(Token::Semicolon) | Some(Token::RBrace) | None) {
            let mut s = String::new();
            while let Some(t) = self.peek() {
                match t {
                    Token::Semicolon | Token::LBrace | Token::RBrace | Token::Eof => break,
                    Token::Comment(_, _) => { self.advance(); }
                    Token::Whitespace => { s.push(' '); self.advance(); }
                    _ => { s.push_str(&t.to_string()); self.advance(); }
                }
            }
            s.trim().to_string()
        } else {
            String::new()
        };
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        let url = if urls.len() == 1 {
            urls.into_iter().next().expect("urls has exactly 1 element")
        } else {
            urls.iter().map(|u| format!("\"{u}\"")).collect::<Vec<_>>().join("\", \"")
        };
        Ok(Node::Import { url, modifier })
    }
}
