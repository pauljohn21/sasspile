use super::super::ParseStream;
use super::super::ast::*;
use crate::error::{Result, SassError};
use crate::lex::token::Token;

impl ParseStream<'_> {
    pub(crate) fn parse_prefix(&mut self) -> Result<Value> {
        self.skip_ws();
        match self.peek() {
            Some(Token::Minus) => {
                // 一元负号：当后面是数字、变量、括号表达式时
                // 标识符前的 - 是 CSS 厂商前缀（如 -webkit-inline-box）
                let next = self.tokens.get(self.pos + 1);
                match matches!(
                    next,
                    Some(Token::Number(_) | Token::Dollar(_) | Token::LParen | Token::Hash(_))
                ) {
                    true => {
                        self.advance();
                        self.skip_ws();
                        let val = self.parse_prefix()?;
                        Ok(Value::UnaryOp(UnaryOp::Neg, Box::new(val)))
                    }
                    false => {
                        // 厂商前缀标识符——作为字符串保留
                        let mut name = String::from("-");
                        self.advance();
                        match self.peek() {
                            Some(Token::Ident(s)) => {
                                name.push_str(s);
                                self.advance();
                            }
                            _ => {}
                        }
                        // 检查是否是函数调用
                        self.skip_ws();
                        match self.peek() {
                            Some(Token::LParen) => {
                                let args = self.parse_args()?;
                                Ok(Value::Call(name, args))
                            }
                            _ => Ok(Value::String(name, false)),
                        }
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
                | Token::And
                | Token::Or
                | Token::Not
                | Token::Amp
                | Token::Star,
            ) => {
                match self.parse_literal()? {
                    Some(v) => Ok(v),
                    None => {
                        // parse_literal 返回 None — 安全兜底
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
            }
            Some(Token::LParen) => {
                self.advance();
                self.skip_ws();
                // 空 Map 或列表
                match self.peek() {
                    Some(Token::RParen) => {
                        self.advance();
                        return Ok(Value::List(vec![], Separator::Undecided, false));
                    }
                    _ => {}
                }
                let first = self.parse_expr(0)?;
                self.skip_ws();
                match self.peek() {
                    Some(Token::Colon) => {
                        // Map
                        self.advance();
                        self.skip_ws();
                        let val = self.parse_expr(0)?;
                        let mut pairs = vec![(first, val)];
                        self.skip_ws();
                        while self.peek() == Some(&Token::Comma) {
                            self.advance();
                            self.skip_ws();
                            match self.peek() {
                                Some(Token::RParen) => break, // 尾随逗号
                                _ => {}
                            }
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
                    }
                    _ => {
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
                                    match self.peek() {
                                        Some(Token::RParen) => break Separator::Comma,
                                        _ => {}
                                    }
                                }
                                Some(Token::RParen) => {
                                    break match saw_comma {
                                        true => Separator::Comma,
                                        false => Separator::Space,
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
                                    break match saw_comma {
                                        true => Separator::Comma,
                                        false => Separator::Space,
                                    };
                                }
                            }
                            self.skip_ws();
                            match self.peek() {
                                Some(Token::RParen) => {
                                    break match saw_comma {
                                        true => Separator::Comma,
                                        false => Separator::Space,
                                    };
                                }
                                _ => {}
                            }
                        };
                        self.skip_ws();
                        match self.peek() {
                            Some(Token::RParen) => { self.advance(); }
                            _ => {}
                        }
                        match items.len() == 1 && !saw_comma {
                            true => {
                                // len == 1 已确认，expect 仅作文档
                                #[allow(clippy::expect_used)]
                                let single_item = items.into_iter().next().expect("items has exactly 1 element");
                                Ok(Value::Paren(Box::new(single_item)))
                            }
                            false => Ok(Value::List(items, sep, false)),
                        }
                    }
                }
            }
            Some(Token::LBracket) => {
                // bracketed list
                self.advance();
                let mut items = Vec::new();
                let mut saw_comma = false;
                loop {
                    self.skip_ws();
                    match self.peek() {
                        Some(Token::RBracket) => break,
                        _ => {}
                    }
                    items.push(self.parse_expr(0)?);
                    self.skip_ws();
                    match self.peek() {
                        Some(Token::Comma) => {
                            self.advance();
                            saw_comma = true;
                        }
                        _ => break,
                    }
                }
                match self.peek() {
                    Some(Token::RBracket) => { self.advance(); }
                    _ => {}
                }
                // 单元素无逗号：如果内部是 List，提升为 bracketed（保留分隔符）
                // 例如 [1 2 3] → List([1,2,3], Space, true) 而非 List([List([1,2,3])], Undecided, true)
                match items.len() == 1 && !saw_comma {
                    true => match items.into_iter().next() {
                        Some(Value::List(inner_items, inner_sep, _)) => {
                            Ok(Value::List(inner_items, inner_sep, true))
                        }
                        other => Ok(Value::List(other.into_iter().collect(), Separator::Undecided, true)),
                    },
                    false => {
                        let sep = match saw_comma {
                            true => Separator::Comma,
                            false => Separator::Space,
                        };
                        Ok(Value::List(items, sep, true))
                    }
                }
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
        match segments.len() {
            1 => {
                // len == 1 已确认，expect 仅作文档
                #[allow(clippy::expect_used)]
                let single = segments.into_iter().next().expect("segments has 1 element");
                match &single {
                    InterpSegment::Expr(_) | InterpSegment::Text(_) => {
                        self.skip_ws();
                        match self.peek() {
                            Some(Token::LParen) => {
                                let expr = match single {
                                    InterpSegment::Expr(e) => e,
                                    InterpSegment::Text(t) => t,
                                };
                                let args = self.parse_args()?;
                                Ok(Value::Call(expr, args))
                            }
                            _ => Ok(Value::Interp(vec![single])),
                        }
                    }
                }
            }
            _ => {
                let joined: String = segments
                    .iter()
                    .map(|seg| match seg {
                        InterpSegment::Expr(e) => e.clone(),
                        InterpSegment::Text(t) => t.clone(),
                    })
                    .collect();
                self.skip_ws();
                match self.peek() {
                    Some(Token::LParen) => {
                        let args = self.parse_args()?;
                        Ok(Value::Call(joined, args))
                    }
                    _ => Ok(Value::Interp(segments)),
                }
            }
        }
    }
}

/// 解析数字字符串为 `Value::Number`。
///
/// 支持科学计数法 `1e15`、`2.5E-10`，单位跟随其后如 `1px`、`3em`。
/// 策略：先整体尝试 f64 parse（覆盖纯数字含指数）；失败则按字符扫描分离数值和单位。
pub(crate) fn parse_number(s: &str) -> Result<Value> {
    // 快速路径：整个字符串作为 f64 解析（处理 1e15、2.5E-10 等科学计数法）
    if let Ok(n) = s.parse::<f64>() {
        return Ok(Value::Number(n, None));
    }
    // 扫描数值部分（含指数 e/E），unit 从第一个非数值字符开始
    let num_end = find_number_end(s);
    let num_str = &s[..num_end];
    let unit = &s[num_end..];
    match num_str.parse::<f64>() {
        Ok(n) => Ok(Value::Number(n, match unit.is_empty() {
            true => None,
            false => Some(unit.to_string()),
        })),
        Err(_) => Err(SassError::Parse {
            expected: "number".into(),
            found: s.to_string(),
        }),
    }
}

/// 找到数值部分的结束位置（含小数点、+/- 号、科学计数法 e/E）。
fn find_number_end(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_ascii_digit() || c == '.' || c == '-' || c == '+' {
            i += 1;
        } else if c == 'e' || c == 'E' {
            // 科学计数法：e/E 后应跟可选 +/- 和至少一位数字
            let next_non_sign = if i + 1 < bytes.len()
                && (bytes[i + 1] == b'+' || bytes[i + 1] == b'-')
            {
                i + 2
            } else {
                i + 1
            };
            if next_non_sign <= bytes.len() {
                // 确认 e 后确实有数字才算指数
                if next_non_sign < bytes.len()
                    && (bytes[next_non_sign] as char).is_ascii_digit()
                {
                    i = next_non_sign + 1;
                    while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
                        i += 1;
                    }
                } else {
                    // 'e' 后没有数字，属于单位前缀
                    break;
                }
            } else {
                break;
            }
        } else if c == '%' {
            // % 既是数值后缀标记也是单位标记，若紧邻数字后则停止扫描
            break;
        } else {
            break;
        }
    }
    i
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
