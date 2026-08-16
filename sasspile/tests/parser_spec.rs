//! Parser specification tests.
//!
//! See: openspec/changes/scss-compiler/specs/parser/spec.md

use sasspile::*;
use sasspile::lexer;

/// Helper to tokenize and parse sass source.
fn parse(source: &str) -> (Stylesheet, sasspile::diagnostics::Diagnostics) {
    let (tokens, _) = lexer::tokenize(source);
    let parser = Parser::new(&tokens);
    parser.parse()
}

#[test]
fn rule_with_declaration() {
    let (stylesheet, diags) = parse("a { color: red; }");
    assert!(diags.is_empty(), "no diagnostics expected, got: {:?}", diags);
    assert_eq!(stylesheet.nodes.len(), 1);
    match &stylesheet.nodes[0] {
        Node::Rule(rule) => {
            assert_eq!(rule.selector, Selector::Type("a".to_string()));
            assert_eq!(rule.nodes.len(), 1);
            match &rule.nodes[0] {
                Node::Declaration(decl) => {
                    assert_eq!(decl.name, "color");
                    assert!(!decl.important);
                }
                other => panic!("expected Declaration, got {:?}", other),
            }
        }
        other => panic!("expected Rule, got {:?}", other),
    }
}

#[test]
fn nested_rule() {
    let (stylesheet, _) = parse("parent { color: red; child { color: blue; } }");
    assert_eq!(stylesheet.nodes.len(), 1);
    if let Node::Rule(parent) = &stylesheet.nodes[0] {
        assert_eq!(parent.nodes.len(), 2);
        match &parent.nodes[1] {
            Node::Rule(child) => {
                assert_eq!(child.selector, Selector::Type("child".to_string()));
            }
            other => panic!("expected nested Rule, got {:?}", other),
        }
    }
}

#[test]
fn declaration_with_important() {
    let (stylesheet, _) = parse("a { font-size: 16px !important; }");
    if let Node::Rule(rule) = &stylesheet.nodes[0] {
        if let Node::Declaration(decl) = &rule.nodes[0] {
            assert_eq!(decl.name, "font-size");
            assert!(decl.important);
        }
    }
}

#[test]
fn media_at_rule() {
    let (stylesheet, diags) = parse("@media screen { a { color: red; } }");
    assert!(diags.is_empty());
    assert_eq!(stylesheet.nodes.len(), 1);
    match &stylesheet.nodes[0] {
        Node::AtRule(AtRule::Media(media)) => {
            assert!(media.query.contains("screen"));
            assert_eq!(media.body.len(), 1);
        }
        other => panic!("expected Media, got {:?}", other),
    }
}

#[test]
fn supports_at_rule() {
    let (stylesheet, _) = parse("@supports (display: flex) { a { color: blue; } }");
    assert_eq!(stylesheet.nodes.len(), 1);
    match &stylesheet.nodes[0] {
        Node::AtRule(AtRule::Supports(sup)) => {
            assert!(sup.condition.contains("display"));
            assert_eq!(sup.body.len(), 1);
        }
        other => panic!("expected Supports, got {:?}", other),
    }
}

