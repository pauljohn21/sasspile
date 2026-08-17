//! Expression parser — handles operator precedence, function calls, lists, maps.

use crate::ast::*;
use crate::error::SassError;
use crate::token::Token;
use crate::value::Value;
use super::Parser;

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
        // Check for comma-separated list at top level
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

    fn parse_or(&mut self) -> Result<Expr, SassError> {
        let mut left = self.parse_and()?;
        while matches!(self.parser.peek(), Token::Ident(s) if s == "or") {
            self.parser.advance();
            let right = self.parse_and()?;
            left = Expr::Operation {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, SassError> {
        let mut left = self.parse_equality()?;
        while matches!(self.parser.peek(), Token::Ident(s) if s == "and") {
            self.parser.advance();
            let right = self.parse_equality()?;
            left = Expr::Operation {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
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
            left = Expr::Operation {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
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
            left = Expr::Operation {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
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
            left = Expr::Operation {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
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
            // Check if this is actually a slash-separated list
            // (parser context decides, but for now treat as division)
            self.parser.advance();
            let right = self.parse_unary()?;
            left = Expr::Operation {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, SassError> {
        match self.parser.peek() {
            Token::Minus => {
                self.parser.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                })
            }
            Token::Ident(s) if s == "not" => {
                self.parser.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, SassError> {
        let pos = self.parser.current_pos();
        match self.parser.peek().clone() {
            Token::Number(v, u) => {
                self.parser.advance();
                Ok(Expr::Literal(Value::Number(crate::value::Number::new(v, u))))
            }
            Token::HexColor(v) => {
                self.parser.advance();
                // Expand 3-digit hex (#fff → #ffffff)
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
                // Check for function call
                if matches!(self.parser.peek(), Token::LParen) {
                    self.parser.advance(); // (
                    let args = self.parse_args()?;
                    self.parser.expect(Token::RParen)?;
                    // namespace check: if name contains '.', split
                    if let Some(idx) = name.find('.') {
                        let ns = name[..idx].to_string();
                        let fn_name = name[idx + 1..].to_string();
                        return Ok(Expr::FunctionCall {
                            name: fn_name,
                            args,
                            namespace: Some(ns),
                        });
                    }
                    Ok(Expr::FunctionCall {
                        name,
                        args,
                        namespace: None,
                    })
                } else if name == "true" {
                    Ok(Expr::Literal(Value::Bool(true)))
                } else if name == "false" {
                    Ok(Expr::Literal(Value::Bool(false)))
                } else if name == "null" {
                    Ok(Expr::Literal(Value::Null))
                } else {
                    Ok(Expr::Literal(Value::String(crate::value::SassString::unquoted(name))))
                }
            }
            Token::LParen => {
                self.parser.advance();
                // Could be parenthesized expression, map, or list
                let expr = self.parse_paren_content()?;
                self.parser.expect(Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                self.parser.advance();
                let items = self.parse_list_items(ListSeparator::Space)?;
                self.parser.expect(Token::RBracket)?;
                Ok(Expr::ListExpr {
                    items,
                    separator: ListSeparator::Space,
                    bracketed: true,
                })
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

    /// Parse function call arguments
    pub fn parse_args(&mut self) -> Result<Vec<Arg>, SassError> {
        let mut args = Vec::new();
        while !matches!(self.parser.peek(), Token::RParen | Token::Eof) {
            // Check for named argument: $name: value
            let spread = if let Token::Ident(s) = self.parser.peek() {
                s == "..."
            } else {
                false
            };

            if let Token::Variable(name) = self.parser.peek().clone() {
                self.parser.advance();
                if matches!(self.parser.peek(), Token::Colon) {
                    self.parser.advance();
                    let value = self.parse_space_list()?;
                    args.push(Arg {
                        name: Some(name),
                        value,
                        spread: false,
                    });
                } else {
                    // Positional variable arg — could be followed by space-separated items
                    let value = Expr::Variable(name);
                    let value = self.maybe_space_list(value)?;
                    args.push(Arg {
                        name: None,
                        value,
                        spread,
                    });
                }
            } else {
                let value = self.parse_space_list()?;
                args.push(Arg {
                    name: None,
                    value,
                    spread,
                });
            }

            if matches!(self.parser.peek(), Token::Comma) {
                self.parser.advance();
            } else {
                break;
            }
        }
        Ok(args)
    }

    /// Parse a space-separated list (used inside function args).
    /// Parses a single expression with parse_or, then checks if there are
    /// more space-separated values to form a list.
    fn parse_space_list(&mut self) -> Result<Expr, SassError> {
        let first = self.parse_or()?;
        // Check if there are more space-separated items (not operators, not comma, not rparen)
        if self.is_space_separated() {
            let mut items = vec![first];
            while self.is_space_separated() {
                items.push(self.parse_or()?);
            }
            return Ok(Expr::ListExpr {
                items,
                separator: ListSeparator::Space,
                bracketed: false,
            });
        }
        Ok(first)
    }

    /// If the next token starts a new space-separated item, build a list.
    fn maybe_space_list(&mut self, first: Expr) -> Result<Expr, SassError> {
        if self.is_space_separated() {
            let mut items = vec![first];
            while self.is_space_separated() {
                items.push(self.parse_or()?);
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
            Token::Colon
        ) && !matches!(self.parser.peek(), Token::Ident(s) if matches!(s.as_str(), "and" | "or" | "through" | "to" | "from" | "!default" | "!global"))
    }

    /// Parse content inside parentheses — could be expr, map, or list
    fn parse_paren_content(&mut self) -> Result<Expr, SassError> {
        // Empty parens = empty list
        if matches!(self.parser.peek(), Token::RParen) {
            return Ok(Expr::ListExpr {
                items: Vec::new(),
                separator: ListSeparator::Space,
                bracketed: false,
            });
        }

        let first = self.parse_or()?;

        // Map: key: value, ...
        if matches!(self.parser.peek(), Token::Colon) {
            self.parser.advance();
            let val = self.parse_or()?;
            let mut entries = vec![(first, val)];
            while matches!(self.parser.peek(), Token::Comma) {
                self.parser.advance();
                if matches!(self.parser.peek(), Token::RParen) {
                    break;
                }
                let key = self.parse_or()?;
                self.parser.expect(Token::Colon)?;
                let v = self.parse_or()?;
                entries.push((key, v));
            }
            return Ok(Expr::MapExpr(entries));
        }

        // List: items separated by space or comma
        let mut items = vec![first];
        let separator = if matches!(self.parser.peek(), Token::Comma) {
            self.parser.advance();
            while !matches!(self.parser.peek(), Token::RParen | Token::Eof) {
                items.push(self.parse_or()?);
                if matches!(self.parser.peek(), Token::Comma) {
                    self.parser.advance();
                } else {
                    break;
                }
            }
            ListSeparator::Comma
        } else {
            while !matches!(self.parser.peek(), Token::RParen | Token::Eof | Token::Comma) {
                items.push(self.parse_or()?);
            }
            ListSeparator::Space
        };

        if items.len() == 1 {
            Ok(Expr::Paren(Box::new(items.into_iter().next().unwrap())))
        } else {
            Ok(Expr::ListExpr {
                items,
                separator,
                bracketed: false,
            })
        }
    }

    /// Parse list items with a given separator
    fn parse_list_items(
        &mut self,
        _sep: ListSeparator,
    ) -> Result<Vec<Expr>, SassError> {
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
