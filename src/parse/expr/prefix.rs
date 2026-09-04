use super::super::Parser;
use super::super::ast::*;
use crate::error::{Result, SassError};
use crate::lex::token::Token;

impl Parser<'_> {
    pub(crate) fn parse_prefix(&mut self) -> Result<Value> {
        self.skip_ws();
        match self.peek() {
            Some(Token::Minus) => {
                // 一元负号：当后面是数字、变量、括号表达式时
                // 标识符前的 - 是 CSS 厂商前缀（如 -webkit-inline-box）
                let next = self.tokens.get(self.pos + 1);
                if matches!(
                    next,
                    Some(Token::Number(_) | Token::Dollar(_) | Token::LParen | Token::Hash(_))
                ) {
                    self.advance();
                    self.skip_ws();
                    let val = self.parse_prefix()?;
                    Ok(Value::UnaryOp(UnaryOp::Neg, Box::new(val)))
                } else {
                    // 厂商前缀标识符——作为字符串保留
                    let mut name = String::from("-");
                    self.advance();
                    if let Some(Token::Ident(s)) = self.peek() {
                        name.push_str(s);
                        self.advance();
                    }
                    // 检查是否是函数调用
                    self.skip_ws();
                    if self.peek() == Some(&Token::LParen) {
                        let args = self.parse_args()?;
                        Ok(Value::Call(name, args))
                    } else {
                        Ok(Value::String(name, false))
                    }
                }
            }
            // ── 字面量解析委托给 literals 模块 ──
            Some(
                Token::Number(_)
                | Token::String(_, _)
                | Token::Hash(_)
                | Token::Dollar(_)
                | Token::Ident(_)
                | Token::Interp(_)
                | Token::True
                | Token::False
                | Token::Null
                | Token::Amp
                | Token::Star,
            ) => {
                if let Some(v) = self.parse_literal()? {
                    Ok(v)
                } else {
                    // parse_literal 返回 None — 理论上不应到此处（上面的 guard 确保匹配）
                    // 但作为安全兜底，尝试解析为字符串
                    match self.peek() {
                        Some(t) => {
                            let v = Value::String(t.to_string(), false);
                            self.advance();
                            Ok(v)
                        }
                        None => Ok(Value::Null),
                    }
                }
            }
            Some(Token::LParen) => {
                self.advance();
                self.skip_ws();
                // 空 Map 或列表
                if self.peek() == Some(&Token::RParen) {
                    self.advance();
                    return Ok(Value::List(vec![], Separator::Undecided, false));
                }
                let first = self.parse_expr(0)?;
                self.skip_ws();
                if self.peek() == Some(&Token::Colon) {
                    // Map
                    self.advance();
                    self.skip_ws();
                    let val = self.parse_expr(0)?;
                    let mut pairs = vec![(first, val)];
                    self.skip_ws();
                    while self.peek() == Some(&Token::Comma) {
                        self.advance();
                        self.skip_ws();
                        if self.peek() == Some(&Token::RParen) {
                            break;
                        } // 尾随逗号
                        let k = self.parse_expr(0)?;
                        self.skip_ws();
                        self.expect(&Token::Colon)?;
                        self.skip_ws();
                        let v = self.parse_expr(0)?;
                        pairs.push((k, v));
                        self.skip_ws();
                    }
                    self.expect(&Token::RParen)?;
                    Ok(Value::Map(pairs))
                } else {
                    // 分组或列表
                    let mut items = vec![first];
                    let mut saw_comma = false;
                    let sep = loop {
                        self.skip_ws();
                        match self.peek() {
                            Some(Token::Comma) => {
                                self.advance();
                                saw_comma = true;
                                self.skip_ws();
                                if self.peek() == Some(&Token::RParen) {
                                    break Separator::Comma;
                                }
                            }
                            Some(Token::RParen) => {
                                break if saw_comma {
                                    Separator::Comma
                                } else {
                                    Separator::Space
                                };
                            }
                            // 空格分隔的值——继续解析
                            Some(
                                Token::Number(_)
                                | Token::String(_, _)
                                | Token::Ident(_)
                                | Token::Hash(_)
                                | Token::Dollar(_)
                                | Token::Interp(_)
                                | Token::LParen,
                            ) => {
                                items.push(self.parse_expr(0)?);
                            }
                            _ => {
                                break if saw_comma {
                                    Separator::Comma
                                } else {
                                    Separator::Space
                                };
                            }
                        }
                        self.skip_ws();
                        if self.peek() == Some(&Token::RParen) {
                            break if saw_comma {
                                Separator::Comma
                            } else {
                                Separator::Space
                            };
                        }
                    };
                    self.skip_ws();
                    if self.peek() == Some(&Token::RParen) {
                        self.advance();
                    }
                    if items.len() == 1 && !saw_comma {
                        Ok(Value::Paren(Box::new(
                            items
                                .into_iter()
                                .next()
                                .expect("items has exactly 1 element"),
                        )))
                    } else {
                        Ok(Value::List(items, sep, false))
                    }
                }
            }
            Some(Token::Not) => {
                // 检查 not 后面是否紧跟 ( 无空格
                self.advance();
                let next_is_paren_no_ws = matches!(self.peek(), Some(Token::LParen))
                    && !matches!(
                        self.tokens.get(self.pos.saturating_sub(1)),
                        Some(Token::Whitespace)
                    );
                if next_is_paren_no_ws {
                    return Err(SassError::Parse {
                        expected: "whitespace between \"not\" and \"(\"".into(),
                        found: "(".into(),
                    });
                }
                self.skip_ws();
                let v = self.parse_prefix()?;
                Ok(Value::UnaryOp(UnaryOp::Not, Box::new(v)))
            }
            Some(Token::LBracket) => {
                // bracketed list
                self.advance();
                let mut items = Vec::new();
                let mut saw_comma = false;
                loop {
                    self.skip_ws();
                    if self.peek() == Some(&Token::RBracket) {
                        break;
                    }
                    items.push(self.parse_expr(0)?);
                    self.skip_ws();
                    if self.peek() == Some(&Token::Comma) {
                        self.advance();
                        saw_comma = true;
                    } else {
                        break;
                    }
                }
                if self.peek() == Some(&Token::RBracket) {
                    self.advance();
                }
                let sep = match (saw_comma, items.len()) {
                    (true, _) => Separator::Comma,
                    (false, n) if n <= 1 => Separator::Undecided,
                    (false, _) => Separator::Space,
                };
                Ok(Value::List(items, sep, true))
            }
            Some(Token::Percent) => {
                // % 作为独立值 = 字符串 %
                self.advance();
                Ok(Value::String("%".to_string(), false))
            }
            _ => {
                // 尝试解析为标识符字符串——但不消费终止符
                match self.peek() {
                    Some(Token::Dot) => Err(SassError::Parse {
                        expected: "digit".into(),
                        found: ".".into(),
                    }),
                    // and/or/not 关键字不能作为值起始
                    Some(Token::And | Token::Or) => {
                        let found = self
                            .peek()
                            .map_or("EOF".to_string(), std::string::ToString::to_string);
                        Err(SassError::Parse {
                            expected: "expression".into(),
                            found,
                        })
                    }
                    Some(
                        Token::RBrace
                        | Token::RParen
                        | Token::Semicolon
                        | Token::RBracket
                        | Token::Comma
                        | Token::Colon,
                    ) => Ok(Value::Null),
                    Some(t) => {
                        let v = Value::String(t.to_string(), false);
                        self.advance();
                        Ok(v)
                    }
                    None => Ok(Value::Null),
                }
            }
        }
    }

    /// 从已有片段开始，继续向后拼接相邻的 ident/number/interp/hash token。
    ///
    /// 处理 `hey#{$y}ho`、`#{$a}px`、`#{1+2}rem` 等场景。
    /// 当片段后紧跟 `()` 时，作为函数调用名。
    pub(crate) fn parse_interp_adjacent(
        &mut self,
        mut segments: Vec<crate::parse::ast::InterpSegment>,
    ) -> Result<Value> {
        use crate::parse::ast::InterpSegment;
        loop {
            match self.peek() {
                Some(Token::Ident(t)) if !Self::is_keyword(t) => {
                    segments.push(InterpSegment::Text(t.clone()));
                    self.advance();
                }
                Some(Token::Number(n)) => {
                    segments.push(InterpSegment::Text(n.clone()));
                    self.advance();
                }
                Some(Token::Interp(t)) => {
                    segments.push(InterpSegment::Expr(t.clone()));
                    self.advance();
                }
                Some(Token::Hash(h)) => {
                    segments.push(InterpSegment::Text(format!("#{h}")));
                    self.advance();
                }
                _ => break,
            }
        }
        if segments.len() == 1 {
            let single = segments.into_iter().next().expect("segments has 1 element");
            match &single {
                InterpSegment::Expr(_) | InterpSegment::Text(_) => {
                    self.skip_ws();
                    if self.peek() == Some(&Token::LParen) {
                        let expr = match single {
                            InterpSegment::Expr(e) => e,
                            InterpSegment::Text(t) => t,
                        };
                        let args = self.parse_args()?;
                        Ok(Value::Call(expr, args))
                    } else {
                        Ok(Value::Interp(vec![single]))
                    }
                }
            }
        } else {
            let joined: String = segments
                .iter()
                .map(|seg| match seg {
                    InterpSegment::Expr(e) => e.clone(),
                    InterpSegment::Text(t) => t.clone(),
                })
                .collect();
            self.skip_ws();
            if self.peek() == Some(&Token::LParen) {
                let args = self.parse_args()?;
                Ok(Value::Call(joined, args))
            } else {
                Ok(Value::Interp(segments))
            }
        }
    }
}

