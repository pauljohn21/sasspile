//! Body / params / args 解析辅助。

use crate::error::{Result, SassError};
use crate::lex::Token;
use crate::parse::ast::{Param, Arg};
use crate::eval::value::Value;
use super::Parser;

impl Parser {
    /// 解析函数参数列表 ($a, $b: default, $rest...)。
    pub fn parse_params(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();
        if !matches!(self.peek(), Token::LParen) {
            return Ok(params);
        }
        self.bump(); // (

        if matches!(self.peek(), Token::RParen) {
            self.bump();
            return Ok(params);
        }

        loop {
            // $name
            let name = match self.bump() {
                Token::Variable(n) => n,
                Token::Ident(ref s) if s == "..." => {
                    // rest without name? skip
                    continue;
                }
                t => return Err(SassError::parse(format!("Expected param name, got {:?}", t))),
            };

            // ... (rest)
            let rest = matches!(self.peek(), Token::DotDotDot);
            if rest { self.bump(); }

            // : default
            let default = if matches!(self.peek(), Token::Colon) {
                self.bump();
                Some(self.parse_value()?)
            } else {
                None
            };

            params.push(Param { name, default, rest });

            match self.peek() {
                Token::Comma => { self.bump(); }
                Token::RParen => { self.bump(); break; }
                t => return Err(SassError::parse(format!("Expected , or ), got {:?}", t))),
            }
        }

        Ok(params)
    }

    /// 解析调用参数列表。
    pub fn parse_args(&mut self) -> Result<Vec<Arg>> {
        let mut args = Vec::new();
        if !matches!(self.peek(), Token::LParen) {
            return Ok(args);
        }
        self.bump(); // (

        if matches!(self.peek(), Token::RParen) {
            self.bump();
            return Ok(args);
        }

        loop {
            // named arg? $name: value
            let (name, value) = if matches!(self.peek(), Token::Variable(_)) {
                let saved = self.pos;
                let n = match self.bump() {
                    Token::Variable(n) => n,
                    _ => unreachable!(),
                };
                if matches!(self.peek(), Token::Colon) {
                    self.bump();
                    (Some(n), self.parse_value()?)
                } else {
                    // not named, restore
                    self.pos = saved;
                    (None, self.parse_value()?)
                }
            } else {
                (None, self.parse_value()?)
            };

            let spread = matches!(self.peek(), Token::DotDotDot);
            if spread { self.bump(); }

            args.push(Arg { name, value, spread });

            match self.peek() {
                Token::Comma => { self.bump(); }
                Token::RParen => { self.bump(); break; }
                t => return Err(SassError::parse(format!("Expected , or ), got {:?}", t))),
            }
        }

        Ok(args)
    }
}
