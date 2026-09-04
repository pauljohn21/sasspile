//! 选择器 is_superselector 算法测试。

use sasspile::css::selector_parser::parse_selector;
use sasspile::css::selector_ops;

#[test]
fn test_super_same() {
    let a = parse_selector(".foo");
    assert!(selector_ops::is_superselector(&a, &a));
}

#[test]
fn test_super_class_subset() {
    let super_ = parse_selector(".foo");
    let sub = parse_selector(".foo.bar");
    assert!(selector_ops::is_superselector(&super_, &sub));
}

#[test]
fn test_super_class_not_subset() {
    let super_ = parse_selector(".foo.bar");
    let sub = parse_selector(".foo");
    assert!(!selector_ops::is_superselector(&super_, &sub));
}

#[test]
fn test_super_universal() {
    let super_ = parse_selector("*");
    let sub = parse_selector("div.foo");
    assert!(selector_ops::is_superselector(&super_, &sub));
}

#[test]
fn test_super_descendant() {
    let super_ = parse_selector("a");
    let sub = parse_selector("a b");
    assert!(selector_ops::is_superselector(&super_, &sub));
}

#[test]
fn test_super_type() {
    let super_ = parse_selector("div");
    let sub = parse_selector("div.foo");
    assert!(selector_ops::is_superselector(&super_, &sub));
}

#[test]
fn test_super_different_type() {
    let super_ = parse_selector("div");
    let sub = parse_selector("span");
    assert!(!selector_ops::is_superselector(&super_, &sub));
}

#[test]
fn test_super_comma_list() {
    let super_ = parse_selector("a");
    let sub = parse_selector("a b, a c");
    assert!(selector_ops::is_superselector(&super_, &sub));
}
