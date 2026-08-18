//! Argument and paren-content parsing for the expression parser.

use crate::ast::*;
use crate::error::SassError;
use crate::token::Token;
use crate::value::Value;
use super::ExprParser;

impl ExprParser<'_> {
    /// Parse function call arguments.
    ///
    /// Handles positional args, named args ($name: value),
    /// and spread/rest args (e.g. $args...).
    pub fn parse_args(&mut self) -> Result<Vec<Arg>, SassError> {
        let mut args = Vec::new();
        while !matches!(self.parser.peek(), Token::RParen | Token::Eof) {
            // Check for named argument: $name: value
            if let Token::Variable(name) = self.parser.peek().clone() {
                let next_idx = self.parser.pos + 1;
                if let Some(ts) = self.parser.tokens.get(next_idx) {
                    if matches!(ts.token, Token::Colon) {
                        self.parser.advance(); // variable
                        self.parser.advance(); // colon
                        let value = self.parse_space_list()?;
                        let spread = matches!(self.parser.peek(), Token::Spread);
                        if spread {
                            self.parser.advance();
                        }
                        args.push(Arg { name: Some(name), value, spread });
                        if matches!(self.parser.peek(), Token::Comma) {
                            self.parser.advance();
                        } else {
                            break;
                        }
                        continue;
                    }
                }
            }

            // Check for standalone spread operator
            if matches!(self.parser.peek(), Token::Spread) {
                self.parser.advance();
                args.push(Arg {
                    name: None,
                    value: Expr::Literal(Value::Null),
                    spread: true,
                });
                if matches!(self.parser.peek(), Token::Comma) {
                    self.parser.advance();
                } else {
                    break;
                }
                continue;
            }

            // Positional argument — parse full expression
            let value = self.parse_space_list()?;
            let spread = matches!(self.parser.peek(), Token::Spread);
            if spread {
                self.parser.advance();
            }
            args.push(Arg { name: None, value, spread });

            if matches!(self.parser.peek(), Token::Comma) {
                self.parser.advance();
            } else {
                break;
            }
        }
        Ok(args)
    }

    /// Parse content inside parentheses — could be expr, map, or list.
    pub fn parse_paren_content(&mut self) -> Result<Expr, SassError> {
        // Empty parens = empty list
        if matches!(self.parser.peek(), Token::RParen) {
            return Ok(Expr::ListExpr {
                items: Vec::new(),
                separator: ListSeparator::Space,
                bracketed: false,
            });
        }

        // Parse the first item as a space-separated list so that
        // expressions like `0px 12px rgba(0,0,0,0.04)` are correctly
        // grouped as a single list item before checking for comma
        // separation.  Previously this used parse_or() which only
        // captured the first term (e.g. `0px`), leaving the remaining
        // terms to be parsed as separate items — breaking comma-list
        // detection when a space list contains function calls with
        // comma-separated arguments.
        let first = self.parse_space_list()?;

        // Map: key: value, ...
        if matches!(self.parser.peek(), Token::Colon) {
            self.parser.advance();
            let val = self.parse_space_list()?;
            let mut entries = vec![(first, val)];
            while matches!(self.parser.peek(), Token::Comma) {
                self.parser.advance();
                if matches!(self.parser.peek(), Token::RParen) {
                    break;
                }
                let key = self.parse_space_list()?;
                self.parser.expect(Token::Colon)?;
                let v = self.parse_space_list()?;
                entries.push((key, v));
            }
            return Ok(Expr::MapExpr(entries));
        }

        // List: items separated by space or comma
        let mut items = vec![first];
        let separator = if matches!(self.parser.peek(), Token::Comma) {
            self.parser.advance();
            while !matches!(self.parser.peek(), Token::RParen | Token::Eof) {
                items.push(self.parse_space_list()?);
                if matches!(self.parser.peek(), Token::Comma) {
                    self.parser.advance();
                } else {
                    break;
                }
            }
            ListSeparator::Comma
        } else {
            while !matches!(self.parser.peek(), Token::RParen | Token::Eof | Token::Comma) {
                items.push(self.parse_space_list()?);
            }
            ListSeparator::Space
        };

        if items.len() == 1 {
            Ok(Expr::Paren(Box::new(items.into_iter().next().unwrap())))
        } else {
            Ok(Expr::ListExpr { items, separator, bracketed: false })
        }
    }
}
