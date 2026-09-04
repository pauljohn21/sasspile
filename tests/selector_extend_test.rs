//! 选择器 extend/replace 算法测试。

use sasspile::css::selector_parser::parse_selector;
use sasspile::css::selector_ops;

#[test]
fn test_extend_simple() {
    let selector = parse_selector(".foo");
    let extendee = parse_selector(".foo");
    let extender = parse_selector(".bar");
    let result = selector_ops::extend_selector(&selector, &extendee, &extender);
    let s = result.to_string();
    assert!(s.contains(".foo"));
    assert!(s.contains(".bar"));
}

#[test]
fn test_extend_no_match() {
    let selector = parse_selector(".foo");
    let extendee = parse_selector(".bar");
    let extender = parse_selector(".baz");
    let result = selector_ops::extend_selector(&selector, &extendee, &extender);
    assert_eq!(result.to_string(), ".foo");
}

#[test]
fn test_extend_descendant() {
    let selector = parse_selector(".foo .bar");
    let extendee = parse_selector(".bar");
    let extender = parse_selector(".baz");
    let result = selector_ops::extend_selector(&selector, &extendee, &extender);
    let s = result.to_string();
    assert!(s.contains(".foo .bar"));
    // 应该也包含扩展后的选择器
    assert!(s.contains(".baz"));
}

#[test]
fn test_replace_simple() {
    let selector = parse_selector(".foo .bar");
    let original = parse_selector(".bar");
    let replacement = parse_selector(".baz");
    let result = selector_ops::replace_selector(&selector, &original, &replacement);
    let s = result.to_string();
    assert!(s.contains(".baz"));
    assert!(!s.contains(".bar"));
}

#[test]
fn test_replace_no_match() {
    let selector = parse_selector(".foo");
    let original = parse_selector(".bar");
    let replacement = parse_selector(".baz");
    let result = selector_ops::replace_selector(&selector, &original, &replacement);
    assert_eq!(result.to_string(), ".foo");
}
