//! calc AST 解析 + 序列化 round-trip 测试。

use sasspile::eval::value::calc_ast::{parse_calc_expr, CalcNode};

fn roundtrip(input: &str) -> Option<String> {
    parse_calc_expr(input).map(|n| n.to_string())
}

#[test]
fn test_parse_number() {
    assert_eq!(roundtrip("1px"), Some("1px".to_string()));
    assert_eq!(roundtrip("42"), Some("42".to_string()));
    assert_eq!(roundtrip("3.14"), Some("3.14".to_string()));
}

#[test]
fn test_parse_add() {
    assert_eq!(roundtrip("1px + 2px"), Some("1px + 2px".to_string()));
}

#[test]
fn test_parse_mul() {
    assert_eq!(roundtrip("2 * 3px"), Some("2 * 3px".to_string()));
}

#[test]
fn test_parse_parens() {
    let r = roundtrip("(1px + 2px) * 3");
    assert!(r.is_some());
}

#[test]
fn test_parse_func() {
    let r = roundtrip("min(1px, 2px)");
    assert!(r.is_some());
}

#[test]
fn test_parse_var() {
    let r = roundtrip("var(--x)");
    assert!(r.is_some());
}

#[test]
fn test_parse_precedence() {
    // 1 + 2 * 3 → 1 + (2 * 3)
    let node = parse_calc_expr("1 + 2 * 3");
    assert!(matches!(
        node,
        Some(CalcNode::Op {
            op: sasspile::eval::value::calc_ast::CalcOp::Add,
            ..
        })
    ));
}

#[test]
fn test_parse_complex() {
    let r = roundtrip("1px + var(--x) * 2");
    assert!(r.is_some());
}
