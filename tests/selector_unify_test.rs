//! 选择器 unify 算法测试。

use sasspile::css::selector_parser::parse_selector;
use sasspile::css::selector_ops;

#[test]
fn test_unify_same_class() {
    let a = parse_selector(".foo");
    let b = parse_selector(".foo");
    let result = selector_ops::unify(&a, &b);
    assert_eq!(result.map(|s| s.to_string()), Some(".foo".to_string()));
}

#[test]
fn test_unify_different_classes() {
    let a = parse_selector(".foo");
    let b = parse_selector(".bar");
    let result = selector_ops::unify(&a, &b);
    assert_eq!(result.map(|s| s.to_string()), Some(".foo.bar".to_string()));
}

#[test]
fn test_unify_type_conflict() {
    let a = parse_selector("div");
    let b = parse_selector("span");
    let result = selector_ops::unify(&a, &b);
    assert_eq!(result, None);
}

#[test]
fn test_unify_id_conflict() {
    let a = parse_selector("#main");
    let b = parse_selector("#other");
    let result = selector_ops::unify(&a, &b);
    assert_eq!(result, None);
}

#[test]
fn test_unify_universal_with_type() {
    let a = parse_selector("*");
    let b = parse_selector("div");
    let result = selector_ops::unify(&a, &b);
    assert_eq!(result.map(|s| s.to_string()), Some("div".to_string()));
}

#[test]
fn test_unify_descendant() {
    let a = parse_selector("a b");
    let b = parse_selector("c d");
    let result = selector_ops::unify(&a, &b);
    // a b + c d: 最后一个复合 b 和 d 类型冲突 → None
    assert_eq!(result, None);
}

#[test]
fn test_unify_descendant_same_type() {
    let a = parse_selector(".a b");
    let b = parse_selector(".c b");
    let result = selector_ops::unify(&a, &b);
    // .a b + .c b: 最后一个复合 b 和 b 统一 = b，前缀 .a + .c
    assert!(result.is_some());
    let s = result.unwrap().to_string();
    assert!(s.contains("b"));
}

#[test]
fn test_unify_class_with_id() {
    let a = parse_selector(".foo");
    let b = parse_selector("#bar");
    let result = selector_ops::unify(&a, &b);
    assert_eq!(result.map(|s| s.to_string()), Some("#bar.foo".to_string()));
}

#[test]
fn test_unify_comma_list() {
    let a = parse_selector(".a, .b");
    let b = parse_selector(".c");
    let result = selector_ops::unify(&a, &b);
    assert!(result.is_some());
    let s = result.unwrap().to_string();
    assert!(s.contains(".a.c") || s.contains(".c.a"));
    assert!(s.contains(".b.c") || s.contains(".c.b"));
}
