//! Pratt 表达式解析 + 数值/颜色解析。
//!
//! 包含 parse_expr/is_value_start/parse_prefix/peek_binding_power 方法
//! 和 parse_number/parse_hash_color/hex2/hex1 自由函数。

use super::Parser;
use super::ast::*;
use crate::error::{Result, SassError};
use crate::lex::token::Token;

impl<'tok> Parser<'tok> {
    // —— Pratt 表达式解析 ——
    /// 解析值表达式（顶层，到 ; 或 } 停止）。
    pub fn parse_value(&mut self) -> Result<Value> {
        let first = self.parse_expr(0)?;
        self.skip_ws();
        // 逗号分隔列表
        if self.peek() == Some(&Token::Comma) {
            let mut items = vec![first];
            while self.peek() == Some(&Token::Comma) {
                self.advance();
                self.skip_ws();
                items.push(self.parse_expr(0)?);
                self.skip_ws();
            }
            return Ok(Value::List(items, Separator::Comma, false));
        }
        Ok(first)
    }

    pub(crate) fn parse_expr(&mut self, min_bp: u8) -> Result<Value> {
        self.skip_ws();
        let mut lhs = self.parse_prefix()?;
        loop {
            self.skip_ws();
            let (op, bp) = match self.peek_binding_power() {
                Some(v) => v,
                None => {
                    // 空格分隔列表——仅顶层（min_bp=0）
                    if min_bp == 0 && self.is_value_start() {
                        let mut items = vec![lhs.clone()];
                        loop {
                            self.skip_ws();
                            // 先检查二元运算符——仅算术运算符（+,-,*,/,%）才在列表内消费
                            // 低优先级运算符（or,and,==,!=,<,>,<=,>=）不在此消费
                            if let Some((_, bp)) = self.peek_binding_power() {
                                if bp >= 4 {
                                    let last = items.pop().unwrap();
                                    let binop_result = self.parse_expr_rest(last, 4)?;
                                    items.push(binop_result);
                                    continue;
                                }
                            }
                            if !self.is_value_start() {
                                break;
                            }
                            items.push(self.parse_prefix()?);
                        }
                        if items.len() > 1 {
                            lhs = Value::List(items, Separator::Space, false);
                            continue; // 继续检查后续运算符（如 / ）
                        }
                    }
                    break;
                }
            };
            if bp < min_bp {
                break;
            }
            self.advance(); // 消费运算符
            self.skip_ws();
            let rhs = self.parse_expr(bp + 1)?;
            lhs = Value::BinOp(Box::new(BinOp {
                op,
                left: lhs,
                right: rhs,
            }));
        }
        Ok(lhs)
    }

    /// 从已有的 lhs 继续解析二元运算符表达式（不构建空格列表）。
    fn parse_expr_rest(&mut self, mut lhs: Value, min_bp: u8) -> Result<Value> {
        loop {
            self.skip_ws();
            let (op, bp) = match self.peek_binding_power() {
                Some(v) => v,
                None => break,
            };
            if bp < min_bp {
                break;
            }
            self.advance();
            self.skip_ws();
            let rhs = self.parse_expr(bp + 1)?;
            lhs = Value::BinOp(Box::new(BinOp {
                op,
                left: lhs,
                right: rhs,
            }));
        }
        Ok(lhs)
    }

    /// 检查字符串是否为 SCSS 关键字（不应被拼接进字符串）。
    fn is_keyword(&self, s: &str) -> bool {
        matches!(
            s,
            "through" | "from" | "to" | "and" | "or" | "not" | "in" | "with"
                | "show" | "hide" | "as" | "using" | "else"
        )
    }

    /// 检查当前 token 是否是值起始 token（排除关键字）。
    pub(crate) fn is_value_start(&self) -> bool {
        if let Some(Token::Ident(s)) = self.peek() {
            if matches!(
                s.as_str(),
                "through"
                    | "from"
                    | "to"
                    | "and"
                    | "or"
                    | "not"
                    | "in"
                    | "with"
                    | "show"
                    | "hide"
                    | "as"
                    | "using"
                    | "else"
            ) {
                return false;
            }
        }
        matches!(
            self.peek(),
            Some(Token::Number(_))
                | Some(Token::String(_, _))
                | Some(Token::Ident(_))
                | Some(Token::Hash(_))
                | Some(Token::Dollar(_))
                | Some(Token::Interp(_))
                | Some(Token::LParen)
                | Some(Token::LBracket)
                | Some(Token::Minus)
                | Some(Token::Percent)
                | Some(Token::True)
                | Some(Token::False)
                | Some(Token::Null)
        )
    }

