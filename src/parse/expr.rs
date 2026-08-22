//! 表达式解析——Pratt parser（优先级爬升法）。
//!
//! 参考 sasspile prefix.rs 逻辑，用 scss-rs 的架构重新实现。
//! parse 阶段构建 AST 级别表达式树（BinOp/UnaryOp/Call 等），
//! 延迟到 eval 阶段求值。

use crate::error::{Result, SassError};
use crate::lex::Token;
use crate::eval::value::{Value, Separator, BinOp, BinOpKind, UnaryOp};
use super::Parser;

impl Parser {
    /// 解析一个值表达式（顶层，到 ; 或 } 停止）。
    /// 用于变量赋值、函数参数等——`/` 做除法。
    pub fn parse_value(&mut self) -> Result<Value> {
        self.parse_value_with_slash(false)
    }

    /// 解析单个表达式（不消费逗号列表）。
    /// 用于函数/ mixin 参数解析——每个参数是一个单独的值。
    pub fn parse_single_value(&mut self) -> Result<Value> {
        self.parse_expr(0, false)
    }

    /// 解析声明值表达式——`/` 作为斜杠分隔符保留。
    /// 用于 CSS 声明值（如 `a {b: 1/2}` → `1/2`）。
    pub fn parse_decl_value(&mut self) -> Result<Value> {
        self.parse_value_with_slash(true)
    }

    /// 解析值表达式核心逻辑。
    /// `slash_as_sep=true` 时，顶层 `/` 被视为斜杠分隔列表而非除法。
    fn parse_value_with_slash(&mut self, slash_as_sep: bool) -> Result<Value> {
        let first = self.parse_expr(0, slash_as_sep)?;
        // 逗号分隔列表
        if matches!(self.peek(), Token::Comma) {
            let mut items = vec![first];
            while matches!(self.peek(), Token::Comma) {
                self.bump();
                items.push(self.parse_expr(0, slash_as_sep)?);
            }
            return Ok(Value::List(items, Separator::Comma, false));
        }
        Ok(first)
    }

    /// Pratt 表达式解析核心。
    /// `min_bp` = 最小绑定优先级（0 = 顶层）。
    /// `slash_as_sep` = 顶层 `/` 是否作为斜杠分隔符。
    fn parse_expr(&mut self, min_bp: u8, slash_as_sep: bool) -> Result<Value> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // 声明值上下文 + 顶层 + 遇到 Slash → 斜杠分隔列表
            // 但 / 前面是括号表达式或变量引用时做除法
            if slash_as_sep && min_bp == 0 && matches!(self.peek(), Token::Slash)
                && !matches!(lhs, Value::Paren(_) | Value::Variable(_))
            {
                // 检查 / 后面是否有算术运算符（+,-,*,%）
                if !self.slash_followed_by_arith_op() {
                    let mut slash_items = vec![lhs.clone()];
                    while matches!(self.peek(), Token::Slash) {
                        self.bump();
                        let item = self.parse_expr(6, false)?;
                        slash_items.push(item);
                    }
                    if slash_items.len() > 1 {
                        lhs = Value::List(slash_items, Separator::SlashLiteral, false);
                    }
                    continue;
                }
            }

            let (op, bp) = match self.peek_binding_power() {
                Some(v) => v,
                None => {
                    // 空格分隔列表——仅顶层
                    if min_bp == 0 && self.is_value_start() {
                        let mut items = vec![lhs.clone()];
                        loop {
                            // 空格列表中的 / 始终作为斜杠分隔符
                            if matches!(self.peek(), Token::Slash) {
                                let last = items.pop().unwrap_or(lhs.clone());
                                let mut slash_items = vec![last];
                                while matches!(self.peek(), Token::Slash) {
                                    self.bump();
                                    let item = self.parse_expr(6, false)?;
                                    slash_items.push(item);
                                }
                                let slash_list = if slash_items.len() > 1 {
                                    Value::List(slash_items, Separator::SlashLiteral, false)
                                } else {
                                    slash_items.into_iter().next().unwrap_or(lhs.clone())
                                };
                                items.push(slash_list);
                                continue;
                            }
                            // 算术运算符在列表内消费
                            if let Some((_, bp)) = self.peek_binding_power() {
                                if bp >= 4 {
                                    let last = items.pop().unwrap_or(lhs.clone());
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
                            continue;
                        }
                    }
                    break;
                }
            };

            if bp < min_bp {
                break;
            }
            self.bump(); // 消费运算符
            let rhs = self.parse_expr(bp + 1, false)?;
            lhs = Value::BinOp(Box::new(BinOp { op, left: lhs, right: rhs }));
        }
        Ok(lhs)
    }

