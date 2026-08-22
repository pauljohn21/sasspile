//! 规则 / 声明解析。
//!
//! 使用 lookahead 策略区分规则（`selector { }`）和声明（`prop: value;`）。

use crate::error::{Result, SassError};
use crate::lex::Token;
use crate::parse::{Node, Parser};

/// 解析规则或声明（区分 selector { } 和 prop: value;）。
pub fn parse_rule_or_decl(parser: &mut Parser) -> Result<Node> {
    // Lookahead: { 先出现 → 规则, ; 或 } 先出现 → 声明
    let is_rule = is_rule_lookahead(parser);

    if is_rule {
        parse_rule(parser)
    } else {
        parse_decl(parser)
    }
}

/// Lookahead 判断是否为规则（遇到 { 前没有 ; 或 }）。
fn is_rule_lookahead(parser: &Parser) -> bool {
    let mut i = parser.pos();
    while i < parser.tokens_len() {
        match &parser.token_at(i) {
            Token::LBrace => return true,
            Token::Semicolon | Token::RBrace | Token::Eof => return false,
            Token::LParen => {
                // 跳过括号内容
                let mut depth = 1;
                i += 1;
                while i < parser.tokens_len() && depth > 0 {
                    match &parser.token_at(i) {
                        Token::LParen => depth += 1,
                        Token::RParen => depth -= 1,
                        _ => {}
                    }
                    i += 1;
                }
                continue;
            }
            _ => { i += 1; }
        }
    }
    true
}

/// 解析规则——selector { body }。
fn parse_rule(parser: &mut Parser) -> Result<Node> {
    let selector = parse_selector(parser)?;
    parser.eat(&Token::LBrace)?;
    let body = parser.parse_body()?;
    parser.eat(&Token::RBrace)?;
    Ok(Node::Rule { selector, body })
}

/// 解析选择器——收集 token 到 { 为止。
fn parse_selector(parser: &mut Parser) -> Result<String> {
    let mut s = String::new();
    let mut bracket_depth = 0i32;
    while !parser.is_eof() && !matches!(parser.peek(), Token::RBrace) {
        match parser.peek().clone() {
            Token::LBrace => break,
            Token::LBracket => {
                bracket_depth += 1;
                s.push('[');
                parser.bump();
            }
            Token::RBracket => {
                bracket_depth -= 1;
                s.push(']');
                parser.bump();
            }
            Token::Comment(_) | Token::SilentComment(_) => {
                parser.bump();
            }
            Token::Comma => {
                s.push_str(", ");
                parser.bump();
            }
            t => {
                let ts = token_to_selector_str(&t);
                if !ts.is_empty() {
                    // 避免连续空格
                    if ts == " " && s.ends_with(' ') {
                        // skip
                    } else {
                        s.push_str(&ts);
                    }
                }
                parser.bump();
            }
        }
    }
    Ok(s.trim().to_string())
}

/// 解析声明——property: value。
fn parse_decl(parser: &mut Parser) -> Result<Node> {
    let property = parse_property(parser)?;
    parser.eat(&Token::Colon)?;
    // 声明值中使用斜杠分隔符语义
    let value = parser.parse_decl_value()?;
    let important = check_important(parser)?;
    consume_semicolon(parser);
    Ok(Node::Decl {
        property: property.trim().to_string(),
        value,
        important,
    })
}

/// 解析属性名——收集 token 到 : 为止。
fn parse_property(parser: &mut Parser) -> Result<String> {
    let mut s = String::new();
    while !parser.is_eof() {
        match parser.peek() {
            Token::Colon | Token::Semicolon | Token::RBrace | Token::Eof => break,
            Token::LBrace => {
                return Err(SassError::parse(format!(
                    "Unexpected {{ after property: {s}"
                )))
            }
            t => {
                s.push_str(t.as_str());
                parser.bump();
            }
        }
    }
    Ok(s)
}

fn check_important(parser: &mut Parser) -> Result<bool> {
    if matches!(parser.peek(), Token::Bang) {
        parser.bump();
        if let Token::Ident(s) = parser.peek() {
            if s == "important" {
                parser.bump();
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn consume_semicolon(parser: &mut Parser) {
    if matches!(parser.peek(), Token::Semicolon) {
        parser.bump();
    }
}

/// 将 token 转换为选择器文本片段。
fn token_to_selector_str(t: &Token) -> String {
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
        Token::Minus => "-".to_string(),
        Token::Star => "*".to_string(),
        Token::Slash => "/".to_string(),
        Token::Percent => "%".to_string(),
        Token::Interp(s) => s.clone(),
        Token::Comment(_) => String::new(),
        Token::LParen => "(".to_string(),
        Token::RParen => ")".to_string(),
        Token::LBracket => "[".to_string(),
        Token::RBracket => "]".to_string(),
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
        Token::Caret => "^".to_string(),
        Token::Pipe => "|".to_string(),
        Token::Eof => String::new(),
        Token::LBrace => String::new(),
        Token::RBrace => String::new(),
    }
}
