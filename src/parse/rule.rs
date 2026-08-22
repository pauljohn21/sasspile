//! 规则 / 声明解析。

use crate::error::{Result, SassError};
use crate::lex::Token;
use crate::parse::{Node, Parser, ast::VarFlags};
use crate::eval::value::Value;

/// 解析规则或声明（区分 selector { } 和 prop: value;）。
pub fn parse_rule_or_decl(parser: &mut Parser) -> Result<Node> {
    // 收集 token 直到遇到 { 或 : 或 ;
    let mut parts: Vec<String> = Vec::new();
    let mut found_colon = false;
    let mut found_lbrace = false;

    loop {
        match parser.peek() {
            Token::LBrace => {
                found_lbrace = true;
                break;
            }
            Token::Colon => {
                found_colon = true;
                break;
            }
            Token::Semicolon | Token::Eof | Token::RBrace => break,
            t => {
                parts.push(token_to_string(t));
                parser.bump();
            }
        }
    }

    let selector_or_prop = parts.join("");

    if found_lbrace {
        parser.eat(&Token::LBrace)?;
        let body = parser.parse_body()?;
        parser.eat(&Token::RBrace)?;
        Ok(Node::Rule { selector: selector_or_prop.trim().to_string(), body })
    } else if found_colon {
        parser.eat(&Token::Colon)?;
        let value = parser.parse_value()?;
        let important = check_important(parser)?;
        consume_semicolon(parser);
        let prop = selector_or_prop.trim().to_string();
        Ok(Node::Decl { property: prop, value, important })
    } else {
        consume_semicolon(parser);
        if selector_or_prop.trim().is_empty() {
            Err(SassError::parse("Empty statement"))
        } else {
            // CSS custom property 或纯 ident 声明
            Ok(Node::Decl {
                property: selector_or_prop.trim().to_string(),
                value: Value::Null,
                important: false,
            })
        }
    }
}

fn check_important(parser: &mut Parser) -> Result<bool> {
    // !important
    if matches!(parser.peek(), Token::Bang) {
        parser.bump();
        if matches!(parser.peek(), Token::Ident(s) if s == "important") {
            parser.bump();
            return Ok(true);
        }
    }
    Ok(false)
}

fn consume_semicolon(parser: &mut Parser) {
    if matches!(parser.peek(), Token::Semicolon) {
        parser.bump();
    }
}

fn token_to_string(t: &Token) -> String {
    match t {
        Token::Ident(s) => s.clone(),
        Token::Number(n, Some(u)) => format!("{n}{u}"),
        Token::Number(n, None) => n.clone(),
        Token::String(s, _) => format!("\"{s}\""),
        Token::Variable(s) => format!("${s}"),
        Token::HexColor(s) => format!("#{s}"),
        Token::Hash => "#".to_string(),
        Token::Ampersand => "&".to_string(),
        Token::Colon => ":".to_string(),
        Token::Comma => ", ".to_string(),
        Token::Dot => ".".to_string(),
        Token::AtRule(s) => format!("@{s}"),
        Token::Plus => " + ".to_string(),
        Token::Minus => " - ".to_string(),
        Token::Star => "*".to_string(),
        Token::Slash => "/".to_string(),
        Token::Percent => "%".to_string(),
        Token::Interp(s) => format!("#{{{s}}}"),
        Token::Comment(s) => format!("/*{s}*/"),
        Token::LParen => "(".to_string(),
        Token::RParen => ")".to_string(),
        Token::LBracket => "[".to_string(),
        Token::RBracket => "]".to_string(),
        Token::LBrace => " {".to_string(),
        Token::RBrace => "}".to_string(),
        Token::Semicolon => ";".to_string(),
        Token::Eq => "=".to_string(),
        Token::Gt => " > ".to_string(),
        Token::Lt => " < ".to_string(),
        Token::Gte => " >= ".to_string(),
        Token::Lte => " <= ".to_string(),
        Token::NotEq => " != ".to_string(),
        Token::Arrow => " => ".to_string(),
        Token::Bang => "!".to_string(),
        Token::DotDotDot => "...".to_string(),
        Token::True => "true".to_string(),
        Token::False => "false".to_string(),
        Token::Null => "null".to_string(),
        Token::And => " and ".to_string(),
        Token::Or => " or ".to_string(),
        Token::Not => " not ".to_string(),
        Token::SilentComment(_) => String::new(),
        Token::Eof => String::new(),
    }
}