    pub(crate) fn parse_prefix(&mut self) -> Result<Value> {
        self.skip_ws();
        match self.peek() {
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
                    if matches!(name.as_str(), "calc" | "clamp" | "env" | "var" | "url")
                        && !is_url_with_string
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
                    // CSS Level 4: rgb(R G B / A) — 空格分隔 + / alpha 分隔符
                    if matches!(name.as_str(), "rgb" | "rgba") {
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
                                .map(|v| Arg { name: None, value: v, spread: false })
                                .collect();
                            if let Some(a) = alpha {
                                args.push(Arg { name: None, value: a, spread: false });
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
                    Ok(Value::Interp(parts.into_iter().next().unwrap()))
                } else {
                    Ok(Value::Interp(parts.join("")))
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
                    if items.len() == 1 && !saw_comma {
                        Ok(items.into_iter().next().unwrap())
                    } else {
                        Ok(Value::List(items, sep, false))
                    }
                }
            }
            Some(Token::Not) => {
                self.advance();
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
                let sep = if saw_comma { Separator::Comma }
                    else if items.len() <= 1 { Separator::Undecided }
                    else { Separator::Space };
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
                    Some(t)
                        if matches!(
                            t,
                            Token::RBrace
                                | Token::RParen
                                | Token::Semicolon
                                | Token::RBracket
                                | Token::Comma
                        ) =>
                    {
                        Ok(Value::Null)
                    }
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

    pub(crate) fn peek_binding_power(&self) -> Option<(BinOpKind, u8)> {
        match self.peek() {
            Some(Token::Or) => Some((BinOpKind::Or, 1)),
            Some(Token::And) => Some((BinOpKind::And, 2)),
            Some(Token::Eq) => Some((BinOpKind::Eq, 3)),
            Some(Token::NotEq) => Some((BinOpKind::NotEq, 3)),
            Some(Token::Less) => Some((BinOpKind::Lt, 3)),
            Some(Token::Greater) => Some((BinOpKind::Gt, 3)),
            Some(Token::LessEq) => Some((BinOpKind::LtEq, 3)),
            Some(Token::GreaterEq) => Some((BinOpKind::GtEq, 3)),
            Some(Token::Plus) => Some((BinOpKind::Add, 4)),
            Some(Token::Minus) => {
                // Sass 规则：space-before + no-space-after + Number = 一元负号
                let has_ws_before = self.pos > 0
                    && matches!(self.tokens.get(self.pos - 1), Some(Token::Whitespace));
                let next = self.tokens.get(self.pos + 1);
                let has_ws_after = matches!(next, Some(Token::Whitespace) | None);
                if has_ws_before && !has_ws_after && matches!(next, Some(Token::Number(_))) {
                    None // 一元负号，不是二元运算符
                } else {
                    Some((BinOpKind::Sub, 4))
                }
            }
            Some(Token::Star) => Some((BinOpKind::Mul, 5)),
            Some(Token::Slash) => Some((BinOpKind::Div, 5)),
            Some(Token::Percent) => Some((BinOpKind::Mod, 5)),
            _ => None,
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
        4 => Color::rgba(
            hex2(bytes[1], bytes[1]),
            hex2(bytes[2], bytes[2]),
            hex2(bytes[3], bytes[3]),
            hex1(bytes[0]) as f32 / 15.0,
        ),
        6 => Color::rgb(
            hex2(bytes[0], bytes[1]),
            hex2(bytes[2], bytes[3]),
            hex2(bytes[4], bytes[5]),
        ),
        8 => Color::rgba(
            hex2(bytes[0], bytes[1]),
            hex2(bytes[2], bytes[3]),
            hex2(bytes[4], bytes[5]),
            hex2(bytes[6], bytes[7]) as f32 / 255.0,
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
