//! Selector and declaration parsing — handles CSS selector strings and property declarations.

use crate::ast::*;
use crate::error::SassError;
use crate::token::Token;
use super::Parser;
use super::expr::ExprParser;

impl Parser {
    /// Parse a selector string (everything up to `{`).
    pub fn parse_selector(&mut self) -> Result<String, SassError> {
        let mut selector = String::new();

        while !self.is_at_end() && !matches!(self.peek(), Token::LBrace) {
            match self.peek() {
                Token::Dot => {
                    selector.push('.');
                    self.advance();
                }
                Token::Hash => {
                    selector.push('#');
                    self.advance();
                }
                Token::Ampersand => {
                    selector.push('&');
                    self.advance();
                }
                Token::Ident(s) => {
                    selector.push_str(s);
                    self.advance();
                }
                Token::Number(v, u) => {
                    selector.push_str(&format!("{}{}", v, u.as_deref().unwrap_or("")));
                    self.advance();
                }
                Token::Colon => {
                    selector.push(':');
                    self.advance();
                }
                Token::Comma => {
                    selector.push_str(", ");
                    self.advance();
                }
                Token::LBracket => {
                    selector.push('[');
                    self.advance();
                }
                Token::RBracket => {
                    selector.push(']');
                    self.advance();
                }
                Token::InterpolationStart => {
                    // Preserve the full #{...} interpolation syntax for the evaluator
                    selector.push_str("#{");
                    self.advance();
                    while !matches!(self.peek(), Token::RBrace | Token::Eof) {
                        match self.peek() {
                            Token::Variable(name) => {
                                selector.push('$');
                                selector.push_str(name);
                            }
                            Token::Ident(s) => {
                                selector.push_str(s);
                            }
                            Token::Number(v, u) => {
                                selector.push_str(&format!("{}{}", v, u.as_deref().unwrap_or("")));
                            }
                            Token::Plus => selector.push('+'),
                            Token::Minus => selector.push('-'),
                            Token::Star => selector.push('*'),
                            Token::Slash => selector.push('/'),
                            Token::Percent => selector.push('%'),
                            Token::LParen => selector.push('('),
                            Token::RParen => selector.push(')'),
                            Token::Comma => selector.push_str(", "),
                            Token::Dot => selector.push('.'),
                            Token::Colon => selector.push(':'),
                            _ => {}
                        }
                        self.advance();
                    }
                    if matches!(self.peek(), Token::RBrace) {
                        selector.push('}');
                        self.advance();
                    }
                }
                _ => break,
            }
        }

        Ok(selector.trim().to_string())
    }

    /// Check if current position starts a declaration (ident: or #{...}:)
    pub fn is_declaration_start(&self) -> bool {
        let mut i = self.pos;

        // Case 1: Interpolation as property name: #{...}: value
        if matches!(self.tokens.get(i), Some(ts) if matches!(ts.token, Token::InterpolationStart)) {
            i += 1;
            while let Some(ts) = self.tokens.get(i) {
                if matches!(ts.token, Token::RBrace | Token::Eof) { break; }
                i += 1;
            }
            if let Some(ts) = self.tokens.get(i) {
                if matches!(ts.token, Token::RBrace) {
                    i += 1;
                    if let Some(next) = self.tokens.get(i) {
                        return matches!(next.token, Token::Colon);
                    }
                }
            }
            return false;
        }

        // Case 2: Ident followed by Colon (possibly with interpolation)
        if let Some(ts) = self.tokens.get(i) {
            if matches!(ts.token, Token::Ident(_)) {
                i += 1;
                while let Some(ts) = self.tokens.get(i) {
                    if matches!(ts.token, Token::InterpolationStart) {
                        i += 1;
                        while let Some(t) = self.tokens.get(i) {
                            if matches!(t.token, Token::RBrace | Token::Eof) { break; }
                            i += 1;
                        }
                        if let Some(t) = self.tokens.get(i) {
                            if matches!(t.token, Token::RBrace) { i += 1; }
                        }
                    } else {
                        break;
                    }
                }
                if let Some(next) = self.tokens.get(i) {
                    return matches!(next.token, Token::Colon);
                }
            }
        }
        false
    }

    /// Parse a declaration: `property: value;`
    pub fn parse_declaration(&mut self) -> Result<Stmt, SassError> {
        let pos = self.current_pos();
        let property = match self.advance() {
            Token::Ident(s) => s.clone(),
            Token::InterpolationStart => {
                let mut name = String::from("#{");
                while !matches!(self.peek(), Token::RBrace | Token::Eof) {
                    match self.peek() {
                        Token::Variable(v) => {
                            name.push('$');
                            name.push_str(v);
                        }
                        Token::Ident(s) => {
                            name.push_str(s);
                        }
                        Token::Number(v, u) => {
                            name.push_str(&format!("{}{}", v, u.as_deref().unwrap_or("")));
                        }
                        Token::Minus => name.push('-'),
                        Token::Plus => name.push('+'),
                        Token::Dot => name.push('.'),
                        _ => {}
                    }
                    self.advance();
                }
                if matches!(self.peek(), Token::RBrace) {
                    name.push('}');
                    self.advance();
                }
                name
            }
            _ => return Err(SassError::parse("expected property name", pos)),
        };

        if matches!(self.peek(), Token::LBrace) {
            self.advance();
            let body = self.parse_block_body()?;
            self.expect(Token::RBrace)?;
            let mut stmts = vec![Stmt::Declaration {
                property: format!("{}-", property),
                value: Expr::Literal(crate::value::Value::Null),
            }];
            stmts.extend(body);
            return Ok(stmts.remove(0));
        }

        self.expect(Token::Colon)?;

        let mut expr_parser = ExprParser::new(self);
        let value = expr_parser.parse_expr()?;

        if matches!(self.peek(), Token::Semicolon) {
            self.advance();
        }

        Ok(Stmt::Declaration { property, value })
    }
}
