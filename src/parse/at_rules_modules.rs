//! 模块系统 @规则解析：@use / @forward / @import。

use super::Parser;
use super::ast::*;
use crate::error::{Result, SassError};
use crate::lex::token::Token;

impl Parser<'_> {
    pub(crate) fn parse_use(&mut self) -> Result<Node> {
        match (self.in_body, self.saw_other_rule) {
            (true, _) => return Err(SassError::Eval("This at-rule is not allowed here.".into())),
            (false, true) => return Err(SassError::Eval(
                "@use rules must be written before any other rules.".into(),
            )),
            (false, false) => {}
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
        let mut namespace = None;
        let mut star = false;
        let mut config = Vec::new();
        self.skip_ws();
        match self.peek_keyword("as") {
            true => {
                self.advance();
                self.skip_ws();
                match self.peek() {
                    Some(&Token::Star) => {
                        self.advance();
                        star = true;
                    }
                    _ => namespace = Some(self.parse_ident_name()?),
                }
            }
            false => {}
        }
        self.skip_ws();
        match self.peek_keyword("with") {
            true => {
                self.advance();
                self.skip_ws();
                self.expect(&Token::LParen)?;
                config = self.parse_config(false)?;
            }
            false => {}
        }
        self.skip_ws();
        match self.peek() {
            Some(&Token::Semicolon) => {
                self.advance();
            }
            _ => {}
        }
        Ok(Node::Use {
            url,
            namespace,
            star,
            config,
        })
    }

    pub(crate) fn parse_forward(&mut self) -> Result<Node> {
        match (self.in_body, self.saw_other_rule) {
            (true, _) => return Err(SassError::Eval("This at-rule is not allowed here.".into())),
            (false, true) => return Err(SassError::Eval(
                "@forward rules must be written before any other rules.".into(),
            )),
            (false, false) => {}
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
        match self.peek_keyword("as") {
            true => {
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
                match self.peek() {
                    Some(&Token::Star) => {
                        self.advance();
                    }
                    _ => return Err(SassError::Eval("expected \"*\".".into())),
                }
                self.skip_ws();
            }
            false => {}
        }
        match (self.peek_keyword("show"), self.peek_keyword("hide")) {
            (true, _) => {
                self.advance();
                show = self.parse_member_list()?;
                match show.is_empty() {
                    true => return Err(SassError::Eval(
                        "Expected variable, mixin, or function name".into(),
                    )),
                    false => {}
                }
                self.skip_ws();
                match self.peek_keyword("hide") {
                    true => return Err(SassError::Eval("expected \";\".".into())),
                    false => {}
                }
            }
            (false, true) => {
                self.advance();
                hide = self.parse_member_list()?;
                match hide.is_empty() {
                    true => return Err(SassError::Eval(
                        "Expected variable, mixin, or function name".into(),
                    )),
                    false => {}
                }
                self.skip_ws();
                match self.peek_keyword("show") {
                    true => return Err(SassError::Eval("expected \";\".".into())),
                    false => {}
                }
            }
            _ => {}
        }
        self.skip_ws();
        let mut config = Vec::new();
        match self.peek_keyword("with") {
            true => {
                self.advance();
                self.skip_ws();
                self.expect(&Token::LParen)?;
                config = self.parse_config(true)?;
                self.skip_ws();
                match self.peek_keyword("as") || self.peek_keyword("show") || self.peek_keyword("hide") {
                    true => return Err(SassError::Eval("expected \";\".".into())),
                    false => {}
                }
            }
            false => {}
        }
        self.skip_ws();
        match self.peek() {
            Some(&Token::Semicolon) => {
                self.advance();
            }
            _ => {}
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
            self.skip_ws_and_comments();
            // 支持 url() 函数形式
            let url = match self.peek_keyword("url") {
                true => {
                    self.advance();
                    self.skip_ws();
                    match self.peek() {
                        Some(&Token::LParen) => {
                            self.advance();
                            let mut url_content = String::new();
                            while let Some(t) = self.peek() {
                                match t {
                                    Token::RParen => { self.advance(); break; }
                                    Token::Whitespace => { url_content.push(' '); self.advance(); }
                                    Token::Comment(_, _) => { self.advance(); }
                                    _ => { url_content.push_str(&t.to_string()); self.advance(); }
                                }
                            }
                            url_content.trim().to_string()
                        }
                        _ => return Err(SassError::Parse {
                            expected: "(".into(),
                            found: "other".into(),
                        }),
                    }
                }
                false => self.parse_string_value()?,
            };
            urls.push(url);
            self.skip_ws_and_comments();
            match self.peek() {
                Some(&Token::Comma) => { self.advance(); continue; }
                _ => break,
            }
        }
        self.skip_ws_and_comments();
        let modifier = match self.peek() {
            Some(Token::Semicolon | Token::RBrace) | None => String::new(),
            _ => {
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
            }
        };
        self.skip_ws_and_comments();
        match self.peek() {
            Some(&Token::Semicolon) => { self.advance(); }
            _ => {}
        }
        let url = match urls.len() {
            1 => urls.into_iter().next().expect("urls has exactly 1 element"),
            _ => urls
                .iter()
                .map(|u| format!("\"{u}\""))
                .collect::<Vec<_>>()
                .join("\", \""),
        };
        Ok(Node::Import { url, modifier })
    }
}