    /// 从已有的 lhs 继续解析二元运算符表达式（不构建空格列表）。
    fn parse_expr_rest(&mut self, mut lhs: Value, min_bp: u8) -> Result<Value> {
        loop {
            let (op, bp) = match self.peek_binding_power() {
                Some(v) => v,
                None => break,
            };
            if bp < min_bp {
                break;
            }
            self.bump();
            let rhs = self.parse_expr(bp + 1, false)?;
            lhs = Value::BinOp(Box::new(BinOp { op, left: lhs, right: rhs }));
        }
        Ok(lhs)
    }

    /// 检查当前 token 是否是值起始 token（排除关键字）。
    fn is_value_start(&self) -> bool {
        if let Token::Ident(s) = self.peek() {
            if matches!(
                s.as_str(),
                "through" | "from" | "to" | "and" | "or" | "not"
                    | "in" | "with" | "show" | "hide" | "as" | "using" | "else"
            ) {
                return false;
            }
        }
        matches!(
            self.peek(),
            Token::Number(_, _)
                | Token::String(_, _)
                | Token::Ident(_)
                | Token::HexColor(_)
                | Token::Variable(_)
                | Token::Interp(_)
                | Token::LParen
                | Token::LBracket
                | Token::Minus
                | Token::Percent
                | Token::True
                | Token::False
                | Token::Null
        )
    }

