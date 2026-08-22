//! 变量声明解析（$var: value;）。

use crate::error::Result;
use crate::lex::Token;
use crate::parse::{Node, Parser, ast::VarFlags};
use crate::eval::value::Value;

/// 解析 $var: value 声明。
pub fn parse_variable_decl(parser: &mut Parser) -> Result<Node> {
    // $name
    let name = match parser.bump() {
        Token::Variable(n) => n,
        _ => return Err(crate::error::SassError::parse("Expected variable name")),
    };

    // : value
    parser.eat(&Token::Colon)?;

    // 检查标志位 !default / !global
    let value = parser.parse_value()?;
    let flags = parse_var_flags(parser)?;

    // 消费分号
    if matches!(parser.peek(), Token::Semicolon) {
        parser.bump();
    }

    Ok(Node::Variable { name, value, flags })
}

fn parse_var_flags(parser: &mut Parser) -> Result<VarFlags> {
    let mut flags = VarFlags::default();
    loop {
        match parser.peek() {
            Token::Bang => {
                parser.bump();
                if let Token::Ident(t) = parser.peek() {
                    if t == "default" { flags.default = true; }
                    if t == "global" { flags.global = true; }
                    parser.bump();
                }
            }
            _ => break,
        }
    }
    Ok(flags)
}
