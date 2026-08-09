//! 值解析——处理表达式和函数调用。

use crate::error::Result;
use crate::lex::token::Token;
use crate::parse::ast::{Separator, Value};
use crate::parse::utils::{parse_hash_color, parse_number};
use super::parser::Parser;

impl<'tok> Parser<'tok> {
    /// 解析值。
    pub(super) fn parse_value(&mut self) -> Result<Value> {
        self.parse_value_inner(false)
    }

    /// 解析值（内部实现）。
    /// `in_args` 为 true 时在逗号处停止（用于函数参数解析）。
    fn parse_value_inner(&mut self, in_args: bool) -> Result<Value> {
        let mut parts = Vec::new();
        while let Some(token) = self.peek() {
            match token {
                Token::Ident(s) => {
                    parts.push(Value::String(s.clone(), false));
                    self.advance();
                }
                Token::Number(s) => {
                    parts.push(parse_number(s)?);
                    self.advance();
                }
                Token::String(s) => {
                    parts.push(Value::String(s.clone(), true));
                    self.advance();
                }
                Token::Hash(s) => {
                    parts.push(Value::Color(parse_hash_color(s)));
                    self.advance();
                }
                Token::Dollar(name) => {
                    parts.push(Value::Variable(name.clone()));
                    self.advance();
                }
                Token::LParen => {
                    // 函数调用或分组
                    let start = self.pos;
                    self.advance();
                    // 检查是否是函数调用（前面是标识符）
                    if let Some(Value::String(name, false)) = parts.last() {
                        let name = name.clone();
                        parts.pop();
                        let args = parse_args(self)?;
                        if self.peek() == Some(&Token::RParen) {
                            self.advance();
                        }
                        parts.push(Value::Call(name, args));
                    } else {
                        // 分组——回退
                        self.pos = start;
                        break;
                    }
                }
                Token::Whitespace => {
                    self.advance();
                }
                Token::Comma => {
                    if in_args {
                        // 在参数列表中，逗号是分隔符，停止解析
                        break;
                    }
                    // 可能是列表
                    self.advance();
                    self.skip_whitespace();
                }
                // 数学运算符
                Token::Plus => {
                    parts.push(Value::String("+".to_string(), false));
                    self.advance();
                }
                Token::Minus => {
                    parts.push(Value::String("-".to_string(), false));
                    self.advance();
                }
                Token::Star => {
                    parts.push(Value::String("*".to_string(), false));
                    self.advance();
                }
                Token::Slash => {
                    parts.push(Value::String("/".to_string(), false));
                    self.advance();
                }
                Token::Percent => {
                    parts.push(Value::String("%".to_string(), false));
                    self.advance();
                }
                _ => break,
            }
        }
        if parts.len() == 1 {
            Ok(parts.into_iter().next().unwrap())
        } else if !parts.is_empty() {
            Ok(Value::List(parts, Separator::Space))
        } else {
            Ok(Value::String(String::new(), false))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lex::Lexer;
    use crate::parse::Parser;

    #[test]
    fn test_parse_rgba_call() {
        let tokens: Vec<_> = Lexer::new("rgba(0, 0, 0, 0.55)")
            .collect::<crate::error::Result<Vec<_>>>()
            .unwrap();
        let mut parser = Parser::new(&tokens);
        let value = parser.parse_value().unwrap();
        eprintln!("Parsed value: {value:?}");
        assert!(format!("{value}").contains("rgba"));
    }
}

/// 解析函数参数列表。
pub(super) fn parse_args(parser: &mut Parser<'_>) -> Result<Vec<Value>> {
    let mut args = Vec::new();
    loop {
        parser.skip_whitespace();
        if parser.peek() == Some(&Token::RParen) {
            break;
        }
        args.push(parser.parse_value_inner(true)?);
        parser.skip_whitespace();
        if parser.peek() == Some(&Token::Comma) {
            parser.advance();
        } else {
            break;
        }
    }
    Ok(args)
}
