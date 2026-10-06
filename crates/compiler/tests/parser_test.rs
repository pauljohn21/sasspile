//! Unit tests for the parser module.

use rxrust::prelude::*;

use lightforger::parser::{ParserState, SassAstNode, InputSyntax};
use lightforger::lexer::{Token, TokenKind};
use lightforger::parser;

/// Helper: collect an Infallible SassAstStream into Vec<SassAstNode>
fn collect_nodes(stream: LocalBoxedObservableClone<'static, SassAstNode, std::convert::Infallible>) -> Vec<SassAstNode> {
    let result = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let r = result.clone();
    stream.subscribe(move |node| r.borrow_mut().push(node));
    match std::rc::Rc::try_unwrap(result) {
        Ok(cell) => cell.into_inner(),
        Err(_) => Vec::new(),
    }
}

/// Helper: create a Token stream from source using the real lexer
fn tokenize(source: &str) -> LocalBoxedObservableClone<'static, Token, std::convert::Infallible> {
    use lightforger::lexer;
    let normalized = lexer::normalize_newlines(source);
    let chars: Vec<char> = normalized.chars().collect();
    let char_stream = Local::from_iter(chars).box_it_clone();
    lexer::lex(char_stream)
}

#[test]
fn test_parser_state_default() {
    let state = ParserState::new();
    assert!(state.is_eof());
    assert_eq!(state.cursor(), 0);
}

#[test]
fn test_peek_and_advance() {
    let mut state = ParserState::new();
    state.push_token(Token::new(TokenKind::Char('a'), 0, "a"));
    state.push_token(Token::new(TokenKind::Char('b'), 1, "b"));
    state.push_token(Token::new(TokenKind::Char('c'), 2, "c"));

    assert_eq!(state.peek().unwrap().char_kind(), Some('a'));
    assert_eq!(state.peek().unwrap().char_kind(), Some('a'));

    assert_eq!(state.advance().unwrap().char_kind(), Some('a'));
    assert_eq!(state.peek().unwrap().char_kind(), Some('b'));
}

#[test]
fn test_peek_n_lookahead() {
    let mut state = ParserState::new();
    state.push_token(Token::new(TokenKind::Char('a'), 0, "a"));
    state.push_token(Token::new(TokenKind::Char('b'), 1, "b"));
    state.push_token(Token::new(TokenKind::Char('c'), 2, "c"));

    assert_eq!(state.peek_n(0).unwrap().char_kind(), Some('a'));
    assert_eq!(state.peek_n(1).unwrap().char_kind(), Some('b'));
    assert_eq!(state.peek_n(2).unwrap().char_kind(), Some('c'));
    assert!(state.peek_n(3).is_none());
}

#[test]
fn test_cursor_rollback() {
    let mut state = ParserState::new();
    state.push_token(Token::new(TokenKind::Char('a'), 0, "a"));
    state.push_token(Token::new(TokenKind::Char('b'), 1, "b"));

    state.advance();
    assert_eq!(state.cursor(), 1);

    state.set_cursor(0);
    assert_eq!(state.cursor(), 0);
    assert_eq!(state.peek().unwrap().char_kind(), Some('a'));
}

#[test]
fn test_is_eof() {
    let mut state = ParserState::new();
    assert!(state.is_eof());

    state.push_token(Token::new(TokenKind::Char('a'), 0, "a"));
    assert!(!state.is_eof());

    state.advance();
    assert!(state.is_eof());
}

#[test]
fn test_input_syntax_enum() {
    let _scss = InputSyntax::Scss;
    let _sass = InputSyntax::Sass;
    let _css = InputSyntax::Css;
}

#[test]
fn test_parse_style_decl() {
    // Simple: "color: red;"
    let tokens = tokenize("color: red;");
    let nodes = collect_nodes(parser::parse(tokens));
    assert!(!nodes.is_empty(), "parser should produce nodes");
    // Should be a StyleDecl
    match &nodes[0] {
        SassAstNode::StyleDecl { prop, value } => {
            assert_eq!(prop, "color");
            assert_eq!(value, "red");
        }
        other => panic!("expected StyleDecl, got {:?}", other),
    }
}

#[test]
fn test_parse_empty() {
    let tokens = tokenize("");
    let nodes = collect_nodes(parser::parse(tokens));
    assert!(nodes.is_empty());
}

#[test]
fn test_parse_variable_decl() {
    // "$color: red;"
    let tokens = tokenize("$color: red;");
    let nodes = collect_nodes(parser::parse(tokens));
    assert!(!nodes.is_empty());
    match &nodes[0] {
        SassAstNode::VariableDecl { name, value, has_default } => {
            assert!(name.contains("$color"));
            assert_eq!(*has_default, false);
            match value.as_ref() {
                SassAstNode::Raw(v) => assert!(v.contains("red")),
                other => panic!("expected Raw value, got {:?}", other),
            }
        }
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn test_parse_variable_decl_default() {
    // "$color: red !default;"
    let tokens = tokenize("$color: red !default;");
    let nodes = collect_nodes(parser::parse(tokens));
    assert!(!nodes.is_empty());
    match &nodes[0] {
        SassAstNode::VariableDecl { name, has_default, .. } => {
            assert!(name.contains("$color"));
            assert_eq!(*has_default, true);
        }
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn test_parse_rule_simple() {
    // "a { color: red; }"
    let tokens = tokenize("a { color: red; }");
    let nodes = collect_nodes(parser::parse(tokens));
    assert!(!nodes.is_empty());
    match &nodes[0] {
        SassAstNode::Rule { selector, .. } => {
            assert!(selector.contains("a"), "selector was: {}", selector);
        }
        other => panic!("expected Rule, got {:?}", other),
    }
}

#[test]
fn test_parse_nested_rule() {
    // "a { &:hover { color: red; } }"
    let tokens = tokenize("a { &:hover { color: red; } }");
    let nodes = collect_nodes(parser::parse(tokens));
    assert!(!nodes.is_empty());
    // The outer rule should parse with selector "a"
    let first = &nodes[0];
    match first {
        SassAstNode::Rule { selector, inner } => {
            assert!(selector.contains("a"), "outer selector was: {}", selector);
            // Inner should contain the &:hover rule
            assert!(!inner.is_empty(), "inner rules should not be empty");
        }
        other => panic!("expected Rule, got {:?}", other),
    }
}

#[test]
fn test_sast_node_constructors() {
    // VariableDecl
    let _var = SassAstNode::VariableDecl {
        name: "$color".into(),
        value: Box::new(SassAstNode::Raw("red".into())),
        has_default: false,
    };

    // Rule
    let _rule = SassAstNode::Rule {
        selector: "a".into(),
        inner: vec![],
    };

    // StyleDecl
    let _style = SassAstNode::StyleDecl {
        prop: "color".into(),
        value: "red".into(),
    };

    // Interpolated
    let _interp = SassAstNode::Interpolated("$x".into());

    // ParentSelector
    let _parent = SassAstNode::ParentSelector;
}
