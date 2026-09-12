//! CSS 解析器——仅解析原生 CSS + CSS Nesting。
//!
//! 无变量、无控制流、无 mixin、无函数、无 @extend、无 @at-root。
//! 与 SCSS Parser 结构平行，产出 CssAst。

use super::ast::Separator;
use super::css_ast::{CssArg, CssAst, CssNode, CssValue};
use crate::__tracing::trace;
use crate::error::{Result, SassError};
use crate::lex::token::Token;

/// CSS 语法分析器——仅 CSS 原生语法。
pub struct Parser<'tok> {
    tokens: &'tok [Token],
    pos: usize,
}

impl<'tok> Parser<'tok> {
    pub fn new(tokens: &'tok [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    /// 解析入口——解析完整 CSS 文件。
    pub fn parse(tokens: &'tok [Token]) -> Result<CssAst> {
        let mut p = Self::new(tokens);
        let mut nodes = Vec::new();
        while !p.at_end() {
            p.skip_ws();
            match p.at_end() {
                true => break,
                false => {}
            }
            let node = p.parse_node()?;
            nodes.push(node);
        }
        Ok(CssAst { nodes })
    }

    fn parse_node(&mut self) -> Result<CssNode> {
        self.skip_ws();
        let peek_str = self
            .peek()
            .map_or_else(|| "EOF".into(), std::string::ToString::to_string);
        trace!(peek = %peek_str, "css_parse_node");
        match self.peek() {
            Some(Token::AtRule(name)) => self.parse_at_rule(name.clone()),
            Some(Token::Comment(_, silent)) => {
                let is_silent = *silent;
                let text = match self.peek() {
                    Some(Token::Comment(t, _)) => t.clone(),
                    _ => String::new(),
                };
                self.advance();
                match is_silent {
                    true => self.parse_node(),
                    false => Ok(CssNode::Comment(text)),
                }
            }
            Some(Token::Semicolon) => {
                self.advance();
                self.skip_ws();
                self.parse_node()
            }
            Some(Token::Whitespace) => {
                self.advance();
                self.parse_node()
            }
            _ => self.parse_rule_or_decl(),
        }
    }

    fn parse_at_rule(&mut self, name: String) -> Result<CssNode> {
        self.advance();
        let params = self.parse_at_rule_params()?;
        self.skip_ws();
        match self.peek() {
            Some(Token::LBrace) => {
                self.advance();
                let body = self.parse_body()?;
                self.expect(&Token::RBrace)?;
                Ok(CssNode::AtRule {
                    name,
                    params,
                    body: Some(body),
                })
            }
            Some(Token::Semicolon) => {
                self.advance();
                Ok(CssNode::AtRule {
                    name,
                    params,
                    body: None,
                })
            }
            _ => Ok(CssNode::AtRule {
                name,
                params,
                body: None,
            }),
        }
    }

    fn parse_rule_or_decl(&mut self) -> Result<CssNode> {
        match self.is_rule() {
            true => self.parse_rule(),
            false => self.parse_decl(),
        }
    }

    fn is_rule(&self) -> bool {
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Token::LBrace => return true,
                Token::Semicolon | Token::RBrace => return false,
                Token::Whitespace => {
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }
        false
    }

    fn parse_rule(&mut self) -> Result<CssNode> {
        let selector = self.parse_selector_text()?;
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        self.skip_ws();
        self.expect(&Token::RBrace)?;
        Ok(CssNode::Rule { selector, body })
    }

    fn parse_decl(&mut self) -> Result<CssNode> {
        let property = self.parse_property_name()?;
        self.skip_ws();
        self.expect(&Token::Colon)?;
        self.skip_ws();
        let value = self.parse_decl_value()?;
        self.skip_ws();
        let important = self.parse_important()?;
        self.skip_ws();
        match self.peek() {
            Some(Token::Semicolon) => {
                self.advance();
            }
            _ => {}
        }
        Ok(CssNode::Decl {
            property,
            value,
            important,
        })
    }

    fn parse_body(&mut self) -> Result<Vec<CssNode>> {
        let mut body = Vec::new();
        self.skip_ws();
        while !self.at_end() && self.peek() != Some(&Token::RBrace) {
            self.skip_ws();
            match self.peek() {
                Some(Token::RBrace) | None | Some(Token::Eof) => break,
                Some(Token::Semicolon) => {
                    self.advance();
                    continue;
                }
                _ => {}
            }
            let node = self.parse_node()?;
            body.push(node);
            self.skip_ws();
        }
        Ok(body)
    }

    fn parse_decl_value(&mut self) -> Result<CssValue> {
        let first = self.parse_css_value_expr()?;
        self.skip_ws();
        match self.peek() {
            Some(&Token::Comma) => {
                let mut items = vec![first];
                while self.peek() == Some(&Token::Comma) {
                    self.advance();
                    self.skip_ws();
                    items.push(self.parse_css_value_expr()?);
                    self.skip_ws();
                }
                Ok(CssValue::List(items, Separator::Comma, false))
            }
            _ => Ok(first),
        }
    }

    fn parse_css_value_expr(&mut self) -> Result<CssValue> {
        self.skip_ws();
        match self.peek() {
            Some(Token::Number(n)) => {
                let n = n.clone();
                self.advance();
                Ok(CssValue::String(n, false))
            }
            Some(Token::String(s, _)) => {
                let s = s.clone();
                self.advance();
                Ok(CssValue::String(s, true))
            }
            Some(Token::Hash(s)) => {
                let s = s.clone();
                self.advance();
                Ok(CssValue::String(format!("#{s}"), false))
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                match self.peek() {
                    Some(Token::LParen) => self.parse_call(name),
                    _ => Ok(CssValue::String(name, false)),
                }
            }
            _ => {
                let found = self
                    .peek()
                    .map_or("EOF".into(), std::string::ToString::to_string);
                Err(SassError::Parse {
                    expected: "value".into(),
                    found,
                })
            }
        }
    }

    fn parse_call(&mut self, name: String) -> Result<CssValue> {
        self.advance(); // consume (
        self.skip_ws();
        let mut args = Vec::new();
        while self.peek() != Some(&Token::RParen) && !self.at_end() {
            let value = self.parse_css_value_expr()?;
            args.push(CssArg {
                name: None,
                value,
            });
            self.skip_ws();
            match self.peek() {
                Some(Token::Comma) => {
                    self.advance();
                    self.skip_ws();
                }
                _ => {}
            }
        }
        self.skip_ws();
        self.expect(&Token::RParen)?;
        Ok(CssValue::Call(name, args))
    }

    fn parse_at_rule_params(&mut self) -> Result<Option<String>> {
        let start = self.pos;
        let mut depth = 0u32;
        // Safety: 每次迭代必须前进 pos，否则可能无限循环（Rust 无 GC，不抛 StackOverflow）
        while !self.at_end() {
            match self.peek() {
                Some(Token::LBrace) | Some(Token::Semicolon) | None | Some(Token::Eof) => break,
                Some(Token::LParen) => {
                    depth += 1;
                    self.advance();
                }
                // depth==0 遇到 ')' 直接前进，避免 pos 停滞导致无限循环
                Some(Token::RParen) => {
                    if depth > 0 { depth -= 1; }
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
        let end = self.pos;
        match start == end {
            true => Ok(None),
            false => {
                let text: String = self.tokens[start..end]
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect();
                Ok(Some(text))
            }
        }
    }

    fn parse_selector_text(&mut self) -> Result<String> {
        let start = self.pos;
        let mut depth = 0u32;
        while !self.at_end() {
            match self.peek() {
                Some(Token::LBrace) => match depth == 0 {
                    true => break,
                    false => {
                        depth -= 1;
                        self.advance();
                    }
                },
                Some(Token::LParen) => {
                    depth += 1;
                    self.advance();
                }
                // depth==0 遇到 ')' 直接前进，避免 pos 停滞导致无限循环
                Some(Token::RParen) => {
                    if depth > 0 { depth -= 1; }
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
        let text: Vec<String> = self.tokens[start..self.pos]
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        Ok(text.join(""))
    }

    fn parse_property_name(&mut self) -> Result<String> {
        let mut name = String::new();
        while !self.at_end() {
            match self.peek() {
                Some(Token::Colon) => break,
                Some(Token::Whitespace) => {
                    self.advance();
                    continue;
                }
                Some(t) => {
                    name.push_str(&t.to_string());
                    self.advance();
                }
                None => break,
            }
        }
        Ok(name)
    }

    fn parse_important(&mut self) -> Result<bool> {
        let saved = self.pos;
        let mut found = false;
        while !self.at_end() {
            match self.peek() {
                Some(Token::Semicolon) | Some(Token::RBrace) | None | Some(Token::Eof) => break,
                Some(Token::Ident(s)) if s == "important" => {
                    self.advance();
                    found = true;
                    break;
                }
                Some(Token::Bang) => {
                    self.advance();
                    continue;
                }
                Some(Token::Whitespace) => {
                    self.advance();
                    continue;
                }
                _ => break,
            }
        }
        match found {
            true => Ok(true),
            false => {
                self.pos = saved;
                Ok(false)
            }
        }
    }

    // —— 基础操作 ——

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        match t {
            Some(Token::Eof) | None => {}
            _ => self.pos += 1,
        }
        t
    }

    fn at_end(&self) -> bool {
        matches!(self.peek(), None | Some(Token::Eof))
    }

    fn skip_ws(&mut self) {
        while let Some(tok) = self.peek() {
            match tok {
                Token::Whitespace => self.pos += 1,
                _ => break,
            }
        }
    }

    fn expect(&mut self, tok: &Token) -> Result<()> {
        self.skip_ws();
        match self.peek() == Some(tok) {
            true => {
                self.advance();
                Ok(())
            }
            false => {
                let found = self
                    .peek()
                    .map_or("EOF".into(), std::string::ToString::to_string);
                Err(SassError::Parse {
                    expected: tok.to_string(),
                    found,
                })
            }
        }
    }
}
