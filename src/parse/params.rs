//! 参数解析 + 配置解析。
//!
//! `parse_params` / `parse_args` / `parse_config` / `parse_member_list`。

use super::Parser;
use super::ast::*;
use crate::error::{Result, SassError};
use crate::lex::token::Token;

impl Parser<'_> {
    pub(crate) fn parse_params(&mut self) -> Result<Vec<Param>> {
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(&Token::RParen) {
                break;
            }
            let name = match self.peek() {
                Some(Token::Dollar(n)) => {
                    let n = n.clone();
                    self.advance();
                    n
                }
                _ => {
                    return Err(SassError::Parse {
                        expected: "$param".into(),
                        found: "other".into(),
                    });
                }
            };
            self.skip_ws();
            let default = if self.peek() == Some(&Token::Colon) {
                self.advance();
                self.skip_ws();
                Some(self.parse_expr(0)?)
            } else {
                None
            };
            let rest = if self.peek() == Some(&Token::DotDotDot) {
                self.advance();
                true
            } else {
                false
            };
            params.push(Param {
                name,
                default,
                rest,
            });
            self.skip_ws();
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.skip_ws();
        if self.peek() == Some(&Token::RParen) {
            self.advance();
        }
        Ok(params)
    }

    pub(crate) fn parse_args(&mut self) -> Result<Vec<Arg>> {
        self.expect(&Token::LParen)?;
        let mut args = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(&Token::RParen) {
                break;
            }
            // 检查关键字参数 $name: value 或 name: value
            let is_kwarg = match self.peek() {
                Some(Token::Dollar(_)) => {
                    let next = self.tokens.get(self.pos + 1);
                    let after_ws = if matches!(next, Some(Token::Whitespace)) {
                        self.tokens.get(self.pos + 2)
                    } else {
                        next
                    };
                    matches!(after_ws, Some(Token::Colon))
                }
                Some(Token::Ident(s))
                    if !matches!(s.as_str(), "true" | "false" | "null" | "and" | "or" | "not") =>
                {
                    let next = self.tokens.get(self.pos + 1);
                    let after_ws = if matches!(next, Some(Token::Whitespace)) {
                        self.tokens.get(self.pos + 2)
                    } else {
                        next
                    };
                    matches!(after_ws, Some(Token::Colon | Token::Assign))
                }
                _ => false,
            };
            let (name, value) = if is_kwarg {
                let n = match self.peek() {
                    Some(Token::Dollar(n)) => {
                        let n = n.clone();
                        self.advance();
                        n
                    }
                    Some(Token::Ident(n)) => {
                        let n = n.clone();
                        self.advance();
                        n
                    }
                    _ => unreachable!(),
                };
                self.skip_ws();
                if matches!(self.peek(), Some(Token::Colon | Token::Assign)) {
                    self.advance();
                }
                self.skip_ws();
                (Some(n), self.parse_expr(0)?)
            } else {
                let expr = self.parse_expr(0)?;
                self.skip_ws();
                if self.peek() == Some(&Token::Colon) {
                    self.advance();
                    self.skip_ws();
                    let val = self.parse_expr(0)?;
                    let spread = if self.peek() == Some(&Token::DotDotDot) {
                        self.advance();
                        true
                    } else {
                        false
                    };
                    args.push(Arg {
                        name: None,
                        value: val,
                        spread,
                        condition: Some(expr),
                    });
                    self.skip_ws();
                    while self.peek() == Some(&Token::Semicolon) {
                        self.advance();
                        self.skip_ws();
                        if let Some(Token::Ident(s)) = self.peek()
                            && s == "else"
                        {
                            self.advance();
                            self.skip_ws();
                            if self.peek() == Some(&Token::Colon) {
                                self.advance();
                                self.skip_ws();
                            }
                            let else_val = self.parse_expr(0)?;
                            args.push(Arg {
                                name: Some("else".to_string()),
                                value: else_val,
                                spread: false,
                                condition: None,
                            });
                            break;
                        }
                        let cond2 = self.parse_expr(0)?;
                        self.skip_ws();
                        if self.peek() == Some(&Token::Colon) {
                            self.advance();
                            self.skip_ws();
                            let val2 = self.parse_expr(0)?;
                            let spread2 = if self.peek() == Some(&Token::DotDotDot) {
                                self.advance();
                                true
                            } else {
                                false
                            };
                            args.push(Arg {
                                name: None,
                                value: val2,
                                spread: spread2,
                                condition: Some(cond2),
                            });
                        }
                    }
                    self.skip_ws();
                    if self.peek() == Some(&Token::Comma) {
                        self.advance();
                        continue;
                    }
                    break;
                }
                (None, expr)
            };
            let spread = if self.peek() == Some(&Token::DotDotDot) {
                self.advance();
                true
            } else {
                false
            };
            args.push(Arg {
                name,
                value,
                spread,
                condition: None,
            });
            self.skip_ws();
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.skip_ws();
        if self.peek() == Some(&Token::RParen) {
            self.advance();
        }
        Ok(args)
    }

    pub(crate) fn parse_config(&mut self) -> Result<Vec<ConfigVar>> {
        let mut config = Vec::new();
        self.skip_ws();
        if self.peek() == Some(&Token::RParen) {
            return Err(SassError::Eval("expected \"$\".".into()));
        }
        loop {
            self.skip_ws();
            if self.peek() == Some(&Token::RParen) {
                break;
            }
            let name = match self.peek() {
                Some(Token::Dollar(n)) => {
                    let n = n.clone();
                    if n.is_empty() {
                        return Err(SassError::Eval("Expected identifier.".into()));
                    }
                    self.advance();
                    n
                }
                _ => {
                    return Err(SassError::Eval("expected \"$\".".into()));
                }
            };
            self.skip_ws();
            self.expect(&Token::Colon)?;
            self.skip_ws();
            let value = self.parse_expr(0)?;
            self.skip_ws();
            let mut is_default = false;
            while self.peek() == Some(&Token::Bang) {
                self.advance();
                self.skip_ws();
                if let Some(Token::Ident(s)) = self.peek() {
                    if s == "default" {
                        is_default = true;
                    }
                    self.advance();
                    self.skip_ws();
                }
            }
            config.push(ConfigVar {
                name,
                value,
                is_default,
            });
            if self.peek() == Some(&Token::Comma) {
                self.advance();
                self.skip_ws();
                let is_ok = matches!(self.peek(), Some(Token::Dollar(_)));
                if !is_ok && !matches!(self.peek(), Some(Token::RParen)) {
                    return Err(SassError::Eval("expected \")\".".into()));
                }
            } else {
                break;
            }
        }
        if self.peek() == Some(&Token::RParen) {
            self.advance();
        }
        Ok(config)
    }

    pub(crate) fn parse_member_list(&mut self) -> Result<Vec<String>> {
        let mut members = Vec::new();
        self.skip_ws();
        loop {
            match self.peek() {
                Some(Token::Semicolon | Token::LBrace) | None => break,
                Some(Token::Dollar(n)) => {
                    if n.is_empty() {
                        return Err(SassError::Eval(
                            "Expected variable, mixin, or function name".into(),
                        ));
                    }
                    members.push(format!("${n}"));
                    self.advance();
                }
                Some(Token::Ident(n)) => {
                    members.push(n.clone());
                    self.advance();
                }
                _ => {
                    return Err(SassError::Eval(
                        "Expected variable, mixin, or function name".into(),
                    ));
                }
            }
            self.skip_ws();
            if self.peek() == Some(&Token::Comma) {
                self.advance();
                self.skip_ws();
                let is_ok = matches!(self.peek(), Some(Token::Dollar(_)));
                if !is_ok && !matches!(self.peek(), Some(Token::Ident(_))) {
                    return Err(SassError::Eval(
                        "Expected variable, mixin, or function name".into(),
                    ));
                }
            } else {
                break;
            }
        }
        Ok(members)
    }
}
