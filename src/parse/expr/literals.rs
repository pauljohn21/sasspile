//! 字面量解析——Number/String/Hash/Dollar/Ident/True/False/Null/Interp 等。
//!
//! `parse_literal` 处理简单值 token，`parse_prefix` 保留控制流（Minus/Not/LParen 等）。

use super::super::ast::*;
use super::super::Parser;
use crate::error::Result;
use crate::lex::token::Token;
use super::{parse_hash_color, parse_number};

impl<'tok> Parser<'tok> {
    /// 解析字面量值——Number/String/Hash/Dollar/Ident/True/False/Null/Interp 等。
    ///
    /// 返回 `Some(Value)` 当当前 token 是字面量，`None` 当不是（由 `parse_prefix` 处理控制流）。
    pub(crate) fn parse_literal(&mut self) -> Result<Option<Value>> {
        match self.peek() {
            Some(Token::Number(s)) => {
                let v = parse_number(s)?;
                self.advance();
                Ok(Some(v))
            }
            Some(Token::String(s, q)) => {
                let v = Value::String(s.clone(), *q == '"' || *q == '\'');
                self.advance();
                Ok(Some(v))
            }
            Some(Token::Hash(s)) => {
                let v = Value::Color(parse_hash_color(s));
                self.advance();
                Ok(Some(v))
            }
            Some(Token::Dollar(name)) => {
                let v = Value::Variable(name.clone());
                self.advance();
                Ok(Some(v))
            }
            Some(Token::Ident(s)) => {
                let name = s.clone();
                self.advance();
                Ok(Some(self.parse_ident_followup(name)?))
            }
            Some(Token::Interp(s)) => {
                let segments = vec![crate::parse::ast::InterpSegment::Expr(s.clone())];
                self.advance();
                self.parse_interp_adjacent(segments).map(Some)
            }
            Some(Token::True) => {
                self.advance();
                Ok(Some(Value::Bool(true)))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Some(Value::Bool(false)))
            }
            Some(Token::Null) => {
                self.advance();
                Ok(Some(Value::Null))
            }
            Some(Token::Amp) => {
                let v = Value::String("&".to_string(), false);
                self.advance();
                Ok(Some(v))
            }
            Some(Token::Star) => {
                let v = Value::String("*".to_string(), false);
                self.advance();
                Ok(Some(v))
            }
            _ => Ok(None),
        }
    }

    /// 处理 Ident 后的后续 token——插值拼接、模块访问、函数调用、裸标识符。
    fn parse_ident_followup(&mut self, name: String) -> Result<Value> {
        // 检查 ident 后是否紧跟 Interp（无空格分隔）——拼接为插值片段
        // 例如 hey#{$y}ho → [Text("hey"), Expr("$y"), Text("ho")]
        if let Some(Token::Interp(interp_content)) = self.peek() {
            let interp_content = interp_content.clone();
            self.advance();
            return self.parse_interp_adjacent(
                vec![crate::parse::ast::InterpSegment::Text(name),
                     crate::parse::ast::InterpSegment::Expr(interp_content)],
            );
        }
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
            if (name.eq_ignore_ascii_case("calc") || name.eq_ignore_ascii_case("clamp")
                || name.eq_ignore_ascii_case("env") || name.eq_ignore_ascii_case("var")
                || name == "url" || name == "css" || name == "attr")
                && !is_url_with_string
            {
                self.advance(); // 消费 (
                // 检查是否为空参数——calc() 等空参数可能是用户函数调用
                let after = self.peek();
                let is_special = name.eq_ignore_ascii_case("calc")
                    || name.eq_ignore_ascii_case("clamp")
                    || name.eq_ignore_ascii_case("env")
                    || name.eq_ignore_ascii_case("var");
                if matches!(after, Some(Token::RParen)) && is_special {
                    self.advance(); // 消费 )
                    return Ok(Value::Call(name, Vec::new()));
                }
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
                    // 空格分隔参数包装为列表——保留分隔符信息
                    let channels = Value::List(items, Separator::Space, false);
                    let mut args: Vec<Arg> = vec![
                        Arg { name: None, value: channels, spread: false, condition: None },
                    ];
                    if let Some(a) = alpha {
                        args.push(Arg { name: None, value: a, spread: false, condition: None });
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
}