/// 解析数字字符串为 `Value::Number`。
pub(crate) fn parse_number(s: &str) -> Result<Value> {
    let (num_part, unit) = if let Some(idx) = s.find(|c: char| c.is_ascii_alphabetic() || c == '%')
    {
        (&s[..idx], Some(s[idx..].to_string()))
    } else {
        (s, None)
    };
    match num_part.parse::<f64>() {
        Ok(n) => Ok(Value::Number(n, unit)),
        Err(_) => Err(SassError::Parse {
            expected: "number".into(),
            found: s.to_string(),
        }),
    }
}

/// 解析 #hash 字符串为 Color。
pub(crate) fn parse_hash_color(s: &str) -> Color {
    let bytes = s.as_bytes();
    match bytes.len() {
        3 => Color::rgb(
            f64::from(hex2(bytes[0], bytes[0])),
            f64::from(hex2(bytes[1], bytes[1])),
            f64::from(hex2(bytes[2], bytes[2])),
        ),
        4 => Color::rgba(
            f64::from(hex2(bytes[1], bytes[1])),
            f64::from(hex2(bytes[2], bytes[2])),
            f64::from(hex2(bytes[3], bytes[3])),
            f64::from(hex1(bytes[0])) / 15.0,
        ),
        6 => Color::rgb(
            f64::from(hex2(bytes[0], bytes[1])),
            f64::from(hex2(bytes[2], bytes[3])),
            f64::from(hex2(bytes[4], bytes[5])),
        ),
        8 => Color::rgba(
            f64::from(hex2(bytes[0], bytes[1])),
            f64::from(hex2(bytes[2], bytes[3])),
            f64::from(hex2(bytes[4], bytes[5])),
            f64::from(hex2(bytes[6], bytes[7])) / 255.0,
        ),
        _ => Color::default(),
    }
}

pub(crate) fn hex2(hi: u8, lo: u8) -> u8 {
    (hex1(hi) << 4) | hex1(lo)
}
pub(crate) fn hex1(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
}
