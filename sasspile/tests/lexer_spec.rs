//! Tests for the lexer module.

use sasspile::lexer::{tokenize, TokenKind};

#[test]
fn tokenize_ident() {
    let (tokens, _) = tokenize("color");
    assert_eq!(tokens[0].kind, TokenKind::Ident("color".to_string()));
}

#[test]
fn tokenize_number_with_unit() {
    let (tokens, _) = tokenize("16px");
    assert_eq!(tokens[0].kind, TokenKind::Number(16.0, Some("px".to_string())));
}

#[test]
fn tokenize_number_without_unit() {
    let (tokens, _) = tokenize("42");
    assert_eq!(tokens[0].kind, TokenKind::Number(42.0, None));
}

#[test]
fn tokenize_float() {
    let (tokens, _) = tokenize("1.5rem");
    assert_eq!(tokens[0].kind, TokenKind::Number(1.5, Some("rem".to_string())));
}

#[test]
fn tokenize_dot_number() {
    let (tokens, _) = tokenize(".5px");
    assert_eq!(tokens[0].kind, TokenKind::Number(0.5, Some("px".to_string())));
}

#[test]
fn tokenize_variable() {
    let (tokens, _) = tokenize("$var");
    assert_eq!(tokens[0].kind, TokenKind::Variable("var".to_string()));
}

#[test]
fn tokenize_string_double_quoted() {
    let (tokens, _) = tokenize("\"hello world\"");
    assert_eq!(tokens[0].kind, TokenKind::String("hello world".to_string()));
}

#[test]
fn tokenize_string_single_quoted() {
    let (tokens, _) = tokenize("'hello'");
    assert_eq!(tokens[0].kind, TokenKind::String("hello".to_string()));
}

#[test]
fn tokenize_hex_color() {
    let (tokens, _) = tokenize("#ff0000");
    assert_eq!(tokens[0].kind, TokenKind::Color(0xff0000));
}

#[test]
fn tokenize_at_keyword() {
    let (tokens, _) = tokenize("@use");
    assert_eq!(tokens[0].kind, TokenKind::AtKeyword("use".to_string()));
}

#[test]
fn tokenize_interpolation() {
    let (tokens, _) = tokenize("#{");
    assert_eq!(tokens[0].kind, TokenKind::Interpolation);
}

#[test]
fn tokenize_colon() {
    let (tokens, _) = tokenize(":");
    assert_eq!(tokens[0].kind, TokenKind::Colon);
}

#[test]
fn tokenize_equality() {
    let (tokens, _) = tokenize("==");
    assert_eq!(tokens[0].kind, TokenKind::Eq);
}

#[test]
fn tokenize_not_equal() {
    let (tokens, _) = tokenize("!=");
    assert_eq!(tokens[0].kind, TokenKind::NotEq);
}

#[test]
fn tokenize_greater_equal() {
    let (tokens, _) = tokenize(">=");
    assert_eq!(tokens[0].kind, TokenKind::GreaterEq);
}

#[test]
fn tokenize_less_equal() {
    let (tokens, _) = tokenize("<=");
    assert_eq!(tokens[0].kind, TokenKind::LessEq);
}

#[test]
fn tokenize_logical_and() {
    let (tokens, _) = tokenize("and");
    assert_eq!(tokens[0].kind, TokenKind::And);
}

#[test]
fn tokenize_logical_or() {
    let (tokens, _) = tokenize("or");
    assert_eq!(tokens[0].kind, TokenKind::Or);
}

#[test]
fn tokenize_not_operator() {
    let (tokens, _) = tokenize("not");
    assert_eq!(tokens[0].kind, TokenKind::Not);
}

#[test]
fn tokenize_parent_selector() {
    let (tokens, _) = tokenize("&");
    assert_eq!(tokens[0].kind, TokenKind::Ampersand);
}

#[test]
fn tokenize_scss_rule() {
    let (tokens, _) = tokenize("a { color: red; }");
    let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
    assert!(kinds.contains(&&TokenKind::Ident("a".to_string())));
    assert!(kinds.contains(&&TokenKind::LBrace));
    assert!(kinds.contains(&&TokenKind::Ident("color".to_string())));
    assert!(kinds.contains(&&TokenKind::Colon));
    assert!(kinds.contains(&&TokenKind::Ident("red".to_string())));
    assert!(kinds.contains(&&TokenKind::Semicolon));
    assert!(kinds.contains(&&TokenKind::RBrace));
}

#[test]
fn tokenize_dot_dot_dot() {
    let (tokens, _) = tokenize("...");
    assert_eq!(tokens[0].kind, TokenKind::DotDotDot);
}

#[test]
fn tokenize_percent() {
    let (tokens, _) = tokenize("50%");
    assert_eq!(tokens[0].kind, TokenKind::Number(50.0, Some("%".to_string())));
}

#[test]
fn tokenize_eof_is_last() {
    let (tokens, diags) = tokenize("$var");
    assert!(!diags.has_errors());
    assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
}

// ─── Sass Indented Syntax Tests ───────────────────────────────────

#[test]
fn sass_indent_basic() {
    use sasspile::lexer::sass_syntax::IndentTracker;

    let mut tracker = IndentTracker::new();
    let tokens = tracker.process_line("  color: red");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Indent);
}

#[test]
fn sass_dedent_basic() {
    use sasspile::lexer::sass_syntax::IndentTracker;

    let mut tracker = IndentTracker::new();
    tracker.process_line("  color: red");
    let tokens = tracker.process_line(".foo");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Dedent);
}
