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
            match self.peek() {
                Some(Token::RParen) => break,
                _ => {}
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
            let default = match self.peek() {
                Some(Token::Colon) => {
                    self.advance();
                    self.skip_ws();
                    Some(self.parse_expr(0)?)
                }
                _ => None,
            };
            let rest = match self.peek() {
                Some(Token::DotDotDot) => {
                    self.advance();
                    true
                }
                _ => false,
            };
            params.push(Param {
                name,
                default,
                rest,
            });
            self.skip_ws();
            match self.peek() {
                Some(Token::Comma) => {
                    self.advance();
                }
                _ => break,
            }
        }
        self.skip_ws();
        match self.peek() {
            Some(Token::RParen) => {
                self.advance();
            }
            _ => {}
        }
        Ok(params)
    }

    pub(crate) fn parse_args(&mut self) -> Result<Vec<Arg>> {
        self.expect(&Token::LParen)?;
        let mut args = Vec::new();
        loop {
            self.skip_ws();
            match self.peek() {
                Some(Token::RParen) => break,
                _ => {}
            }
            // 检查关键字参数 $name: value 或 name: value
            let is_kwarg = match self.peek() {
                Some(Token::Dollar(_)) => {
                    let next = self.tokens.get(self.pos + 1);
                    let after_ws = match next {
                        Some(Token::Whitespace) => self.tokens.get(self.pos + 2),
                        _ => next,
                    };
                    matches!(after_ws, Some(Token::Colon))
                }
                Some(Token::Ident(s))
                    if !matches!(
                        s.as_str(),
                        "true" | "false" | "null" | "and" | "or" | "not" | "else"
                    ) =>
                {
                    let next = self.tokens.get(self.pos + 1);
                    let after_ws = match next {
                        Some(Token::Whitespace) => self.tokens.get(self.pos + 2),
                        _ => next,
                    };
                    matches!(after_ws, Some(Token::Colon | Token::Assign))
                }
                _ => false,
            };
            let (name, value) = match is_kwarg {
                true => {
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
                    match self.peek() {
                        Some(Token::Colon | Token::Assign) => {
                            self.advance();
                        }
                        _ => {}
                    }
                    self.skip_ws();
                    (Some(n), self.parse_expr(0)?)
                }
                false => {
                    let expr = self.parse_expr(0)?;
                    self.skip_ws();
                    match self.peek() {
                        Some(Token::Colon) => {
                            self.advance();
                            self.skip_ws();
                            let val = self.parse_expr(0)?;
                            let spread = match self.peek() {
                                Some(Token::DotDotDot) => {
                                    self.advance();
                                    true
                                }
                                _ => false,
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
                                match self.peek() {
                                    // 双分号——报错
                                    Some(Token::Semicolon) => {
                                        return Err(SassError::Parse {
                                            expected: "identifier".into(),
                                            found: ";".into(),
                                        });
                                    }
                                    // 分号后紧跟 ) ——报错
                                    Some(Token::RParen) => {
                                        return Err(SassError::Parse {
                                            expected: "identifier".into(),
                                            found: ")".into(),
                                        });
                                    }
                                    // 分号后紧跟 , ——报错
                                    Some(Token::Comma) => {
                                        return Err(SassError::Parse {
                                            expected: ")".into(),
                                            found: ",".into(),
                                        });
                                    }
                                    Some(Token::Ident(s)) if s == "else" => {
                                        self.advance();
                                        self.skip_ws();
                                        match self.peek() {
                                            Some(Token::Colon) => {
                                                self.advance();
                                                self.skip_ws();
                                            }
                                            _ => {}
                                        }
                                        let else_val = self.parse_expr(0)?;
                                        args.push(Arg {
                                            name: Some("else".to_string()),
                                            value: else_val,
                                            spread: false,
                                            condition: None,
                                        });
                                        self.skip_ws();
                                        match self.peek() {
                                            Some(Token::Semicolon) => {
                                                self.advance();
                                            }
                                            _ => {}
                                        }
                                        break;
                                    }
                                    _ => {
                                        let cond2 = self.parse_expr(0)?;
                                        self.skip_ws();
                                        match self.peek() {
                                            Some(Token::Colon) => {
                                                self.advance();
                                                self.skip_ws();
                                                let val2 = self.parse_expr(0)?;
                                                let spread2 = match self.peek() {
                                                    Some(Token::DotDotDot) => {
                                                        self.advance();
                                                        true
                                                    }
                                                    _ => false,
                                                };
                                                args.push(Arg {
                                                    name: None,
                                                    value: val2,
                                                    spread: spread2,
                                                    condition: Some(cond2),
                                                });
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            self.skip_ws();
                            match self.peek() {
                                Some(Token::Comma) => {
                                    return Err(SassError::Parse {
                                        expected: ")".into(),
                                        found: ",".into(),
                                    });
                                }
                                _ => {}
                            }
                            break;
                        }
                        _ => {}
                    }
                    (None, expr)
                }
            };
            let spread = match self.peek() {
                Some(Token::DotDotDot) => {
                    self.advance();
                    true
                }
                _ => false,
            };
            args.push(Arg {
                name,
                value,
                spread,
                condition: None,
            });
            self.skip_ws();
            match self.peek() {
                Some(Token::Comma) => {
                    self.advance();
                }
                _ => break,
            }
        }
        self.skip_ws();
        match self.peek() {
            Some(Token::RParen) => {
                self.advance();
            }
            _ => {}
        }
        Ok(args)
    }

    /// 解析 `with ($x: val, ...)` 配置参数。
    /// `allow_default = true` 允许 `!default` 标志（`@forward` 场景）。
    pub(crate) fn parse_config(&mut self, allow_default: bool) -> Result<Vec<ConfigVar>> {
        let mut config = Vec::new();
        let mut seen = std::collections::HashSet::new();
        self.skip_ws();
        match self.peek() {
            Some(Token::RParen) => return Err(SassError::Eval("expected \"$\".".into())),
            _ => {}
        }
        loop {
            self.skip_ws();
            match self.peek() {
                Some(Token::RParen) => break,
                _ => {}
            }
            let name = match self.peek() {
                Some(Token::Dollar(n)) => {
                    let n = n.clone();
                    match n.is_empty() {
                        true => return Err(SassError::Eval("Expected identifier.".into())),
                        false => {}
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
            // @use with() 中不允许 !default 标志；@forward with() 允许
            let mut is_default = false;
            match self.peek() {
                Some(Token::Bang) => {
                    match allow_default {
                        false => return Err(SassError::Eval("expected \")\".".into())),
                        true => {}
                    }
                    self.advance();
                    self.skip_ws();
                    match self.peek() {
                        Some(Token::Ident(s)) => {
                            match s.as_str() {
                                "default" => is_default = true,
                                _ => {}
                            }
                            self.advance();
                            self.skip_ws();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            // 重复配置变量检测——规范要求在解析阶段就报错
            let normalized = name.replace('-', "_");
            match seen.insert(normalized) {
                false => {
                    return Err(SassError::Eval(
                        "The same variable may only be configured once.".into(),
                    ));
                }
                true => {}
            }
            config.push(ConfigVar {
                name,
                value,
                is_default,
            });
            match self.peek() {
                Some(Token::Comma) => {
                    self.advance();
                    self.skip_ws();
                    let is_ok = matches!(self.peek(), Some(Token::Dollar(_)));
                    match is_ok {
                        false => match self.peek() {
                            Some(Token::RParen) => {}
                            _ => return Err(SassError::Eval("expected \")\".".into())),
                        },
                        true => {}
                    }
                }
                _ => break,
            }
        }
        match self.peek() {
            Some(Token::RParen) => {
                self.advance();
            }
            _ => {}
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
                    match n.is_empty() {
                        true => {
                            return Err(SassError::Eval(
                                "Expected variable, mixin, or function name".into(),
                            ));
                        }
                        false => {}
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
            match self.peek() {
                Some(Token::Comma) => {
                    self.advance();
                    self.skip_ws();
                    let is_ok = matches!(self.peek(), Some(Token::Dollar(_)));
                    match is_ok {
                        false => match self.peek() {
                            Some(Token::Ident(_)) => {}
                            _ => {
                                return Err(SassError::Eval(
                                    "Expected variable, mixin, or function name".into(),
                                ));
                            }
                        },
                        true => {}
                    }
                }
                _ => break,
            }
        }
        Ok(members)
    }
}
