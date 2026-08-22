//! 表达式解析（值）。

use crate::error::{Result, SassError};
use crate::lex::Token;
use crate::eval::value::{Value, Separator};
use super::Parser;

impl Parser {
    /// 解析一个值（表达式）。
    pub fn parse_value(&mut self) -> Result<Value> {
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> Result<Value> {
        let cond = self.parse_or()?;
        // SCSS 不支持三元运算符，但支持 if() 函数
        Ok(cond)
    }

    fn parse_or(&mut self) -> Result<Value> {
        let mut left = self.parse_and()?;
        while self.is_ident("or") {
            self.bump();
            let right = self.parse_and()?;
            left = Value::or(left, right);
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Value> {
        let mut left = self.parse_not()?;
        while self.is_ident("and") {
            self.bump();
            let right = self.parse_not()?;
            left = Value::and(left, right);
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Value> {
        if self.is_ident("not") {
            self.bump();
            let val = self.parse_comparison()?;
            return Ok(Value::not(val));
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Value> {
        let left = self.parse_additive()?;
        match self.peek() {
            Token::Eq => { self.bump(); let r = self.parse_additive()?; Ok(Value::eq(left, r)) }
            Token::NotEq => { self.bump(); let r = self.parse_additive()?; Ok(Value::ne(left, r)) }
            Token::Gt => { self.bump(); let r = self.parse_additive()?; Ok(Value::gt(left, r)) }
            Token::Gte => { self.bump(); let r = self.parse_additive()?; Ok(Value::gte(left, r)) }
            Token::Lt => { self.bump(); let r = self.parse_additive()?; Ok(Value::lt(left, r)) }
            Token::Lte => { self.bump(); let r = self.parse_additive()?; Ok(Value::lte(left, r)) }
            _ => Ok(left),
        }
    }

    fn parse_additive(&mut self) -> Result<Value> {
        let mut left = self.parse_multiplicative()?;
        loop {
            match self.peek() {
                Token::Plus => { self.bump(); let r = self.parse_multiplicative()?; left = Value::add(left, r)? }
                Token::Minus => { self.bump(); let r = self.parse_multiplicative()?; left = Value::sub(left, r)? }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Value> {
        let mut left = self.parse_unary()?;
        loop {
            match self.peek() {
                Token::Star => { self.bump(); let r = self.parse_unary()?; left = Value::mul(left, r)? }
                Token::Slash => { self.bump(); let r = self.parse_unary()?; left = Value::div(left, r)? }
                Token::Percent => { self.bump(); let r = self.parse_unary()?; left = Value::rem(left, r)? }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Value> {
        match self.peek() {
            Token::Minus => { self.bump(); let v = self.parse_primary()?; Ok(Value::neg(v)?) }
            Token::Plus => { self.bump(); self.parse_primary() }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Value> {
        match self.peek().clone() {
            Token::Number(n, unit) => {
                self.bump();
                let v: f64 = n.parse().unwrap_or(0.0);
                Ok(Value::Number(v, unit))
            }
            Token::String(s, style) => {
                self.bump();
                Ok(Value::String(s, style))
            }
            Token::Variable(name) => {
                self.bump();
                Ok(Value::Variable(name))
            }
            Token::HexColor(hex) => {
                self.bump();
                Ok(Value::parse_hex_color(&hex))
            }
            Token::Ident(s) => {
                self.bump();
                if s == "true" { Ok(Value::Bool(true)) }
                else if s == "false" { Ok(Value::Bool(false)) }
                else if s == "null" { Ok(Value::Null) }
                else { Ok(Value::Ident(s)) }
            }
            Token::LParen => {
                self.bump();
                let v = self.parse_value()?;
                self.eat(&Token::RParen)?;
                Ok(v)
            }
            Token::LBracket => {
                self.bump();
                let mut items = Vec::new();
                if !matches!(self.peek(), Token::RBracket) {
                    items.push(self.parse_value()?);
                    while matches!(self.peek(), Token::Comma | Token::Semicolon) {
                        self.bump();
                        if matches!(self.peek(), Token::RBracket) { break; }
                        items.push(self.parse_value()?);
                    }
                }
                self.eat(&Token::RBracket)?;
                Ok(Value::List(items, Separator::Space, true))
            }
            Token::Interp(s) => {
                self.bump();
                Ok(Value::String(s, crate::lex::token::QuoteStyle::None))
            }
            t => Err(SassError::parse(format!("Unexpected token in expression: {:?}", t))),
        }
    }

    fn is_ident(&self, expected: &str) -> bool {
        matches!(self.peek(), Token::Ident(s) if s == expected)
    }
}
