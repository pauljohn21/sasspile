//! Declaration parsing helpers — 使用原语迭代器（不构造 Observable）

use crate::ast::Token;

pub fn parse_declaration_prop(tokens: &[Token]) -> Option<String> {
    tokens
        .iter()
        .filter_map(|t| match t {
            Token::Ident(s) => Some(s.clone()),
            _ => None,
        })
        .reduce(|a, b| format!("{a}-{b}"))
}

pub fn parse_declaration_value(tokens: &[Token]) -> String {
    tokens
        .iter()
        .filter_map(|t| match t {
            Token::Ident(s) => Some(s.clone()),
            Token::Number(n) => Some(n.clone()),
            Token::String(s) => Some(s.clone()),
            Token::Interpolation(s) => Some(format!("#{{{s}}}")),
            Token::Op(o) => Some(o.clone()),
            Token::Comma => Some(",".to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}