#[test]
fn use_at_rule() {
    let (stylesheet, _) = parse(r#"@use "module" as m;"#);
    assert_eq!(stylesheet.nodes.len(), 1);
    match &stylesheet.nodes[0] {
        Node::AtRule(AtRule::Use(use_rule)) => {
            assert_eq!(use_rule.url, "module");
            assert_eq!(use_rule.namespace, Some("m".to_string()));
        }
        other => panic!("expected Use, got {:?}", other),
    }
}

#[test]
fn import_at_rule() {
    let (stylesheet, _) = parse(r#"@import "styles.css";"#);
    assert_eq!(stylesheet.nodes.len(), 1);
    match &stylesheet.nodes[0] {
        Node::AtRule(AtRule::Import(import)) => {
            assert_eq!(import.urls.len(), 1);
            assert_eq!(import.urls[0], "styles.css");
        }
        other => panic!("expected Import, got {:?}", other),
    }
}

#[test]
fn function_call_expression() {
    let (stylesheet, _) = parse("a { width: calc(100% - 20px); }");
    if let Node::Rule(rule) = &stylesheet.nodes[0] {
        if let Node::Declaration(decl) = &rule.nodes[0] {
            match &decl.value {
                Expr::Call(name, args) => {
                    assert_eq!(name, "calc");
                    assert_eq!(args.len(), 1);
                }
                expr => panic!("expected Call expression, got {:?}", expr),
            }
        }
    }
}

#[test]
fn number_with_unit() {
    let (stylesheet, _) = parse("a { size: 16px; }");
    if let Node::Rule(rule) = &stylesheet.nodes[0] {
        if let Node::Declaration(decl) = &rule.nodes[0] {
            match &decl.value {
                Expr::Number(val, unit) => {
                    assert_eq!(*val, 16.0);
                    assert_eq!(unit.as_deref(), Some("px"));
                }
                other => panic!("expected Number, got {:?}", other),
            }
        }
    }
}

#[test]
fn string_value() {
    let (stylesheet, _) = parse("a { font: \"Helvetica\"; }");
    if let Node::Rule(rule) = &stylesheet.nodes[0] {
        if let Node::Declaration(decl) = &rule.nodes[0] {
            match &decl.value {
                Expr::String(s) => {
                    assert_eq!(s, "Helvetica");
                }
                other => panic!("expected String, got {:?}", other),
            }
        }
    }
}

#[test]
fn debug_at_rule() {
    let (stylesheet, _) = parse("@debug 42;");
    assert_eq!(stylesheet.nodes.len(), 1);
    match &stylesheet.nodes[0] {
        Node::AtRule(AtRule::Debug(Expr::Number(val, _))) => {
            assert_eq!(*val, 42.0);
        }
        other => panic!("expected Debug with Number, got {:?}", other),
    }
}

#[test]
fn empty_stylesheet() {
    let (stylesheet, diags) = parse("");
    assert!(diags.is_empty());
    assert!(stylesheet.nodes.is_empty());
}

#[test]
fn multiple_top_level_nodes() {
    let (stylesheet, _) = parse("a { color: red; } b { color: blue; }");
    assert_eq!(stylesheet.nodes.len(), 2);
}

#[test]
fn mixin_definition() {
    let (stylesheet, _) = parse("@mixin foo($a, $b) { color: red; }");
    assert_eq!(stylesheet.nodes.len(), 1);
    match &stylesheet.nodes[0] {
        Node::AtRule(AtRule::Mixin(mixin_def)) => {
            assert_eq!(mixin_def.name, "foo");
            assert_eq!(mixin_def.params.len(), 2);
        }
        other => panic!("expected Mixin, got {:?}", other),
    }
}

#[test]
fn mixin_with_default_params() {
    let (stylesheet, _) = parse("@mixin bar($x: 10px) { width: $x; }");
    match &stylesheet.nodes[0] {
        Node::AtRule(AtRule::Mixin(mixin_def)) => {
            assert_eq!(mixin_def.params.len(), 1);
            assert!(mixin_def.params[0].default.is_some());
        }
        _ => panic!("expected Mixin"),
    }
}

#[test]
fn include_at_rule() {
    let (stylesheet, _) = parse("@include foo(1, 2);");
    match &stylesheet.nodes[0] {
        Node::AtRule(AtRule::Include(inc)) => {
            assert_eq!(inc.name, "foo");
            assert_eq!(inc.args.len(), 2);
        }
        _ => panic!("expected Include"),
    }
}

#[test]
fn parent_selector() {
    let (stylesheet, _) = parse("a { &:hover { color: red; } }");
    if let Node::Rule(parent) = &stylesheet.nodes[0] {
        if let Node::Rule(child) = &parent.nodes[0] {
            assert_eq!(
                child.selector,
                Selector::ParentRef(Box::new(Selector::Pseudo("hover".to_string())))
            );
        }
    }
}

#[test]
fn additive_expression() {
    let (stylesheet, _) = parse("a { width: 10px + 5; }");
    if let Node::Rule(rule) = &stylesheet.nodes[0] {
        if let Node::Declaration(decl) = &rule.nodes[0] {
            match &decl.value {
                Expr::Binary(BinaryOp::Add, left, _) => {
                    assert_eq!(**left, Expr::Number(10.0, Some("px".to_string())));
                }
                other => panic!("expected Add binary, got {:?}", other),
            }
        }
    }
}

#[test]
fn color_value() {
    let (stylesheet, _) = parse("a { color: #ff0000; }");
    if let Node::Rule(rule) = &stylesheet.nodes[0] {
        if let Node::Declaration(decl) = &rule.nodes[0] {
            match &decl.value {
                Expr::Color(_) => {} // good
                other => panic!("expected Color, got {:?}", other),
            }
        }
    }
}
