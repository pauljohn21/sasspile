//! Unit tests for the parser module.

use rxrust::prelude::*;

use lightforger::parser::{ParserState, SassAstNode, InputSyntax};
use lightforger::lexer::{Token, TokenKind};
use lightforger::parser;

/// Helper: collect a SassAstStream into Vec<SassAstNode>
/// Returns Ok(nodes) on success, Err(error) if the stream emits an error.
fn collect_nodes(stream: LocalBoxedObservableClone<'static, SassAstNode, lightforger::Error>) -> Result<Vec<SassAstNode>, lightforger::Error> {
    let result = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let r = result.clone();
    let error = std::rc::Rc::new(std::cell::RefCell::new(None));
    let e = error.clone();
    stream
        .on_error(move |err| *e.borrow_mut() = Some(err))
        .subscribe(move |node| r.borrow_mut().push(node));
    if let Some(err) = error.borrow().clone() {
        Err(err)
    } else {
        match std::rc::Rc::try_unwrap(result) {
            Ok(cell) => Ok(cell.into_inner()),
            Err(_) => Ok(Vec::new()),
        }
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

/// Helper: collect a TokenStream into Vec<Token> (local to parser tests)
fn collect_tokens_local(stream: LocalBoxedObservableClone<'static, Token, std::convert::Infallible>) -> Vec<Token> {
    let result = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let r = result.clone();
    stream.subscribe(move |tok| r.borrow_mut().push(tok));
    match std::rc::Rc::try_unwrap(result) {
        Ok(cell) => cell.into_inner(),
        Err(_) => Vec::new(),
    }
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
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
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
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
    assert!(nodes.is_empty());
}

#[test]
fn test_parse_variable_decl() {
    // "$color: red;"
    let tokens = tokenize("$color: red;");
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
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
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
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
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
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
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
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

// ── Interpolation tests (Task 3.6) ──

#[test]
fn test_parse_interpolation_simple() {
    // "#{$class}" → Interpolated("$class")
    let tokens = tokenize("#{$class}");
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
    assert!(!nodes.is_empty(), "should produce nodes");
    match &nodes[0] {
        SassAstNode::Interpolated(inner) => {
            assert!(inner.contains("$class"), "inner was: {}", inner);
        }
        other => panic!("expected Interpolated, got {:?}", other),
    }
}

#[test]
fn test_parse_interpolation_in_selector() {
    // ".#{$class}" → Interpolated with prefix
    let tokens = tokenize(".#{$class}");
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
    assert!(!nodes.is_empty(), "should produce nodes");
    // Should be an Interpolated node with the interpolation marker
    match &nodes[0] {
        SassAstNode::Interpolated(inner) => {
            assert!(inner.contains("#{$class}"), "inner was: {}", inner);
        }
        other => panic!("expected Interpolated, got {:?}", other),
    }
}

#[test]
fn test_lex_interpolation_token() {
    // Verify that "#{$x}" produces an Interpolation token
    use lightforger::lexer;
    let normalized = lexer::normalize_newlines("#{$x}");
    let chars: Vec<char> = normalized.chars().collect();
    let char_stream = Local::from_iter(chars).box_it_clone();
    let tokens = collect_tokens_local(lexer::lex(char_stream));
    assert!(
        tokens.iter().any(|t| t.kind == TokenKind::Interpolation),
        "expected Interpolation token in {:?}",
        tokens
    );
}

#[test]
fn test_lex_interpolation_nested_property() {
    // "#{$prefix}btn-color" → Interpolation token + Ident
    use lightforger::lexer;
    let normalized = lexer::normalize_newlines("#{$prefix}btn-color");
    let chars: Vec<char> = normalized.chars().collect();
    let char_stream = Local::from_iter(chars).box_it_clone();
    let tokens = collect_tokens_local(lexer::lex(char_stream));
    assert!(
        tokens.iter().any(|t| t.kind == TokenKind::Interpolation),
        "expected Interpolation token in {:?}",
        tokens
    );
}

// ── Group literal tests (Task 3.7) ──

#[test]
fn test_parse_map_literal_simple() {
    // "(blue: #0d6efd)" → MapLiteral
    let tokens = tokenize("(blue: #0d6efd)");
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
    assert!(!nodes.is_empty(), "should produce nodes");
    match &nodes[0] {
        SassAstNode::MapLiteral(entries) => {
            assert!(!entries.is_empty(), "map should have entries");
            // First key should contain "blue"
            match &entries[0].0 {
                SassAstNode::Raw(k) => assert!(k.contains("blue"), "key was: {}", k),
                other => panic!("expected Raw key, got {:?}", other),
            }
        }
        other => panic!("expected MapLiteral, got {:?}", other),
    }
}

#[test]
fn test_parse_map_literal_multi_entry() {
    // "(blue: #0d6efd, red: #dc3545)" → MapLiteral with 2 entries
    let tokens = tokenize("(blue: #0d6efd, red: #dc3545)");
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
    assert!(!nodes.is_empty(), "should produce nodes");
    match &nodes[0] {
        SassAstNode::MapLiteral(entries) => {
            assert_eq!(entries.len(), 2, "expected 2 entries");
        }
        other => panic!("expected MapLiteral, got {:?}", other),
    }
}

#[test]
fn test_parse_list_literal() {
    // "(red, green, blue)" → ListLiteral
    let tokens = tokenize("(red, green, blue)");
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
    assert!(!nodes.is_empty(), "should produce nodes");
    match &nodes[0] {
        SassAstNode::ListLiteral(items) => {
            assert_eq!(items.len(), 3, "expected 3 items, got {}", items.len());
        }
        other => panic!("expected ListLiteral, got {:?}", other),
    }
}

#[test]
fn test_parse_list_literal_single() {
    // "(red)" → ListLiteral with 1 item
    let tokens = tokenize("(red)");
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
    assert!(!nodes.is_empty(), "should produce nodes");
    match &nodes[0] {
        SassAstNode::ListLiteral(items) => {
            assert_eq!(items.len(), 1, "expected 1 item");
        }
        other => panic!("expected ListLiteral, got {:?}", other),
    }
}

#[test]
fn test_parse_empty_list_literal() {
    // "()" → ListLiteral empty
    let tokens = tokenize("()");
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
    assert!(!nodes.is_empty(), "should produce nodes");
    match &nodes[0] {
        SassAstNode::ListLiteral(items) => {
            assert!(items.is_empty(), "expected empty list");
        }
        other => panic!("expected ListLiteral, got {:?}", other),
    }
}

#[test]
fn test_parse_nested_rule_with_parent_selector() {
    // "&:hover" → Interpolated with parent selector
    let tokens = tokenize("&:hover { color: red; }");
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
    assert!(!nodes.is_empty(), "should produce nodes");
    // The first node should be a Rule with selector containing "&:hover"
    match &nodes[0] {
        SassAstNode::Rule { selector, .. } => {
            assert!(selector.contains("&"), "selector was: {}", selector);
        }
        other => panic!("expected Rule, got {:?}", other),
    }
}

#[test]
fn test_map_literal_preserves_values() {
    // "(primary: #0d6efd)" — check key and value
    let tokens = tokenize("(primary: #0d6efd)");
    let nodes = collect_nodes(parser::parse(tokens)).expect("parse should succeed");
    match &nodes[0] {
        SassAstNode::MapLiteral(entries) => {
            assert_eq!(entries.len(), 1);
            // Key should be "primary"
            match &entries[0].0 {
                SassAstNode::Raw(k) => {
                    assert!(k.contains("primary"), "key was: {}", k);
                }
                other => panic!("expected Raw key, got {:?}", other),
            }
            // Value should contain "#0d6efd"
            match &entries[0].1 {
                SassAstNode::Raw(v) => {
                    assert!(v.contains("#0d6efd"), "value was: {}", v);
                }
                other => panic!("expected Raw value, got {:?}", other),
            }
        }
        other => panic!("expected MapLiteral, got {:?}", other),
    }
}

// ── Error propagation tests (Task 3.8) ──

#[test]
fn test_parse_unclosed_paren_error() {
    // "(blue: #0d6efd" — 缺少闭合括号
    let tokens = tokenize("(blue: #0d6efd");
    let result = collect_nodes(parser::parse(tokens));
    assert!(result.is_err(), "unclosed paren should produce an error");
    let err = result.unwrap_err();
    let err_msg = format!("{err}");
    assert!(
        err_msg.contains("unclosed") || err_msg.contains("'('"),
        "error should mention unclosed paren, was: {err_msg}"
    );
}

#[test]
fn test_parse_unclosed_brace_error() {
    // "a { color: red;" — 缺少闭合花括号
    let tokens = tokenize("a { color: red;");
    let result = collect_nodes(parser::parse(tokens));
    assert!(result.is_err(), "unclosed brace should produce an error");
    let err = result.unwrap_err();
    let err_msg = format!("{err}");
    assert!(
        err_msg.contains("unclosed") || err_msg.contains("'{'"),
        "error should mention unclosed brace, was: {err_msg}"
    );
}

#[test]
fn test_parse_deeply_unclosed_paren() {
    // "((blue: #0d6efd)" — 嵌套括号缺少一个
    let tokens = tokenize("((blue: #0d6efd)");
    let result = collect_nodes(parser::parse(tokens));
    assert!(result.is_err(), "deeply unclosed paren should produce an error");
}

#[test]
fn test_parse_valid_no_error() {
    // "(blue: #0d6efd)" — 合法的闭合括号不应产生错误
    let tokens = tokenize("(blue: #0d6efd)");
    let result = collect_nodes(parser::parse(tokens));
    assert!(result.is_ok(), "valid paren should not produce an error");
}

#[test]
fn test_parse_valid_brace_no_error() {
    // "a { color: red; }" — 合法的花括号不应产生错误
    let tokens = tokenize("a { color: red; }");
    let result = collect_nodes(parser::parse(tokens));
    assert!(result.is_ok(), "valid brace should not produce an error");
}
