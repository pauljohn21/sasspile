//! —— Lexer token 化测试 ——
//!
//! 概要：验证 Lexer 将 SCSS 源码正确 token 化为 Token 序列。
//!
//! ## 覆盖场景
//! - 基础 token：Ident、Number、String、Interp
//! - 选择器 token：Amp (&)、DotDotDot (...)、Colon
//! - 特殊 token：Dollar ($var)、Hash (#ff0000)、AtRule (@media)
//! - 注释：LineComment (//)、BlockComment (/* */)
//! - 操作符：==、!=、<=、>=
//! - 关键字：true、false、null、and、or、not
//! - 复合选择器 token 序列（如 `a:hover`）
//!
//! ## sass-spec 参照
//! - 无直接对应（Lexer 内部测试，为 parser 前置）

use sasspile::lex::Lexer;
use sasspile::lex::token::Token;

fn lex(input: &str) -> Vec<Token> {
    Lexer::new(input)
        .filter(|t| !matches!(t.as_ref(), Ok(Token::Whitespace | Token::Eof)))
        .map(|t| t.expect("unexpected failure in test"))
        .collect()
}

#[test]
fn test_ident() {
    assert_eq!(lex("color"), vec![Token::Ident("color".to_string())]);
}

#[test]
fn test_number_with_unit() {
    assert_eq!(lex("16px"), vec![Token::Number("16px".to_string())]);
}

#[test]
fn test_number_decimal() {
    assert_eq!(lex("3.14"), vec![Token::Number("3.14".to_string())]);
}

#[test]
fn test_string() {
    assert_eq!(
        lex("\"hello\""),
        vec![Token::String("hello".to_string(), '"')]
    );
}

#[test]
fn test_interp() {
    assert_eq!(lex("#{1 + 2}"), vec![Token::Interp("1 + 2".to_string())]);
}

#[test]
fn test_interp_with_string() {
    // #{"not"} 的内容应包含引号
    let tokens = lex("#{\"not\"}");
    assert_eq!(tokens, vec![Token::Interp("\"not\"".to_string())]);
}

#[test]
fn test_interp_not_css() {
    // #{"not"} css() 的 token 序列
    let tokens = lex("#{\"not\"} css()");
    assert_eq!(
        tokens,
        vec![
            Token::Interp("\"not\"".to_string()),
            Token::Ident("css".to_string()),
            Token::LParen,
            Token::RParen,
        ]
    );
}

#[test]
fn test_amp() {
    assert_eq!(
        lex("&:hover"),
        vec![Token::Amp, Token::Colon, Token::Ident("hover".to_string())]
    );
}

#[test]
fn test_dot_dot_dot() {
    assert_eq!(lex("..."), vec![Token::DotDotDot]);
}

#[test]
fn test_at_rule() {
    assert_eq!(lex("@media"), vec![Token::AtRule("media".to_string())]);
}

#[test]
fn test_dollar() {
    assert_eq!(lex("$color"), vec![Token::Dollar("color".to_string())]);
}

#[test]
fn test_hash() {
    assert_eq!(lex("#ff0000"), vec![Token::Hash("ff0000".to_string())]);
}

#[test]
fn test_line_comment() {
    let tokens = lex("// comment");
    assert_eq!(tokens, vec![Token::Comment("comment".to_string(), true)]);
}

#[test]
fn test_block_comment() {
    let tokens = lex("/* hello */");
    assert_eq!(tokens, vec![Token::Comment("hello".to_string(), false)]);
}

#[test]
fn test_operators() {
    let tokens = lex("== != <= >=");
    assert_eq!(
        tokens,
        vec![Token::Eq, Token::NotEq, Token::LessEq, Token::GreaterEq]
    );
}

#[test]
fn test_keywords() {
    let tokens = lex("true false null and or not");
    assert_eq!(
        tokens,
        vec![
            Token::True,
            Token::False,
            Token::Null,
            Token::And,
            Token::Or,
            Token::Not
        ]
    );
}

#[test]
fn test_full_selector() {
    let tokens = lex("a:hover");
    assert_eq!(
        tokens,
        vec![
            Token::Ident("a".to_string()),
            Token::Colon,
            Token::Ident("hover".to_string()),
        ]
    );
}
