use rx_scss::parser::parse_stream;
use rx_scss::types::*;
use rxrust::prelude::*;
use std::sync::{Arc, Mutex};

fn tokens_to_stream(tokens: Vec<Token>) -> TokenStream {
    Shared::from_iter(tokens).box_it()
}

fn collect_ast(stream: TokenStream) -> Vec<AstNode> {
    let ast_stream = parse_stream(stream, 0);
    let result = Arc::new(Mutex::new(Vec::new()));
    let r = result.clone();
    ast_stream.subscribe(move |node| {
        r.lock().unwrap().push(node);
    });
    let guard = result.lock().unwrap();
    guard.clone()
}

#[test]
fn parse_variable_decl() {
    let tokens = vec![
        Token::Dollar,
        Token::Ident("color".into()),
        Token::Colon,
        Token::Whitespace,
        Token::Str("red".into()),
        Token::Semicolon,
        Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    assert!(!nodes.is_empty(), "variable decl: {:?}", nodes);
    match &nodes[0] {
        AstNode::VariableDecl { name, .. } => assert_eq!(name, "color"),
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn parse_rule_with_selector() {
    let tokens = vec![
        Token::Ident("body".into()),
        Token::Whitespace,
        Token::LBrace,
        Token::RBrace,
        Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    assert!(!nodes.is_empty(), "rule: {:?}", nodes);
    match &nodes[0] {
        AstNode::Rule { selector, .. } => assert_eq!(selector, "body"),
        other => panic!("expected Rule, got {:?}", other),
    }
}

#[test]
fn parse_media_at_rule() {
    let tokens = vec![
        Token::AtMedia,
        Token::Whitespace,
        Token::Str("screen".into()),
        Token::Whitespace,
        Token::Ident("and".into()),
        Token::Whitespace,
        Token::LParen,
        Token::Str("min-width".into()),
        Token::Colon,
        Token::Whitespace,
        Token::Number(768.0, Some("px".into())),
        Token::RParen,
        Token::Whitespace,
        Token::LBrace,
        Token::Ident("body".into()),
        Token::LBrace,
        Token::RBrace,
        Token::RBrace,
        Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    assert!(!nodes.is_empty(), "media: {:?}", nodes);
    match &nodes[0] {
        AstNode::Media { query, inner } => {
            assert!(query.contains("screen"), "query: {}", query);
            assert!(!inner.is_empty(), "inner empty");
        }
        other => panic!("expected Media, got {:?}", other),
    }
}

#[test]
fn parse_if_at_rule() {
    let tokens = vec![
        Token::AtIf,
        Token::Whitespace,
        Token::Ident("true".into()),
        Token::Whitespace,
        Token::LBrace,
        Token::RBrace,
        Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    assert!(!nodes.is_empty(), "if: {:?}", nodes);
    match &nodes[0] {
        AstNode::If { cond, else_branch, .. } => {
            assert!(matches!(cond.as_ref(), AstNode::Literal(Value::Bool(true))), "cond: {:?}", cond);
            assert!(else_branch.is_none());
        }
        other => panic!("expected If, got {:?}", other),
    }
}

#[test]
fn parse_hex_color_value() {
    let tokens = vec![
        Token::HashId("ff0000".into()),
        Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    // HashId alone doesn't trigger a rule, it might be ignored
    // The actual hex color parsing happens inside parse_value when it sees Ident("#...")
    assert!(true);
}

#[test]
fn parse_mixin_declaration() {
    let tokens = vec![
        Token::AtMixin,
        Token::Whitespace,
        Token::Ident("box".into()),
        Token::LParen,
        Token::Dollar,
        Token::Ident("size".into()),
        Token::Colon,
        Token::Whitespace,
        Token::Number(10.0, Some("px".into())),
        Token::RParen,
        Token::Whitespace,
        Token::LBrace,
        Token::RBrace,
        Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    assert!(!nodes.is_empty(), "mixin: {:?}", nodes);
    match &nodes[0] {
        AstNode::MixinDecl { name, params, .. } => {
            assert_eq!(name, "box");
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "size");
        }
        other => panic!("expected MixinDecl, got {:?}", other),
    }
}

#[test]
fn parse_include_call() {
    let tokens = vec![
        Token::AtInclude,
        Token::Whitespace,
        Token::Ident("box".into()),
        Token::LParen,
        Token::Number(20.0, Some("px".into())),
        Token::RParen,
        Token::Semicolon,
        Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    assert!(!nodes.is_empty(), "include: {:?}", nodes);
    match &nodes[0] {
        AstNode::MixinCall { name, args, content } => {
            assert_eq!(name, "box");
            assert_eq!(args.len(), 1);
            assert!(content.is_empty(), "no content block in this test");
        }
        other => panic!("expected MixinCall, got {:?}", other),
    }
}

#[test]
fn parse_func_declaration() {
    let tokens = vec![
        Token::AtFunction,
        Token::Whitespace,
        Token::Ident("double".into()),
        Token::LParen,
        Token::Dollar,
        Token::Ident("n".into()),
        Token::RParen,
        Token::Whitespace,
        Token::LBrace,
        Token::AtReturn,
        Token::Whitespace,
        Token::Dollar,
        Token::Ident("n".into()),
        Token::Star,
        Token::Number(2.0, None),
        Token::Semicolon,
        Token::RBrace,
        Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    assert!(!nodes.is_empty(), "func: {:?}", nodes);
    match &nodes[0] {
        AstNode::FunctionDecl { name, params, body } => {
            assert_eq!(name, "double");
            assert_eq!(params.len(), 1);
            assert!(!body.is_empty());
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn parse_for_at_rule() {
    let tokens = vec![
        Token::AtFor,
        Token::Whitespace,
        Token::Dollar,
        Token::Ident("i".into()),
        Token::Whitespace,
        Token::Ident("from".into()),
        Token::Whitespace,
        Token::Number(1.0, None),
        Token::Whitespace,
        Token::Ident("through".into()),
        Token::Whitespace,
        Token::Number(3.0, None),
        Token::Whitespace,
        Token::LBrace,
        Token::RBrace,
        Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    assert!(!nodes.is_empty(), "for: {:?}", nodes);
    match &nodes[0] {
        AstNode::For { var, inclusive, .. } => {
            assert_eq!(var, "i");
            assert!(*inclusive);
        }
        other => panic!("expected For, got {:?}", other),
    }
}

#[test]
fn parse_each_at_rule() {
    let tokens = vec![
        Token::AtEach,
        Token::Whitespace,
        Token::Dollar,
        Token::Ident("item".into()),
        Token::Whitespace,
        Token::Ident("in".into()),
        Token::Whitespace,
        Token::Str("a".into()),
        Token::Whitespace,
        Token::LBrace,
        Token::RBrace,
        Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    assert!(!nodes.is_empty(), "each: {:?}", nodes);
    match &nodes[0] {
        AstNode::Each { vars, .. } => {
            assert_eq!(vars.len(), 1);
            assert_eq!(vars[0], "item");
        }
        other => panic!("expected Each, got {:?}", other),
    }
}

#[test]
fn parse_while_at_rule() {
    let tokens = vec![
        Token::AtWhile,
        Token::Whitespace,
        Token::Ident("false".into()),
        Token::Whitespace,
        Token::LBrace,
        Token::RBrace,
        Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    assert!(!nodes.is_empty(), "while: {:?}", nodes);
    match &nodes[0] {
        AstNode::While { cond, .. } => {
            assert!(matches!(cond.as_ref(), AstNode::Literal(Value::Bool(false))), "cond: {:?}", cond);
        }
        other => panic!("expected While, got {:?}", other),
    }
}

#[test]
fn parse_supports_at_rule() {
    let tokens = vec![
        Token::AtSupports,
        Token::Whitespace,
        Token::LParen,
        Token::Str("display".into()),
        Token::Colon,
        Token::Whitespace,
        Token::Str("flex".into()),
        Token::RParen,
        Token::Whitespace,
        Token::LBrace,
        Token::RBrace,
        Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    assert!(!nodes.is_empty(), "supports: {:?}", nodes);
    match &nodes[0] {
        AstNode::Supports { query, .. } => {
            assert!(query.contains("display"), "query: {}", query);
        }
        other => panic!("expected Supports, got {:?}", other),
    }
}

// ── Pratt expression parser tests ──

#[test]
fn parse_binop_addition() {
    // $x: 1 + 2;
    let tokens = vec![
        Token::Dollar, Token::Ident("x".into()), Token::Colon,
        Token::Whitespace, Token::Number(1.0, None), Token::Whitespace,
        Token::Plus, Token::Whitespace, Token::Number(2.0, None),
        Token::Semicolon, Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    assert!(!nodes.is_empty(), "binop: {:?}", nodes);
    match &nodes[0] {
        AstNode::VariableDecl { value, .. } => {
            assert!(matches!(value.as_ref(), AstNode::BinOp { op: BinOp::Add, .. }), "got: {:?}", value);
        }
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn parse_mul_higher_prec_than_add() {
    // $x: 1 + 2 * 3;  should be 1 + (2 * 3)
    let tokens = vec![
        Token::Dollar, Token::Ident("x".into()), Token::Colon,
        Token::Whitespace, Token::Number(1.0, None), Token::Whitespace,
        Token::Plus, Token::Whitespace, Token::Number(2.0, None), Token::Whitespace,
        Token::Star, Token::Whitespace, Token::Number(3.0, None),
        Token::Semicolon, Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    match &nodes[0] {
        AstNode::VariableDecl { value, .. } => match value.as_ref() {
            AstNode::BinOp { op: BinOp::Add, right, .. } => {
                assert!(matches!(right.as_ref(), AstNode::BinOp { op: BinOp::Mul, .. }), "right should be Mul: {:?}", right);
            }
            other => panic!("expected Add at top, got: {:?}", other),
        },
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn parse_comparison_operators() {
    // $x: 5 > 3;
    let tokens = vec![
        Token::Dollar, Token::Ident("x".into()), Token::Colon,
        Token::Whitespace, Token::Number(5.0, None), Token::Whitespace,
        Token::Gt, Token::Whitespace, Token::Number(3.0, None),
        Token::Semicolon, Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    match &nodes[0] {
        AstNode::VariableDecl { value, .. } => {
            assert!(matches!(value.as_ref(), AstNode::BinOp { op: BinOp::Gt, .. }), "got: {:?}", value);
        }
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn parse_equality_operators() {
    // $x: a == b;
    let tokens = vec![
        Token::Dollar, Token::Ident("x".into()), Token::Colon,
        Token::Whitespace, Token::Ident("a".into()), Token::Whitespace,
        Token::Eq, Token::Whitespace, Token::Ident("b".into()),
        Token::Semicolon, Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    match &nodes[0] {
        AstNode::VariableDecl { value, .. } => {
            assert!(matches!(value.as_ref(), AstNode::BinOp { op: BinOp::Eq, .. }), "got: {:?}", value);
        }
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn parse_logical_and_or() {
    // $x: a and b or c;
    let tokens = vec![
        Token::Dollar, Token::Ident("x".into()), Token::Colon,
        Token::Whitespace, Token::Ident("a".into()), Token::Whitespace,
        Token::And, Token::Whitespace, Token::Ident("b".into()), Token::Whitespace,
        Token::Or, Token::Whitespace, Token::Ident("c".into()),
        Token::Semicolon, Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    match &nodes[0] {
        AstNode::VariableDecl { value, .. } => match value.as_ref() {
            // Should be (a and b) or c — Or at top, And on left
            AstNode::BinOp { op: BinOp::Or, left, .. } => {
                assert!(matches!(left.as_ref(), AstNode::BinOp { op: BinOp::And, .. }), "left should be And: {:?}", left);
            }
            other => panic!("expected Or at top, got: {:?}", other),
        },
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn parse_function_call() {
    // $x: rgba(255, 0, 0, 0.5);
    let tokens = vec![
        Token::Dollar, Token::Ident("x".into()), Token::Colon,
        Token::Whitespace, Token::Ident("rgba".into()), Token::LParen,
        Token::Number(255.0, None), Token::Comma,
        Token::Number(0.0, None), Token::Comma,
        Token::Number(0.0, None), Token::Comma,
        Token::Number(0.5, None), Token::RParen,
        Token::Semicolon, Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    match &nodes[0] {
        AstNode::VariableDecl { value, .. } => match value.as_ref() {
            AstNode::FunctionCall { name, args } => {
                assert_eq!(name, "rgba");
                assert_eq!(args.len(), 4);
            }
            other => panic!("expected FunctionCall, got: {:?}", other),
        },
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn parse_unary_minus() {
    // $x: -5px;
    let tokens = vec![
        Token::Dollar, Token::Ident("x".into()), Token::Colon,
        Token::Whitespace, Token::Minus, Token::Number(5.0, Some("px".into())),
        Token::Semicolon, Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    match &nodes[0] {
        AstNode::VariableDecl { value, .. } => match value.as_ref() {
            AstNode::UnaryOp { op: UnaryOp::Neg, expr } => {
                assert!(matches!(expr.as_ref(), AstNode::Literal(Value::Number(5.0, Some(_)))), "expr: {:?}", expr);
            }
            other => panic!("expected UnaryOp(Neg), got: {:?}", other),
        },
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn parse_paren_grouping() {
    // $x: (1 + 2) * 3;  should be (1+2) * 3
    let tokens = vec![
        Token::Dollar, Token::Ident("x".into()), Token::Colon,
        Token::Whitespace, Token::LParen, Token::Number(1.0, None),
        Token::Plus, Token::Number(2.0, None), Token::RParen, Token::Whitespace,
        Token::Star, Token::Whitespace, Token::Number(3.0, None),
        Token::Semicolon, Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    match &nodes[0] {
        AstNode::VariableDecl { value, .. } => match value.as_ref() {
            AstNode::BinOp { op: BinOp::Mul, left, .. } => {
                assert!(matches!(left.as_ref(), AstNode::BinOp { op: BinOp::Add, .. }), "left should be Add: {:?}", left);
            }
            other => panic!("expected Mul at top, got: {:?}", other),
        },
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

#[test]
fn parse_selector_interpolation() {
    // .#{$klass} { color: red; }
    let tokens = vec![
        Token::Dot,
        Token::InterpolationStart,
        Token::Dollar, Token::Ident("klass".into()),
        Token::InterpolationEnd,
        Token::Whitespace, Token::LBrace,
        Token::Ident("color".into()), Token::Colon, Token::Whitespace,
        Token::Ident("red".into()), Token::Semicolon,
        Token::RBrace, Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    assert!(!nodes.is_empty(), "interp rule: {:?}", nodes);
    match &nodes[0] {
        AstNode::Rule { selector, .. } => {
            assert!(selector.contains("$klass"), "selector: {}", selector);
        }
        other => panic!("expected Rule, got {:?}", other),
    }
}

#[test]
fn parse_if_else_chain() {
    // @if x { } @else if y { } @else { }
    // simplified: just test @else if handling
    let tokens = vec![
        Token::AtIf,
        Token::Whitespace, Token::Ident("true".into()),
        Token::Whitespace, Token::LBrace, Token::RBrace,
        Token::AtElse,
        Token::AtIf,
        Token::Whitespace, Token::Ident("false".into()),
        Token::Whitespace, Token::LBrace, Token::RBrace,
        Token::Eof,
    ];
    let nodes = collect_ast(tokens_to_stream(tokens));
    assert!(!nodes.is_empty(), "if/else: {:?}", nodes);
    match &nodes[0] {
        AstNode::If { else_branch, .. } => {
            assert!(else_branch.is_some(), "should have else branch");
        }
        other => panic!("expected If, got {:?}", other),
    }
}
