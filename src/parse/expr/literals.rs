//! 字面量解析——Number/String/Hash/Dollar/Ident/True/False/Null/Interp 等。
//!
//! `parse_literal` 处理简单值 token，`parse_prefix` 保留控制流（Minus/Not/LParen 等）。

use super::super::ParseStream;
use super::super::ast::*;
use super::{parse_hash_color, parse_number};
use crate::error::Result;
use crate::lex::token::Token;

impl ParseStream<'_> {
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
                let name = crate::parse::ast::Value::decode_css_escapes(s);
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
            // and/or/not 关键字可作为函数调用（如 AND()）
            Some(Token::And | Token::Or | Token::Not) => {
                let kw = self.peek().cloned();
                let name = match kw {
                    Some(Token::And) => "and",
                    Some(Token::Or) => "or",
                    Some(Token::Not) => "not",
                    _ => unreachable!(),
                }
                .to_string();
                self.advance();
                // 如果后跟 (，解析为函数调用
                self.skip_ws();
                match self.peek() {
                    Some(Token::LParen) => {
                        let args = self.parse_args()?;
                        Ok(Some(Value::Call(name, args)))
                    }
                    _ => {
                        // 裸关键字作为字符串值（如 CSS 中 and 作为普通值）
                        Ok(Some(Value::String(name, false)))
                    }
                }
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
        match self.peek() {
            Some(Token::Interp(interp_content)) => {
                let interp_content = interp_content.clone();
                self.advance();
                return self.parse_interp_adjacent(vec![
                    crate::parse::ast::InterpSegment::Text(name),
                    crate::parse::ast::InterpSegment::Expr(interp_content),
                ]);
            }
            _ => {}
        }
        self.skip_ws();
        // 检查 module.function() 或 module.$var 语法
        match self.peek() {
            Some(&Token::Dot) => {
                self.advance();
                self.skip_ws();
                // module.$var
                match self.peek() {
                    Some(Token::Dollar(var_name)) => {
                        let var_name = var_name.clone();
                        self.advance();
                        // 私有成员检查：以下划线开头的变量不能从外部访问
                        match var_name.starts_with('_') {
                            true => {
                                return Err(crate::error::SassError::Eval(
                                    "Private members can't be accessed from outside their modules.".into(),
                                ));
                            }
                            false => {}
                        }
                        return Ok(Value::Variable(format!("{name}.{var_name}")));
                    }
                    _ => {}
                }
                match self.peek() {
                    Some(Token::Ident(member)) => {
                        let member = member.clone();
                        self.advance();
                        // 私有成员检查：以下划线开头的函数不能从外部访问
                        match member.starts_with('_') {
                            true => {
                                return Err(crate::error::SassError::Eval(
                                    "Private members can't be accessed from outside their modules.".into(),
                                ));
                            }
                            false => {}
                        }
                        self.skip_ws();
                        // module.function()
                        match self.peek() {
                            Some(&Token::LParen) => {
                                let args = self.parse_args()?;
                                return Ok(Value::Call(format!("{name}.{member}"), args));
                            }
                            _ => {}
                        }
                        // module.member（非调用）— 裸命名空间标识符无效
                        return Err(crate::error::SassError::Parse {
                            expected: "(".into(),
                            found: "other".into(),
                        });
                    }
                    _ => {}
                }
                // 点号后既不是 $var 也不是 Ident——报 "Expected identifier."
                return Err(crate::error::SassError::Eval("Expected identifier.".into()));
            }
            Some(&Token::LParen) => {
                // CSS 原生函数——原样保留内容，不解析参数
                // url() 特殊处理：字符串参数走正常解析（支持插值），裸 URL 走 raw
                let is_url_with_string = name == "url" && {
                    let next = self.tokens.get(self.pos + 1);
                    matches!(next, Some(Token::String(_, _)))
                };
                match (name.eq_ignore_ascii_case("calc")
                    || name.eq_ignore_ascii_case("clamp")
                    || name.eq_ignore_ascii_case("env")
                    || name.eq_ignore_ascii_case("var")
                    || name == "url"
                    || name == "css"
                    || name == "attr")
                    && !is_url_with_string
                {
                    true => {
                        self.advance(); // 消费 (
                        // 检查是否为空参数——calc() 等空参数可能是用户函数调用
                        let after = self.peek();
                        let is_special = name.eq_ignore_ascii_case("calc")
                            || name.eq_ignore_ascii_case("clamp")
                            || name.eq_ignore_ascii_case("env")
                            || name.eq_ignore_ascii_case("var");
                        match matches!(after, Some(&Token::RParen)) && is_special {
                            true => {
                                self.advance(); // 消费 )
                                return Ok(Value::Call(name, Vec::new()));
                            }
                            false => {}
                        }
                        // EP FIX: For var(), try structured arg parsing so that
                        // Sass expressions in the fallback (e.g. map.get()) get
                        // evaluated at eval time instead of being preserved as raw text.
                        // parse_args_prefix parses args WITHOUT requiring leading LParen.
                        if name == "var" {
                            let save_pos = self.pos;
                            if let Ok(args) = self.parse_args_prefix() {
                                self.skip_ws();
                                if matches!(self.peek(), Some(&Token::RParen)) {
                                    self.advance(); // 消费 )
                                    return Ok(Value::Call(name, args));
                                }
                            }
                            self.pos = save_pos;
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
                                    match depth == 0 {
                                        true => break,
                                        false => {}
                                    }
                                    content.push(')');
                                    self.advance();
                                }
                                Token::Whitespace => {
                                    content.push(' ');
                                    self.advance();
                                }
                                Token::Interp(s) => {
                                    // 插值在 CSS 函数中——保留 #{} 供 eval 层展开
                                    // 必须保留 #{} 标记，否则 eval 层无法区分插值与原始文本
                                    content.push_str("#{");
                                    content.push_str(s);
                                    content.push('}');
                                    self.advance();
                                }
                                _ => {
                                    content.push_str(&t.to_string());
                                    self.advance();
                                }
                            }
                        }
                        self.skip_ws();
                        match self.peek() {
                            Some(&Token::RParen) => { self.advance(); }
                            _ => {}
                        }
                        return Ok(Value::Calc(format!("{name}({content})")));
                    }
                    false => {}
                }
                // CSS Level 4: rgb(R G B / A), hsl(H S L / A), hwb(H W B / A)
                // — 空格分隔 + / alpha 分隔符
                match matches!(name.as_str(), "rgb" | "rgba" | "hsl" | "hsla" | "hwb") {
                    true => {
                        let save_pos = self.pos;
                        self.advance(); // 消费 (
                        self.skip_ws();
                        let first = self.parse_prefix()?;
                        self.skip_ws();
                        // 检测是否为空格分隔语法（非逗号、非右括号）
                        match self.is_value_start() || self.peek() == Some(&Token::Slash) {
                            true => {
                                let mut items = vec![first];
                                while self.is_value_start() {
                                    items.push(self.parse_prefix()?);
                                    self.skip_ws();
                                }
                                let alpha = match self.peek() {
                                    Some(&Token::Slash) => {
                                        self.advance();
                                        self.skip_ws();
                                        Some(self.parse_prefix()?)
                                    }
                                    _ => None,
                                };
                                self.skip_ws();
                                match self.peek() {
                                    Some(&Token::RParen) => { self.advance(); }
                                    _ => {}
                                }
                                // 空格分隔参数包装为列表——保留分隔符信息
                                let channels = Value::List(items, Separator::Space, false);
                                let mut args: Vec<Arg> = vec![Arg {
                                    name: None,
                                    value: channels,
                                    spread: false,
                                    condition: None,
                                }];
                                match alpha {
                                    Some(a) => {
                                        args.push(Arg {
                                            name: None,
                                            value: a,
                                            spread: false,
                                            condition: None,
                                        });
                                    }
                                    None => {}
                                }
                                return Ok(Value::Call(name, args));
                            }
                            false => {}
                        }
                        // 不是空格分隔语法——回退到标准 parse_args
                        self.pos = save_pos;
                    }
                    false => {}
                }
                let args = self.parse_args()?;
                Ok(Value::Call(name, args))
            }
            _ => Ok(Value::String(name, false)),
        }
    }
}
