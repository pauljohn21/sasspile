//! Expression parser — handles operator precedence, function calls, lists, maps.

use crate::ast::*;
use crate::error::SassError;
use crate::token::Token;
use crate::value::Value;
use super::Parser;

mod args;

pub struct ExprParser<'a> {
    parser: &'a mut Parser,
}

impl<'a> ExprParser<'a> {
    pub fn new(parser: &'a mut Parser) -> Self {
        Self { parser }
    }

    /// Parse an expression with full operator precedence.
    /// Precedence: comma-list > space-list > or > and > equality > comparison > additive > multiplicative > unary > primary
    pub fn parse_expr(&mut self) -> Result<Expr, SassError> {
        let expr = self.parse_space_list()?;
        if matches!(self.parser.peek(), Token::Comma) {
            let mut items = vec![expr];
            while matches!(self.parser.peek(), Token::Comma) {
                self.parser.advance();
                if matches!(self.parser.peek(), Token::RParen | Token::RBrace | Token::Eof | Token::Semicolon) {
                    break;
                }
                items.push(self.parse_space_list()?);
            }
            return Ok(Expr::ListExpr {
                items,
                separator: ListSeparator::Comma,
                bracketed: false,
            });
        }
        Ok(expr)
    }

    /// Parse a space-separated list (used inside function args).
    pub fn parse_space_list(&mut self) -> Result<Expr, SassError> {
        self.skip_comments();
        let first = self.parse_or()?;
        self.skip_comments();
        if self.is_space_separated() {
            let mut items = vec![first];
            while self.is_space_separated() {
                self.skip_comments();
                items.push(self.parse_or()?);
                self.skip_comments();
            }
            return Ok(Expr::ListExpr {
                items,
                separator: ListSeparator::Space,
                bracketed: false,
            });
        }
        Ok(first)
    }

    /// Check if the next token could start a space-separated list item.
    fn is_space_separated(&self) -> bool {
        !matches!(self.parser.peek(),
            Token::Comma | Token::RParen | Token::RBrace | Token::LBrace | Token::RBracket | Token::Eof |
            Token::Semicolon | Token::Plus | Token::Minus | Token::Star |
            Token::Slash | Token::Percent | Token::Eq | Token::NotEq |
            Token::Lt | Token::LtEq | Token::Gt | Token::GtEq |
            Token::Colon | Token::Spread |
            Token::LineComment(_) | Token::BlockComment(_)
        ) && !matches!(self.parser.peek(), Token::Ident(s) if matches!(s.as_str(), "and" | "or" | "through" | "to" | "from" | "!default" | "!global"))
    }

    /// Skip any comments at the current position.
    fn skip_comments(&mut self) {
        while matches!(self.parser.peek(), Token::LineComment(_) | Token::BlockComment(_)) {
            self.parser.advance();
        }
    }

