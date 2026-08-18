//! At-rule parser — handles all @-rules.

use crate::ast::*;
use crate::error::SassError;
use crate::token::Token;
use super::Parser;
use super::expr::ExprParser;

impl Parser {
    pub fn parse_if(&mut self) -> Result<Stmt, SassError> {
        let cond = {
            let mut ep = ExprParser::new(self);
            ep.parse_expr()?
        };
        self.expect(Token::LBrace)?;
        let body = self.parse_block_body()?;
        self.expect(Token::RBrace)?;

        let mut branches = vec![(cond, body)];
        let mut else_body = None;

// Check for @else if / @else
loop {
// Peek for @else
let is_else = matches!(self.peek(), Token::AtRule(n) if n == "else");
if !is_else { break; }
self.advance(); // @else

// @else if — "if" is an Ident (not AtRule) after @else
if matches!(self.peek(), Token::Ident(s) if s == "if") {
self.advance(); // if
let cond = {
let mut ep = ExprParser::new(self);
ep.parse_expr()?
};
self.expect(Token::LBrace)?;
let body = self.parse_block_body()?;
self.expect(Token::RBrace)?;
branches.push((cond, body));
} else {
                // @else
                self.expect(Token::LBrace)?;
                let body = self.parse_block_body()?;
                self.expect(Token::RBrace)?;
                else_body = Some(body);
                break;
            }
        }

        Ok(Stmt::IfStmt { branches, else_body })
    }

    pub fn parse_for(&mut self) -> Result<Stmt, SassError> {
        let var = match self.advance() {
            Token::Variable(v) => v.clone(),
            t => return Err(SassError::parse(
                format!("expected variable after @for, got {:?}", t),
                self.current_pos(),
            )),
        };
        // expect "from"
        if let Token::Ident(s) = self.advance() {
            if s != "from" {
                return Err(SassError::parse("expected 'from' in @for", self.current_pos()));
            }
        }
        let from = {
            let mut ep = ExprParser::new(self);
            ep.parse_expr()?
        };
        // expect "through" or "to"
        let exclusive = match self.advance() {
            Token::Ident(s) if s == "through" => false,
            Token::Ident(s) if s == "to" => true,
            t => return Err(SassError::parse(
                format!("expected 'through' or 'to', got {:?}", t),
                self.current_pos(),
            )),
        };
        let to = {
            let mut ep = ExprParser::new(self);
            ep.parse_expr()?
        };
        self.expect(Token::LBrace)?;
        let body = self.parse_block_body()?;
        self.expect(Token::RBrace)?;
        Ok(Stmt::ForStmt { var, from, to, exclusive, body })
    }

