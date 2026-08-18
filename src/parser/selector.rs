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
                Token::LParen => {
                    selector.push('(');
                    self.advance();
                }
                Token::RParen => {
                    selector.push(')');
                    self.advance();
                }
                Token::Star => {
                    selector.push('*');
                    self.advance();
                }
                Token::Percent => {
                    selector.push('%');
                    self.advance();
                }
                Token::Minus => {
                    selector.push('-');
                    self.advance();
                }
                Token::Comma => {
                    selector.push_str(", ");
                    self.advance();
                }
                Token::Gt => {
                    selector.push_str(" > ");
                    self.advance();
                }
                Token::Plus => {
                    selector.push_str(" + ");
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
                Token::SingleEq => {
                    selector.push('=');
                    self.advance();
                }
                Token::String(s, quote) => {
                    selector.push(*quote);
                    selector.push_str(s);
                    selector.push(*quote);
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
                Token::LineComment(_) | Token::BlockComment(_) => {
                    self.advance();
                }
                _ => break,
            }
        }

        Ok(selector.trim().to_string())
    }

    /// Check if current position starts a declaration (ident: or #{...}: or --name:)
    pub fn is_declaration_start(&self) -> bool {
        let mut i = self.pos;

        // Case 0: CSS custom property --name: or --#{...}: value
        if matches!(self.tokens.get(i), Some(ts) if matches!(ts.token, Token::Minus)) {
            let next_idx = i + 1;
            if let Some(ts) = self.tokens.get(next_idx) {
                if matches!(ts.token, Token::Minus | Token::InterpolationStart) {
                    // Scan forward until we find a Colon (property separator)
                    // or LBrace (nested rule) / Semicolon / RBrace
                    i = next_idx + 1;
                    if matches!(ts.token, Token::Minus) {
                        // Skip past second minus
                    } else {
                        // InterpolationStart — skip to RBrace
                        while let Some(t) = self.tokens.get(i) {
                            if matches!(t.token, Token::RBrace | Token::Eof) { break; }
                            i += 1;
                        }
                        if let Some(t) = self.tokens.get(i) {
                            if matches!(t.token, Token::RBrace) { i += 1; }
                        }
                    }
                    // Now scan for Colon
                    while let Some(t) = self.tokens.get(i) {
                        match &t.token {
                            Token::Colon => return true,
                            Token::LBrace | Token::Semicolon | Token::RBrace | Token::Eof => return false,
                            Token::InterpolationStart => {
                                i += 1;
                                while let Some(t2) = self.tokens.get(i) {
                                    if matches!(t2.token, Token::RBrace | Token::Eof) { break; }
                                    i += 1;
                                }
                                if let Some(t2) = self.tokens.get(i) {
                                    if matches!(t2.token, Token::RBrace) { i += 1; }
                                }
                            }
                            _ => { i += 1; }
                        }
                    }
                    return false;
                }
            }
        }

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

        // Case 2: Ident followed by Colon (possibly with interpolation or additional idents)
        if let Some(ts) = self.tokens.get(i) {
            if matches!(ts.token, Token::Ident(_)) {
                i += 1;
                // Scan forward through idents and interpolations until we find Colon or something else
                while let Some(ts) = self.tokens.get(i) {
                    match &ts.token {
                        Token::InterpolationStart => {
                            i += 1;
                            while let Some(t) = self.tokens.get(i) {
                                if matches!(t.token, Token::RBrace | Token::Eof) { break; }
                                i += 1;
                            }
                            if let Some(t) = self.tokens.get(i) {
                                if matches!(t.token, Token::RBrace) { i += 1; }
                            }
                        }
                        Token::Ident(_) | Token::Minus => { i += 1; }
                        Token::Colon => return true,
                        _ => break,
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
            Token::Ident(s) => {
                let mut name = s.clone();
                // Property name may contain interpolations (e.g. --#{$prefix}-color)
                // or additional ident parts after interpolation
                loop {
                    match self.peek().clone() {
                        Token::InterpolationStart => {
                            self.advance();
                            name.push_str("#{");
                            while !matches!(self.peek(), Token::RBrace | Token::Eof) {
                                match self.peek().clone() {
                                    Token::Variable(v) => {
                                        name.push('$');
                                        name.push_str(&v);
                                    }
                                    Token::Ident(s) => name.push_str(&s),
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
                        }
                        Token::Ident(s) => {
                            name.push_str(&s);
                            self.advance();
                        }
                        _ => break,
                    }
                }
                name
            }
            Token::Minus => {
                // CSS custom property: --name or --#{...}
                let mut name = String::from("-");
                if matches!(self.peek(), Token::Minus) {
                    self.advance();
                    name.push('-');
                }
                // Read remaining property name (idents, interpolations, etc.)
                loop {
                    match self.peek().clone() {
                        Token::Ident(s) => {
                            name.push_str(&s);
                            self.advance();
                        }
                        Token::InterpolationStart => {
                            self.advance();
                            name.push_str("#{");
                            while !matches!(self.peek(), Token::RBrace | Token::Eof) {
                                match self.peek().clone() {
                                    Token::Variable(v) => {
                                        name.push('$');
                                        name.push_str(&v);
                                    }
                                    Token::Ident(s) => name.push_str(&s),
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
                        }
                        Token::Number(v, u) => {
                            name.push_str(&format!("{}{}", v, u.as_deref().unwrap_or("")));
                            self.advance();
                        }
                        _ => break,
                    }
                }
                name
            }
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