    fn parse_or(&mut self) -> Result<Expr, SassError> {
        let mut left = self.parse_and()?;
        while matches!(self.parser.peek(), Token::Ident(s) if s == "or") {
            self.parser.advance();
            let right = self.parse_and()?;
            left = Expr::Operation { op: BinOp::Or, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, SassError> {
        let mut left = self.parse_equality()?;
        while matches!(self.parser.peek(), Token::Ident(s) if s == "and") {
            self.parser.advance();
            let right = self.parse_equality()?;
            left = Expr::Operation { op: BinOp::And, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, SassError> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match self.parser.peek() {
                Token::Eq => BinOp::Eq,
                Token::NotEq => BinOp::NotEq,
                _ => break,
            };
            self.parser.advance();
            let right = self.parse_comparison()?;
            left = Expr::Operation { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, SassError> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.parser.peek() {
                Token::Lt => BinOp::Lt,
                Token::LtEq => BinOp::LtEq,
                Token::Gt => BinOp::Gt,
                Token::GtEq => BinOp::GtEq,
                _ => break,
            };
            self.parser.advance();
            let right = self.parse_additive()?;
            left = Expr::Operation { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, SassError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.parser.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.parser.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::Operation { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, SassError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.parser.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.parser.advance();
            let right = self.parse_unary()?;
            left = Expr::Operation { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, SassError> {
        match self.parser.peek() {
            Token::Minus => {
                self.parser.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Neg, operand: Box::new(operand) })
            }
            Token::Ident(s) if s == "not" => {
                self.parser.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Not, operand: Box::new(operand) })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, SassError> {
        // Skip comments in expressions
        while matches!(self.parser.peek(), Token::LineComment(_) | Token::BlockComment(_)) {
            self.parser.advance();
        }
        let pos = self.parser.current_pos();
        match self.parser.peek().clone() {
            Token::Number(v, u) => {
                self.parser.advance();
                Ok(Expr::Literal(Value::Number(crate::value::Number::new(v, u))))
            }
            Token::HexColor(v) => {
                self.parser.advance();
                let (r, g, b) = if v <= 0xfff {
                    let r = (v >> 8) & 0xF;
                    let g = (v >> 4) & 0xF;
                    let bl = v & 0xF;
                    ((r | (r << 4)) as f64, (g | (g << 4)) as f64, (bl | (bl << 4)) as f64)
                } else {
                    (((v >> 16) & 0xFF) as f64, ((v >> 8) & 0xFF) as f64, (v & 0xFF) as f64)
                };
                Ok(Expr::Literal(Value::Color(crate::value::Color::rgb(r, g, b, 1.0))))
            }
            Token::String(s, _) => {
                self.parser.advance();
                Ok(Expr::Literal(Value::String(crate::value::SassString::quoted(s))))
            }
            Token::Variable(name) => {
                self.parser.advance();
                Ok(Expr::Variable(name))
            }
            Token::Ident(name) => {
                self.parser.advance();
                // Check for namespace-qualified variable: ident.$var (e.g. config.$modifier-separator)
                if matches!(self.parser.peek(), Token::Dot) {
                    let next_idx = self.parser.pos + 1;
                    let is_namespaced_var = self.parser.tokens.get(next_idx)
                        .map(|ts| matches!(&ts.token, Token::Variable(_)))
                        .unwrap_or(false);
                    if is_namespaced_var {
                        let var_name = if let Some(ts) = self.parser.tokens.get(next_idx) {
                            if let Token::Variable(ref vn) = ts.token { vn.clone() } else { String::new() }
                        } else { String::new() };
                        self.parser.advance(); // Dot
                        self.parser.advance(); // Variable
                        return Ok(Expr::NamespacedVariable {
                            namespace: name,
                            name: var_name,
                        });
                    }
                }
                // Check for namespace-qualified name: ident.ident (e.g. map.deep-merge)
                // The lexer produces Ident("map") Dot Ident("deep-merge") — combine them.
                let full_name = if matches!(self.parser.peek(), Token::Dot) {
                    let next_idx = self.parser.pos + 1;
                    if let Some(ts) = self.parser.tokens.get(next_idx) {
                        if let Token::Ident(ref second) = ts.token {
                            let combined = format!("{}.{}", name, second);
                            self.parser.advance(); // Dot
                            self.parser.advance(); // second ident
                            combined
                        } else {
                            name
                        }
                    } else {
                        name
                    }
                } else {
                    name
                };
                if matches!(self.parser.peek(), Token::LParen) {
                    self.parser.advance();
                    let args = self.parse_args()?;
                    self.parser.expect(Token::RParen)?;
                    if let Some(idx) = full_name.find('.') {
                        let ns = full_name[..idx].to_string();
                        let fn_name = full_name[idx + 1..].to_string();
                        return Ok(Expr::FunctionCall { name: fn_name, args, namespace: Some(ns) });
                    }
                    Ok(Expr::FunctionCall { name: full_name, args, namespace: None })
                } else if full_name == "true" {
                    Ok(Expr::Literal(Value::Bool(true)))
                } else if full_name == "false" {
                    Ok(Expr::Literal(Value::Bool(false)))
                } else if full_name == "null" {
                    Ok(Expr::Literal(Value::Null))
                } else {
                    Ok(Expr::Literal(Value::String(crate::value::SassString::unquoted(full_name))))
                }
            }
            Token::LParen => {
                self.parser.advance();
                let expr = self.parse_paren_content()?;
                self.parser.expect(Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                self.parser.advance();
                let items = self.parse_list_items(ListSeparator::Space)?;
                self.parser.expect(Token::RBracket)?;
                Ok(Expr::ListExpr { items, separator: ListSeparator::Space, bracketed: true })
            }
            Token::InterpolationStart => {
                self.parser.advance();
                let parts = self.parse_interpolation_parts()?;
                Ok(Expr::Interpolation(parts))
            }
            Token::Ampersand => {
                self.parser.advance();
                Ok(Expr::ParentSelector)
            }
            _ => Err(SassError::parse(
                format!("unexpected token in expression: {:?}", self.parser.peek()),
                pos,
            )),
        }
    }

    /// Parse list items with a given separator
    fn parse_list_items(&mut self, _sep: ListSeparator) -> Result<Vec<Expr>, SassError> {
        let mut items = Vec::new();
        while !matches!(self.parser.peek(), Token::RBracket | Token::Eof) {
            items.push(self.parse_expr()?);
        }
        Ok(items)
    }

    /// Parse interpolation parts until }
    fn parse_interpolation_parts(&mut self) -> Result<Vec<InterpPart>, SassError> {
        let mut parts = Vec::new();
        while !matches!(self.parser.peek(), Token::RBrace | Token::Eof) {
            let expr = self.parse_expr()?;
            parts.push(InterpPart::Expr(expr));
        }
        if matches!(self.parser.peek(), Token::RBrace) {
            self.parser.advance();
        }
        Ok(parts)
    }
}