    pub fn parse_each(&mut self) -> Result<Stmt, SassError> {
        let mut vars = Vec::new();
        loop {
            match self.advance() {
                Token::Variable(v) => vars.push(v.clone()),
                t => return Err(SassError::parse(
                    format!("expected variable in @each, got {:?}", t),
                    self.current_pos(),
                )),
            }
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        // expect "in"
        if let Token::Ident(s) = self.advance() {
            if s != "in" {
                return Err(SassError::parse("expected 'in' in @each", self.current_pos()));
            }
        }
        let list = {
            let mut ep = ExprParser::new(self);
            ep.parse_expr()?
        };
        self.expect(Token::LBrace)?;
        let body = self.parse_block_body()?;
        self.expect(Token::RBrace)?;
        Ok(Stmt::EachStmt { vars, list, body })
    }

    pub fn parse_while(&mut self) -> Result<Stmt, SassError> {
        let cond = {
            let mut ep = ExprParser::new(self);
            ep.parse_expr()?
        };
        self.expect(Token::LBrace)?;
        let body = self.parse_block_body()?;
        self.expect(Token::RBrace)?;
        Ok(Stmt::WhileStmt { cond, body })
    }

    pub fn parse_mixin_def(&mut self) -> Result<Stmt, SassError> {
        let name = match self.advance() {
            Token::Ident(n) => n.clone(),
            t => return Err(SassError::parse(
                format!("expected mixin name, got {:?}", t),
                self.current_pos(),
            )),
        };
        let params = if matches!(self.peek(), Token::LParen) {
            self.advance();
            let p = self.parse_params()?;
            self.expect(Token::RParen)?;
            p
        } else {
            Vec::new()
        };
        self.expect(Token::LBrace)?;
        let body = self.parse_block_body()?;
        self.expect(Token::RBrace)?;
        Ok(Stmt::MixinDef { name, params, body })
    }

    pub fn parse_include(&mut self) -> Result<Stmt, SassError> {
        let name = match self.advance() {
            Token::Ident(n) => n.clone(),
            t => return Err(SassError::parse(
                format!("expected mixin name, got {:?}", t),
                self.current_pos(),
            )),
        };
        let args = if matches!(self.peek(), Token::LParen) {
            self.advance();
            let mut ep = ExprParser::new(self);
            let a = ep.parse_args()?;
            self.expect(Token::RParen)?;
            a
        } else {
            Vec::new()
        };
        // Optional @content block
        let content = if matches!(self.peek(), Token::LBrace) {
            self.advance();
            let body = self.parse_block_body()?;
            self.expect(Token::RBrace)?;
            Some(body)
        } else {
            if matches!(self.peek(), Token::Semicolon) {
                self.advance();
            }
            None
        };
        Ok(Stmt::IncludeCall { name, args, content })
    }

    pub fn parse_function_def(&mut self) -> Result<Stmt, SassError> {
        let name = match self.advance() {
            Token::Ident(n) => n.clone(),
            t => return Err(SassError::parse(
                format!("expected function name, got {:?}", t),
                self.current_pos(),
            )),
        };
        self.expect(Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;
        let body = self.parse_block_body()?;
        self.expect(Token::RBrace)?;
        Ok(Stmt::FunctionDef { name, params, body })
    }

    pub fn parse_return(&mut self) -> Result<Stmt, SassError> {
        let value = {
            let mut ep = ExprParser::new(self);
            ep.parse_expr()?
        };
        if matches!(self.peek(), Token::Semicolon) {
            self.advance();
        }
        Ok(Stmt::ReturnStmt(value))
    }

    pub fn parse_error_stmt(&mut self) -> Result<Stmt, SassError> {
        let value = {
            let mut ep = ExprParser::new(self);
            ep.parse_expr()?
        };
        if matches!(self.peek(), Token::Semicolon) { self.advance(); }
        Ok(Stmt::ErrorStmt(value))
    }

    pub fn parse_warn_stmt(&mut self) -> Result<Stmt, SassError> {
        let value = {
            let mut ep = ExprParser::new(self);
            ep.parse_expr()?
        };
        if matches!(self.peek(), Token::Semicolon) { self.advance(); }
        Ok(Stmt::WarnStmt(value))
    }

    pub fn parse_debug_stmt(&mut self) -> Result<Stmt, SassError> {
        let value = {
            let mut ep = ExprParser::new(self);
            ep.parse_expr()?
        };
        if matches!(self.peek(), Token::Semicolon) { self.advance(); }
        Ok(Stmt::DebugStmt(value))
    }

    pub fn parse_at_root(&mut self) -> Result<Stmt, SassError> {
        // @at-root can have an optional selector or query before {
        // e.g. @at-root #{$rule-name} { ... } or @at-root (without: ...) { ... }
        if matches!(self.peek(), Token::LParen) {
            self.advance();
            // Skip the query content (e.g. `(without: media)`)
            while !matches!(self.peek(), Token::RParen | Token::Eof) {
                self.advance();
            }
            self.expect(Token::RParen)?;
        }
        // Read optional selector until { (e.g. #{$rule-name} or #{&}#{':#{$pseudo}'})
        let selector = if !matches!(self.peek(), Token::LBrace) {
            let s = self.read_until_brace();
            if s.trim().is_empty() { None } else { Some(s) }
        } else {
            None
        };
        self.expect(Token::LBrace)?;
        let body = self.parse_block_body()?;
        self.expect(Token::RBrace)?;
        // If there's a selector, we need to wrap the body in a style rule.
        // For now, just store as AtRootRule and handle selector in eval.
        if let Some(sel) = selector {
            // Prepend a style rule with the selector to the body
            let mut new_body = Vec::new();
            new_body.push(Stmt::StyleRule { selector: sel, body });
            Ok(Stmt::AtRootRule(new_body))
        } else {
            Ok(Stmt::AtRootRule(body))
        }
    }

    pub fn parse_media(&mut self) -> Result<Stmt, SassError> {
        let query = self.read_until_brace();
        self.expect(Token::LBrace)?;
        let body = self.parse_block_body()?;
        self.expect(Token::RBrace)?;
        Ok(Stmt::MediaRule { query, body })
    }

    pub fn parse_supports(&mut self) -> Result<Stmt, SassError> {
        let condition = self.read_until_brace();
        self.expect(Token::LBrace)?;
        let body = self.parse_block_body()?;
        self.expect(Token::RBrace)?;
        Ok(Stmt::SupportsRule { condition, body })
    }

    pub fn parse_use(&mut self) -> Result<Stmt, SassError> {
        let url = self.read_string_or_ident();
        let mut namespace = None;
        let mut config = Vec::new();

        // Check for "as" clause
        if matches!(self.peek(), Token::Ident(s) if s == "as") {
            self.advance();
            if let Token::Ident(n) = self.advance() {
                if n != "*" {
                    namespace = Some(n.clone());
                }
            }
        } else if !url.starts_with("sass:") {
            // Default namespace: last component of URL path
            // e.g. "./config" → "config", "../mixins/function" → "function"
            let ns = url
                .trim_start_matches("./")
                .trim_start_matches("../")
                .rsplit('/')
                .next()
                .unwrap_or(&url)
                .trim_end_matches(".scss")
                .trim_end_matches(".css")
                .to_string();
            if !ns.is_empty() {
                namespace = Some(ns);
            }
        }

        // Check for "with" clause
        if matches!(self.peek(), Token::Ident(s) if s == "with") {
            self.advance();
            self.expect(Token::LParen)?;
            while !matches!(self.peek(), Token::RParen | Token::Eof) {
                let var = match self.advance() {
                    Token::Variable(v) => v.clone(),
                    _ => return Err(SassError::parse("expected variable in @use with", self.current_pos())),
                };
                self.expect(Token::Colon)?;
                let val = {
                    let mut ep = ExprParser::new(self);
                    ep.parse_expr()?
                };
                config.push((var, val));
                if matches!(self.peek(), Token::Comma) { self.advance(); }
            }
            self.expect(Token::RParen)?;
        }

        if matches!(self.peek(), Token::Semicolon) { self.advance(); }
        Ok(Stmt::UseRule { url, namespace, config })
    }

    pub fn parse_forward(&mut self) -> Result<Stmt, SassError> {
        let url = self.read_string_or_ident();
        let mut show = None;
        let mut hide = None;

        if matches!(self.peek(), Token::Ident(s) if s == "show") {
            self.advance();
            show = Some(self.read_member_list());
        } else if matches!(self.peek(), Token::Ident(s) if s == "hide") {
            self.advance();
            hide = Some(self.read_member_list());
        }

        if matches!(self.peek(), Token::Semicolon) { self.advance(); }
        Ok(Stmt::ForwardRule { url, show, hide })
    }

    pub fn parse_import(&mut self) -> Result<Stmt, SassError> {
        let url = self.read_string_or_ident();
        if matches!(self.peek(), Token::Semicolon) { self.advance(); }
        Ok(Stmt::ImportRule(url))
    }

    pub fn parse_extend(&mut self) -> Result<Stmt, SassError> {
        let selector = self.read_until_semicolon();
        let optional = selector.ends_with("!optional");
        let selector = if optional {
            selector.trim_end_matches("!optional").trim().to_string()
        } else {
            selector
        };
        if matches!(self.peek(), Token::Semicolon) { self.advance(); }
        Ok(Stmt::ExtendRule { selector, optional })
    }

    pub fn parse_content(&mut self) -> Result<Stmt, SassError> {
        if matches!(self.peek(), Token::Semicolon) { self.advance(); }
        Ok(Stmt::ContentRule)
    }

    pub fn parse_css_at_rule(&mut self, name: &str) -> Result<Stmt, SassError> {
        let value = self.read_until_brace();
        let body = if matches!(self.peek(), Token::LBrace) {
            self.advance();
            let b = self.parse_block_body()?;
            self.expect(Token::RBrace)?;
            Some(b)
        } else {
            if matches!(self.peek(), Token::Semicolon) { self.advance(); }
            None
        };
        Ok(Stmt::CssAtRule {
            name: name.to_string(),
            value,
            body,
        })
    }

    /// Parse function/mixin parameters
    pub fn parse_params(&mut self) -> Result<Vec<Param>, SassError> {
        let mut params = Vec::new();
        while !matches!(self.peek(), Token::RParen | Token::Eof) {
            let name = match self.advance() {
                Token::Variable(v) => v.clone(),
                _ => return Err(SassError::parse("expected parameter name", self.current_pos())),
            };
// Check for rest args ...
if matches!(self.peek(), Token::Spread) {
    self.advance();
    params.push(Param { name, default: None, rest: true });
    if matches!(self.peek(), Token::Comma) { self.advance(); }
    continue;
}
    // Check for default value
    let default = if matches!(self.peek(), Token::Colon) {
        self.advance();
        let mut ep = ExprParser::new(self);
        Some(ep.parse_space_list()?)
    } else {
        None
    };
            params.push(Param { name, default, rest: false });
            if matches!(self.peek(), Token::Comma) { self.advance(); }
        }
        Ok(params)
    }
}
