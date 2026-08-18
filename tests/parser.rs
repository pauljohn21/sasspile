//! Parser tests — tests parsing of SCSS source into AST.

use sasspile::tokenize;
use sasspile::parse;
use sasspile::ast::Stmt;

fn parse_src(src: &str) -> Vec<Stmt> {
    let tokens = tokenize(src, "test_parser").expect("tokenize should succeed");
    parse(tokens).expect("parse should succeed")
}

#[test]
fn test_parse_empty() {
    let stmts = parse_src("");
    assert!(stmts.is_empty());
}

#[test]
fn test_parse_simple_rule() {
    let stmts = parse_src("a { color: red; }");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::StyleRule { selector, body } => {
            assert_eq!(selector, "a");
            assert!(!body.is_empty());
        }
        _ => panic!("expected StyleRule"),
    }
}

#[test]
fn test_parse_variable_decl() {
    let stmts = parse_src("$color: red;");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::VariableDecl { name, default, global, .. } => {
            assert_eq!(name, "color");
            assert!(!*default);
            assert!(!*global);
        }
        _ => panic!("expected VariableDecl"),
    }
}

#[test]
fn test_parse_variable_default() {
    let stmts = parse_src("$color: red !default;");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::VariableDecl { default, .. } => assert!(*default),
        _ => panic!("expected VariableDecl"),
    }
}

#[test]
fn test_parse_variable_global() {
    let stmts = parse_src("$color: red !global;");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::VariableDecl { global, .. } => assert!(*global),
        _ => panic!("expected VariableDecl"),
    }
}

#[test]
fn test_parse_multiple_rules() {
    let stmts = parse_src("a { color: red; } b { color: blue; }");
    assert_eq!(stmts.len(), 2);
    assert!(matches!(&stmts[0], Stmt::StyleRule { selector, .. } if selector == "a"));
    assert!(matches!(&stmts[1], Stmt::StyleRule { selector, .. } if selector == "b"));
}

#[test]
fn test_parse_nested_rules() {
    let stmts = parse_src(".parent { .child { color: red; } }");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::StyleRule { selector, body } => {
            assert_eq!(selector, ".parent");
            assert!(!body.is_empty());
            assert!(matches!(&body[0], Stmt::StyleRule { selector, .. } if selector == ".child"));
        }
        _ => panic!("expected StyleRule"),
    }
}

#[test]
fn test_parse_mixin_def() {
    let stmts = parse_src("@mixin foo { color: red; }");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::MixinDef { name, .. } if name == "foo"));
}

#[test]
fn test_parse_mixin_with_params() {
    let stmts = parse_src("@mixin foo($a, $b: 10px) { color: $a; }");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::MixinDef { name, params, .. } => {
            assert_eq!(name, "foo");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].name, "a");
            assert_eq!(params[1].name, "b");
            assert!(params[1].default.is_some());
        }
        _ => panic!("expected MixinDef"),
    }
}

#[test]
fn test_parse_include() {
    let stmts = parse_src("@include foo;");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::IncludeCall { name, .. } if name == "foo"));
}

#[test]
fn test_parse_function_def() {
    let stmts = parse_src("@function foo($a) { @return $a; }");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::FunctionDef { name, .. } if name == "foo"));
}

#[test]
fn test_parse_if() {
    let stmts = parse_src("@if true { color: red; }");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::IfStmt { .. }));
}

#[test]
fn test_parse_if_else() {
    let src = "@if true { color: red; } @else { color: blue; }";
    let stmts = parse_src(src);
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::IfStmt { branches, else_body } => {
            assert_eq!(branches.len(), 1);
            assert!(else_body.is_some());
        }
        _ => panic!("expected IfStmt"),
    }
}

#[test]
fn test_parse_for() {
    let stmts = parse_src("@for $i from 1 through 3 { .item-#{$i} { width: $i; } }");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::ForStmt { var, .. } if var == "i"));
}

#[test]
fn test_parse_each() {
    let stmts = parse_src("@each $color in red, blue { color: $color; }");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::EachStmt { vars, .. } if vars[0] == "color"));
}

#[test]
fn test_parse_while() {
    let stmts = parse_src("@while true { color: red; }");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::WhileStmt { .. }));
}

#[test]
fn test_parse_extend() {
    let stmts = parse_src("@extend .foo;");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::ExtendRule { selector, .. } if selector == ".foo"));
}

#[test]
fn test_parse_media() {
    let stmts = parse_src("@media screen { color: red; }");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::MediaRule { .. }));
}

#[test]
fn test_parse_use() {
    let stmts = parse_src("@use \"sass:math\";");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::UseRule { url, .. } if url == "sass:math"));
}

#[test]
fn test_parse_import() {
    let stmts = parse_src("@import \"foo.scss\";");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::ImportRule(url) if url == "foo.scss"));
}

#[test]
fn test_parse_block_comment() {
    let stmts = parse_src("/* comment */");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::Comment(text) if text.contains("comment")));
}

#[test]
fn test_parse_expression_arithmetic() {
    let stmts = parse_src("$x: 1 + 2 * 3;");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::VariableDecl { .. }));
}
