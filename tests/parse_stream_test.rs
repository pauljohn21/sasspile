//! 流式 Parser 测试 —— 验证 scan 状态机解析正确性
//!
//! 核心验证：`feed_token` 纯函数正确地将 Token 流变换为 Node 流。

use sasspile::lex::token::Token;
use sasspile::parse::stream::{feed_token, parse_all, ParseState};

/// 辅助函数：lex + parse 为 Node Vec
fn lex_and_feed(input: &str) -> Vec<sasspile::parse::ast::Node> {
    let tokens: Vec<Token> = sasspile::lex::Lexer::new(input)
        .filter(|t| !matches!(t, Ok(Token::Whitespace)))
        .map(|t| t.unwrap())
        .collect();

    let mut state = ParseState::default();
    let mut all_nodes = Vec::new();

    for token in tokens {
        let (new_state, nodes) = feed_token(state, token);
        state = new_state;
        all_nodes.extend(nodes);
    }

    all_nodes
}

// ─── 基础解析测试 ──────────────────────────────────────────────────────────

#[test]
fn test_parse_simple_rule() {
    let nodes = lex_and_feed("a { color: red }");
    assert!(!nodes.is_empty(), "should parse simple rule");
}

#[test]
fn test_parse_declaration_semicolon() {
    let nodes = lex_and_feed("color: red;");
    assert!(!nodes.is_empty(), "should parse declaration with semicolon");
}

#[test]
fn test_parse_variable_assignment() {
    let nodes = lex_and_feed("$x: 42");
    assert!(!nodes.is_empty(), "should parse variable assignment");
}

// ─── parse_all（fallback）测试 ────────────────────────────────────────────────

#[test]
fn test_parse_all_simple_rule() {
    let tokens: Vec<Token> = sasspile::lex::Lexer::new("a { color: red }")
        .filter(|t| !matches!(t, Ok(Token::Whitespace)))
        .map(|t| t.unwrap())
        .collect();

    let nodes = parse_all(&tokens).expect("parse_all should succeed");
    assert!(!nodes.is_empty(), "parse_all should produce nodes");
}

#[test]
fn test_parse_all_nested_rules() {
    let tokens: Vec<Token> = sasspile::lex::Lexer::new(".a { .b { color: red } }")
        .filter(|t| !matches!(t, Ok(Token::Whitespace)))
        .map(|t| t.unwrap())
        .collect();

    let nodes = parse_all(&tokens).expect("parse_all should succeed");
    assert!(!nodes.is_empty(), "parse_all should handle nested rules");
}
