use super::super::Parser;
use super::super::ast::*;
use crate::error::{Result, SassError};
use crate::lex::token::Token;

impl<'tok> Parser<'tok> {
    pub(crate) fn parse_prefix(&mut self) -> Result<Value> {
        self.skip_ws();
        match self.peek() {
            Some(Token::Plus) => {
                // 一元正号：+0, +1, +$var 等
                let next = self.tokens.get(self.pos + 1);
                if matches!(
                    next,
                    Some(Token::Number(_))
                        | Some(Token::Dollar(_))
                        | Some(Token::LParen)
                        | Some(Token::Hash(_))
                ) {
                    self.advance();
                    self.skip_ws();
                    self.parse_prefix()
                } else {
                    self.advance();
                    Ok(Value::String("+".to_string(), false))
                }
            }
            Some(Token::Minus) => {
                // 一元负号：当后面是数字、变量、括号表达式时
                // 标识符前的 - 是 CSS 厂商前缀（如 -webkit-inline-box）
                let next = self.tokens.get(self.pos + 1);
                if matches!(
                    next,
                    Some(Token::Number(_))
                        | Some(Token::Dollar(_))
                        | Some(Token::LParen)
                        | Some(Token::Hash(_))
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
            Some(Token::Number(s)) => {
                let v = parse_number(s)?;
                self.advance();
                Ok(v)
            }
            Some(Token::String(s, q)) => {
                let v = Value::String(s.clone(), *q == '"' || *q == '\'');
                self.advance();
                Ok(v)
            }
            Some(Token::Hash(s)) => {
                let v = Value::Color(parse_hash_color(s));
                self.advance();
                Ok(v)
            }
            Some(Token::Dollar(name)) => {
                let v = Value::Variable(name.clone());
                self.advance();
                Ok(v)
            }
            Some(Token::Ident(s)) => {
                let name = s.clone();
                self.advance();
                self.skip_ws();
                // 检查 module.function() 或 module.$var 语法
                if self.peek() == Some(&Token::Dot) {
                    self.advance();
                    self.skip_ws();
                    // module.$var
                    if let Some(Token::Dollar(var_name)) = self.peek() {
                        let var_name = var_name.clone();
                        self.advance();
                        return Ok(Value::Variable(format!("{name}.{var_name}")));
                    }
                    if let Some(Token::Ident(member)) = self.peek() {
                        let member = member.clone();
                        self.advance();
                        self.skip_ws();
                        // module.function()
                        if self.peek() == Some(&Token::LParen) {
                            let args = self.parse_args()?;
                            return Ok(Value::Call(format!("{name}.{member}"), args));
                        }
                        // module.member（非调用）
                        return Ok(Value::String(format!("{name}.{member}"), false));
                    }
                }
                if self.peek() == Some(&Token::LParen) {
                    // CSS 原生函数——原样保留内容，不解析参数
                    // url() 特殊处理：字符串参数走正常解析（支持插值），裸 URL 走 raw
                    let is_url_with_string = name == "url" && {
                        let next = self.tokens.get(self.pos + 1);
                        matches!(next, Some(Token::String(_, _)))
                    };
                    if matches!(
                        name.as_str(),
                        "calc" | "clamp" | "env" | "var" | "url" | "css" | "attr"
                    ) && !is_url_with_string
                    {
                        self.advance(); // 消费 (
                        let mut content = String::new();
                        let mut depth = 1;
                        while let Some(t) = self.peek() {
                            match t {
                                Token::LParen => {
                                    depth += 1;
                                    content.push('(');
                                    self.advance();
                                }
                                Token::RParen => {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                    content.push(')');
                                    self.advance();
                                }
                                Token::Whitespace => {
                                    content.push(' ');
                                    self.advance();
                                }
                                Token::Interp(s) => {
                                    // 插值在 CSS 函数中——求值变量后输出
                                    content.push_str(s);
                                    self.advance();
                                }
                                _ => {
                                    content.push_str(&t.to_string());
                                    self.advance();
                                }
                            }
                        }
                        self.skip_ws();
                        if self.peek() == Some(&Token::RParen) {
                            self.advance();
                        }
                        return Ok(Value::Calc(format!("{name}({content})")));
                    }
                    // CSS Level 4: rgb(R G B / A), hsl(H S L / A), hwb(H W B / A)
                    // — 空格分隔 + / alpha 分隔符
                    if matches!(name.as_str(), "rgb" | "rgba" | "hsl" | "hsla" | "hwb") {
                        let save_pos = self.pos;
                        self.advance(); // 消费 (
                        self.skip_ws();
                        let first = self.parse_prefix()?;
                        self.skip_ws();
                        // 检测是否为空格分隔语法（非逗号、非右括号）
                        if self.is_value_start() || self.peek() == Some(&Token::Slash) {
                            let mut items = vec![first];
                            while self.is_value_start() {
                                items.push(self.parse_prefix()?);
                                self.skip_ws();
                            }
                            let alpha = if self.peek() == Some(&Token::Slash) {
                                self.advance();
                                self.skip_ws();
                                Some(self.parse_prefix()?)
                            } else {
                                None
                            };
                            self.skip_ws();
                            if self.peek() == Some(&Token::RParen) {
                                self.advance();
                            }
                            let mut args: Vec<Arg> = items
                                .into_iter()
                                .map(|v| Arg {
                                    name: None,
                                    value: v,
                                    spread: false,
                                    condition: None,
                                })
                                .collect();
                            if let Some(a) = alpha {
                                args.push(Arg {
                                    name: None,
                                    value: a,
                                    spread: false,
                                    condition: None,
                                });
                            }
                            return Ok(Value::Call(name, args));
                        }
                        // 不是空格分隔语法——回退到标准 parse_args
                        self.pos = save_pos;
                    }
                    let args = self.parse_args()?;
                    Ok(Value::Call(name, args))
                } else {
                    Ok(Value::String(name, false))
                }
            }
            Some(Token::Interp(s)) => {
                let mut parts = vec![s.clone()];
                self.advance();
                // 相邻的标识符/数字/插值/Hash（无空格分隔）应拼接为字符串
                // 例如 #{divide(...)}rem → "divide(...)rem"
                loop {
                    match self.peek() {
                        Some(Token::Ident(t)) if !self.is_keyword(t) => {
                            parts.push(t.clone());
                            self.advance();
                        }
                        Some(Token::Number(n)) => {
                            parts.push(n.clone());
                            self.advance();
                        }
                        Some(Token::Interp(t)) => {
                            parts.push(t.clone());
                            self.advance();
                        }
                        Some(Token::Hash(h)) => {
                            parts.push(format!("#{h}"));
                            self.advance();
                        }
                        _ => break,
                    }
                }
                if parts.len() == 1 {
                    let interp = parts.into_iter().next().unwrap();
                    // 插值后跟 () → 函数调用（如 #{css}() → Call("css", [])）
                    self.skip_ws();
                    if self.peek() == Some(&Token::LParen) {
                        let args = self.parse_args()?;
                        Ok(Value::Call(interp, args))
                    } else {
                        Ok(Value::Interp(interp))
                    }
                } else {
                    let joined = parts.join("");
                    self.skip_ws();
                    if self.peek() == Some(&Token::LParen) {
                        let args = self.parse_args()?;
                        Ok(Value::Call(joined, args))
                    } else {
                        Ok(Value::Interp(joined))
                    }
                }
            }
            Some(Token::True) => {
                self.advance();
                Ok(Value::Bool(true))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Value::Bool(false))
            }
            Some(Token::Null) => {
                self.advance();
                Ok(Value::Null)
            }
            Some(Token::LParen) => {
                self.advance();
                self.paren_depth += 1; // 进入括号——/ 在括号内是除法
                self.skip_ws();
                // 空 Map 或列表
                if self.peek() == Some(&Token::RParen) {
                    self.advance();
                    self.paren_depth -= 1;
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
                        // 重复键检测：未求值的键若与已有键结构相同则报错
                        if pairs.iter().any(|(ek, _)| ek == &k) {
                            self.paren_depth -= 1;
                            return Err(SassError::Eval("Duplicate key.".into()));
                        }
                        self.skip_ws();
                        self.expect(&Token::Colon)?;
                        self.skip_ws();
                        let v = self.parse_expr(0)?;
                        pairs.push((k, v));
                        self.skip_ws();
                    }
                    self.expect(&Token::RParen)?;
                    self.paren_depth -= 1;
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
                            Some(Token::Number(_))
                            | Some(Token::String(_, _))
                            | Some(Token::Ident(_))
                            | Some(Token::Hash(_))
                            | Some(Token::Dollar(_))
                            | Some(Token::Interp(_))
                            | Some(Token::LParen) => {
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
                    self.paren_depth -= 1;
                    if items.len() == 1 && !saw_comma {
                        Ok(Value::Paren(Box::new(items.into_iter().next().unwrap())))
                    } else {
                        Ok(Value::List(items, sep, false))
                    }
                }
            }
            Some(Token::Not) => {
                // `not` 后紧跟 `(` 没有空白 → 报错 "Whitespace is required"
                // 例如 `not(css())` 报错，`not (css())` 合法
                let next = self.tokens.get(self.pos + 1);
                if matches!(next, Some(Token::LParen)) {
                    return Err(SassError::Parse {
                        expected: "Whitespace is required after not".into(),
                        found: "(".into(),
                    });
                }
                self.advance();
                self.skip_ws();
                let v = self.parse_prefix()?;
                // `not not` 是语法错误（不能连续两个 not）
                if matches!(v, Value::UnaryOp(UnaryOp::Not, _)) {
                    return Err(SassError::Parse {
                        expected: "(".into(),
                        found: "not".into(),
                    });
                }
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
                let sep = if saw_comma {
                    Separator::Comma
                } else if items.len() <= 1 {
                    Separator::Undecided
                } else {
                    Separator::Space
                };
                Ok(Value::List(items, sep, true))
            }
            Some(Token::Amp) => {
                let v = Value::String("&".to_string(), false);
                self.advance();
                Ok(v)
            }
            Some(Token::Star) => {
                let v = Value::String("*".to_string(), false);
                self.advance();
                Ok(v)
            }
            Some(Token::Percent) => {
                // % 作为独立值 = 字符串 %
                self.advance();
                Ok(Value::String("%".to_string(), false))
            }
            _ => {
                // 尝试解析为标识符字符串——但不消费终止符
                match self.peek() {
                    Some(
                        Token::RBrace
                        | Token::RParen
                        | Token::Semicolon
                        | Token::RBracket
                        | Token::Comma,
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
}

/// 解析数字字符串为 Value::Number。
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
            expected: "数字".into(),
            found: s.to_string(),
        }),
    }
}

/// 解析 #hash 字符串为 Color。
pub(crate) fn parse_hash_color(s: &str) -> Color {
    let bytes = s.as_bytes();
    match bytes.len() {
        3 => Color::rgb(
            hex2(bytes[0], bytes[0]),
            hex2(bytes[1], bytes[1]),
            hex2(bytes[2], bytes[2]),
        ),
        4 => {
            // #RGBA 格式：每个字符复制一份（如 #AbCd → R=AA G=BB B=CC A=DD）
            let r = hex2(bytes[0], bytes[0]);
            let g = hex2(bytes[1], bytes[1]);
            let b = hex2(bytes[2], bytes[2]);
            let a = round_alpha(hex2(bytes[3], bytes[3]) as f64 / 255.0);
            Color::rgba(r, g, b, a)
        }
        6 => Color::rgb(
            hex2(bytes[0], bytes[1]),
            hex2(bytes[2], bytes[3]),
            hex2(bytes[4], bytes[5]),
        ),
        8 => {
            // #RRGGBBAA 格式
            let r = hex2(bytes[0], bytes[1]);
            let g = hex2(bytes[2], bytes[3]);
            let b = hex2(bytes[4], bytes[5]);
            let a = round_alpha(hex2(bytes[6], bytes[7]) as f64 / 255.0);
            Color::rgba(r, g, b, a)
        }
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
