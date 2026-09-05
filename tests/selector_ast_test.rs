//! 选择器 AST 解析 + 序列化 round-trip 测试。

use sasspile::css::selector_parser::parse_selector;

fn roundtrip(input: &str) -> String {
    parse_selector(input).to_string()
}

#[test]
fn test_simple_type() {
    assert_eq!(roundtrip("div"), "div");
}

#[test]
fn test_class() {
    assert_eq!(roundtrip(".btn"), ".btn");
}

#[test]
fn test_id() {
    assert_eq!(roundtrip("#main"), "#main");
}

#[test]
fn test_compound() {
    assert_eq!(roundtrip("div.btn"), "div.btn");
    assert_eq!(roundtrip(".a.b"), ".a.b");
}

#[test]
fn test_descendant() {
    assert_eq!(roundtrip("a b"), "a b");
}

#[test]
fn test_child_combinator() {
    assert_eq!(roundtrip("a > b"), "a > b");
}

#[test]
fn test_adjacent_combinator() {
    assert_eq!(roundtrip("a + b"), "a + b");
}

#[test]
fn test_sibling_combinator() {
    assert_eq!(roundtrip("a ~ b"), "a ~ b");
}

#[test]
fn test_comma_list() {
    assert_eq!(roundtrip("a, b, c"), "a, b, c");
}

#[test]
fn test_universal() {
    assert_eq!(roundtrip("*"), "*");
}

#[test]
fn test_placeholder() {
    assert_eq!(roundtrip("%button"), "%button");
}

#[test]
fn test_pseudo_class() {
    assert_eq!(roundtrip("a:hover"), "a:hover");
}

#[test]
fn test_pseudo_class_with_arg() {
    assert_eq!(roundtrip("a:nth-child(2n+1)"), "a:nth-child(2n+1)");
}

#[test]
fn test_pseudo_element() {
    assert_eq!(roundtrip("a::before"), "a::before");
}

#[test]
fn test_complex_selector() {
    assert_eq!(roundtrip(".foo .bar"), ".foo .bar");
    assert_eq!(roundtrip(".foo > .bar"), ".foo > .bar");
}

#[test]
fn test_multiple_compounds() {
    assert_eq!(roundtrip("div.foo.bar"), "div.foo.bar");
}

#[test]
fn test_mixed_comma_and_combinator() {
    assert_eq!(roundtrip("a b, c d"), "a b, c d");
}