    /// 检查当前位置的 / 后面是否有 +、-、*、% 算术运算符。
    fn slash_followed_by_arith_op(&self) -> bool {
        let mut i = self.pos + 1; // 跳过当前的 Slash
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Token::Plus => return true,
                Token::Star => return true,
                Token::Minus => {
                    // 简化：空格前后模式不做精确检测
                    // 二元减法 → 有外部算术运算
                    return true;
                }
                Token::Number(_, _) => { i += 1; }
                Token::Slash => { i += 1; }
                Token::Percent => { i += 1; }
                Token::Semicolon | Token::RBrace | Token::Comma | Token::RBracket
                | Token::Eof | Token::Ident(_) | Token::Variable(_)
                | Token::HexColor(_) | Token::Interp(_)
                | Token::LBracket | Token::True | Token::False
                | Token::Null => break,
                _ => { i += 1; }
            }
        }
        false
    }

    /// 运算符绑定优先级。
    fn peek_binding_power(&self) -> Option<(BinOpKind, u8)> {
        match self.peek() {
            Token::Or => Some((BinOpKind::Or, 1)),
            Token::And => Some((BinOpKind::And, 2)),
            Token::Eq => Some((BinOpKind::Eq, 3)),
            Token::NotEq => Some((BinOpKind::NotEq, 3)),
            Token::Gt => Some((BinOpKind::Gt, 3)),
            Token::Lt => Some((BinOpKind::Lt, 3)),
            Token::Gte => Some((BinOpKind::GtEq, 3)),
            Token::Lte => Some((BinOpKind::LtEq, 3)),
            Token::Plus => Some((BinOpKind::Add, 4)),
            Token::Minus => Some((BinOpKind::Sub, 4)),
            Token::Star => Some((BinOpKind::Mul, 5)),
            Token::Slash => Some((BinOpKind::Div, 5)),
            Token::Percent => Some((BinOpKind::Mod, 5)),
            _ => None,
        }
    }

    // —— 前缀解析 —— //

    /// 解析前缀表达式（字面量、变量、括号、一元运算等）。
    fn parse_prefix(&mut self) -> Result<Value> {
        match self.peek().clone() {
            Token::Minus => {
                // 一元负号 vs 厂商前缀标识符
                let next = self.peek_n(1);
                if matches!(
                    next,
                    Token::Number(_, _)
                        | Token::Variable(_)
                        | Token::LParen
                        | Token::HexColor(_)
                ) {
                    self.bump();
                    let val = self.parse_prefix()?;
                    Ok(Value::UnaryOp(UnaryOp::Neg, Box::new(val)))
                } else {
                    // 厂商前缀标识符（如 -webkit-inline-box）
                    self.bump();
                    let mut name = String::from("-");
                    if let Token::Ident(s) = self.peek() {
                        name.push_str(s);
                        self.bump();
                    }
                    // 检查是否是函数调用
                    if matches!(self.peek(), Token::LParen) {
                        let args = self.parse_args()?;
                        Ok(Value::Call(name, args))
                    } else {
                        Ok(Value::Ident(name))
                    }
                }
            }
            Token::Not => {
                self.bump();
                let val = self.parse_prefix()?;
                Ok(Value::UnaryOp(UnaryOp::Not, Box::new(val)))
            }
            Token::Number(n, unit) => {
                self.bump();
                let v: f64 = n.parse().unwrap_or(0.0);
                Ok(Value::Number(v, unit))
            }
            Token::String(s, style) => {
                self.bump();
                Ok(Value::String(s, style))
            }
            Token::HexColor(hex) => {
                self.bump();
                Ok(Value::parse_hex_color(&hex))
            }
            Token::Variable(name) => {
                self.bump();
                Ok(Value::Variable(name))
            }
            Token::Ident(s) => {
                self.bump();
                // module.function() 或 module.$var 语法
                if matches!(self.peek(), Token::Dot) {
                    self.bump();
                    // module.$var
                    if let Token::Variable(var_name) = self.peek() {
                        let var_name = var_name.clone();
                        self.bump();
                        return Ok(Value::Variable(format!("{s}.{var_name}")));
                    }
                    // module.function()
                    if let Token::Ident(member) = self.peek() {
                        let member = member.clone();
                        self.bump();
                        if matches!(self.peek(), Token::LParen) {
                            let args = self.parse_args()?;
                            return Ok(Value::Call(format!("{s}.{member}"), args));
                        }
                        // module.member（非调用）
                        return Ok(Value::Ident(format!("{s}.{member}")));
                    }
                }
                // 函数调用 name(args)
                if matches!(self.peek(), Token::LParen) {
                    let name = s;
                    // CSS 原生函数——原样保留内容
                    if matches!(
                        name.as_str(),
                        "calc" | "clamp" | "env" | "var" | "url" | "css" | "attr"
                    ) {
                        self.bump(); // 消费 (
                        let mut content = String::new();
                        let mut depth = 1;
                        while !self.is_eof() {
                            match self.peek().clone() {
                                Token::LParen => {
                                    depth += 1;
                                    content.push('(');
                                    self.bump();
                                }
                                Token::RParen => {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                    content.push(')');
                                    self.bump();
                                }
                                Token::Comma => {
                                    content.push_str(", ");
                                    self.bump();
                                }
                                Token::Colon => {
                                    content.push_str(": ");
                                    self.bump();
                                }
                                Token::Semicolon => {
                                    content.push_str("; ");
                                    self.bump();
                                }
                                t => {
                                    content.push_str(t.as_str());
                                    if !t.as_str().is_empty() {
                                        content.push(' ');
                                    }
                                    self.bump();
                                }
                            }
                        }
                        if matches!(self.peek(), Token::RParen) {
                            self.bump();
                        }
                        return Ok(Value::Calc(format!("{name}({content})").trim_end().to_string()));
                    }
                    let args = self.parse_args()?;
                    Ok(Value::Call(name, args))
                } else {
                    Ok(Value::Ident(s))
                }
            }
            Token::True => { self.bump(); Ok(Value::Bool(true)) }
            Token::False => { self.bump(); Ok(Value::Bool(false)) }
            Token::Null => { self.bump(); Ok(Value::Null) }
            Token::Interp(s) => {
                self.bump();
                // 相邻的标识符/数字/插值/Hex 拼接为字符串
                let mut parts = vec![s];
                loop {
                    match self.peek().clone() {
                        Token::Ident(t) if !self.is_keyword(&t) => {
                            parts.push(t);
                            self.bump();
                        }
                        Token::Number(n, _) => {
                            parts.push(n);
                            self.bump();
                        }
                        Token::Interp(t) => {
                            parts.push(t);
                            self.bump();
                        }
                        Token::HexColor(h) => {
                            parts.push(format!("#{h}"));
                            self.bump();
                        }
                        _ => break,
                    }
                }
                if parts.len() == 1 {
                    let interp = parts.into_iter().next().unwrap_or_default();
                    // 插值后跟 () → 函数调用
                    if matches!(self.peek(), Token::LParen) {
                        let args = self.parse_args()?;
                        Ok(Value::Call(interp, args))
                    } else {
                        Ok(Value::Interp(interp))
                    }
                } else {
                    let joined = parts.join("");
                    if matches!(self.peek(), Token::LParen) {
                        let args = self.parse_args()?;
                        Ok(Value::Call(joined, args))
                    } else {
                        Ok(Value::Interp(joined))
                    }
                }
            }
            Token::LParen => {
                self.bump();
                // 空列表
                if matches!(self.peek(), Token::RParen) {
                    self.bump();
                    return Ok(Value::List(vec![], Separator::Undecided, false));
                }
                let first = self.parse_expr(0, false)?;
                if matches!(self.peek(), Token::Colon) {
                    // Map
                    self.bump();
                    let val = self.parse_expr(0, false)?;
                    let mut pairs = vec![(first, val)];
                    while matches!(self.peek(), Token::Comma) {
                        self.bump();
                        if matches!(self.peek(), Token::RParen) {
                            break; // 尾随逗号
                        }
                        let k = self.parse_expr(0, false)?;
                        self.eat(&Token::Colon)?;
                        let v = self.parse_expr(0, false)?;
                        pairs.push((k, v));
                    }
                    self.eat(&Token::RParen)?;
                    Ok(Value::Map(pairs))
                } else {
                    // 分组或列表
                    let mut items = vec![first];
                    let mut saw_comma = false;
                    loop {
                        if matches!(self.peek(), Token::Comma) {
                            self.bump();
                            saw_comma = true;
                            if matches!(self.peek(), Token::RParen) {
                                break;
                            }
                        } else if matches!(self.peek(), Token::RParen) {
                            break;
                        } else if self.is_value_start() {
                            items.push(self.parse_expr(0, false)?);
                        } else {
                            break;
                        }
                    }
                    self.eat(&Token::RParen)?;
                    let sep = if saw_comma { Separator::Comma } else { Separator::Space };
                    if items.len() == 1 && !saw_comma {
                        Ok(Value::Paren(Box::new(items.into_iter().next().unwrap())))
                    } else {
                        Ok(Value::List(items, sep, false))
                    }
                }
            }
            Token::LBracket => {
                self.bump();
                let mut items = Vec::new();
                let mut saw_comma = false;
                if !matches!(self.peek(), Token::RBracket) {
                    items.push(self.parse_expr(0, false)?);
                    while matches!(self.peek(), Token::Comma) {
                        self.bump();
                        saw_comma = true;
                        if matches!(self.peek(), Token::RBracket) {
                            break;
                        }
                        items.push(self.parse_expr(0, false)?);
                    }
                }
                self.eat(&Token::RBracket)?;
                let sep = if saw_comma { Separator::Comma }
                    else if items.len() <= 1 { Separator::Undecided }
                    else { Separator::Space };
                Ok(Value::List(items, sep, true))
            }
            Token::Percent => {
                // % 作为独立值 = 字符串 %
                self.bump();
                Ok(Value::Ident("%".to_string()))
            }
            Token::Ampersand => {
                self.bump();
                Ok(Value::Ident("&".to_string()))
            }
            Token::Star => {
                self.bump();
                Ok(Value::Ident("*".to_string()))
            }
            t => {
                // 终止符 → Null
                match &t {
                    Token::RBrace | Token::RParen | Token::Semicolon | Token::RBracket
                    | Token::Comma | Token::Eof => Ok(Value::Null),
                    _ => Err(SassError::parse(format!(
                        "Unexpected token in expression: {t:?}"
                    ))),
                }
            }
        }
    }

    /// 检查字符串是否为 SCSS 关键字。
    fn is_keyword(&self, s: &str) -> bool {
        matches!(
            s,
            "through" | "from" | "to" | "and" | "or" | "not" | "in" | "with"
                | "show" | "hide" | "as" | "using" | "else"
        )
    }
}
