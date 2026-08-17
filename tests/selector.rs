//! Selector engine tests — tests parsing, unification, and extension.

use sasspile::selector::parse::parse_selector_list;
use sasspile::selector::extend::{ExtendTable, apply_extends_to_selector};
use sasspile::selector::{unify, CompoundSelector, SimpleSelector};

#[test]
fn test_parse_simple_class() {
    let list = parse_selector_list(".foo");
    assert_eq!(list.selectors.len(), 1);
    assert_eq!(list.to_string(), ".foo");
}

#[test]
fn test_parse_compound_selector() {
    let list = parse_selector_list("div.foo.bar");
    assert_eq!(list.selectors[0].leading.components.len(), 3);
    assert_eq!(list.to_string(), "div.foo.bar");
}

#[test]
fn test_parse_complex_selector() {
    let list = parse_selector_list("div > .foo + p");
    assert_eq!(list.selectors[0].rest.len(), 2);
    assert_eq!(list.to_string(), "div > .foo + p");
}

#[test]
fn test_parse_selector_list() {
    let list = parse_selector_list("div, .foo, #bar");
    assert_eq!(list.selectors.len(), 3);
    assert_eq!(list.to_string(), "div, .foo, #bar");
}

#[test]
fn test_parse_placeholder() {
    let list = parse_selector_list("%base");
    assert!(list.selectors[0].leading.has_placeholder());
}

#[test]
fn test_parse_pseudo_class() {
    let list = parse_selector_list("a:hover");
    assert_eq!(list.to_string(), "a:hover");
}

#[test]
fn test_parse_pseudo_element() {
    let list = parse_selector_list("a::before");
    assert_eq!(list.to_string(), "a::before");
}

#[test]
fn test_parse_pseudo_with_arg() {
    let list = parse_selector_list(":nth-child(2n+1)");
    assert!(list.to_string().contains("nth-child"));
}

#[test]
fn test_parse_attribute_selector() {
    let list = parse_selector_list("[type=\"text\"]");
    assert!(list.to_string().contains("type"));
}

#[test]
fn test_unify_class_and_type() {
    let mut a = CompoundSelector::new();
    a.add(SimpleSelector::Type("div".to_string()));
    let mut b = CompoundSelector::new();
    b.add(SimpleSelector::Class("foo".to_string()));

    let result = unify(&a, &b).unwrap();
    assert_eq!(result.to_string(), "div.foo");
}

#[test]
fn test_unify_two_classes() {
    let mut a = CompoundSelector::new();
    a.add(SimpleSelector::Class("foo".to_string()));
    let mut b = CompoundSelector::new();
    b.add(SimpleSelector::Class("bar".to_string()));

    let result = unify(&a, &b).unwrap();
    assert_eq!(result.to_string(), ".foo.bar");
}

#[test]
fn test_extend_simple() {
    let mut table = ExtendTable::new();
    table.add(".bar".to_string(), ".foo".to_string(), false);
    let result = apply_extends_to_selector(".foo", &table);
    assert!(result.contains(".foo"), "should contain .foo: {}", result);
    assert!(result.contains(".bar"), "should contain .bar: {}", result);
}

#[test]
fn test_extend_placeholder() {
    let mut table = ExtendTable::new();
    table.add(".foo".to_string(), "%base".to_string(), false);
    let result = apply_extends_to_selector("%base", &table);
    // Placeholder should be replaced by .foo
    assert!(result.contains(".foo"), "should contain .foo: {}", result);
}

#[test]
fn test_is_superselector() {
    let parent = parse_selector_list(".foo");
    let child = parse_selector_list(".foo.bar");
    assert!(parent.is_superselector(&child));
}

#[test]
fn test_remove_placeholders() {
    let mut list = parse_selector_list("%base.foo");
    assert!(list.has_placeholder());
    list.remove_placeholders();
    assert!(!list.has_placeholder());
    assert_eq!(list.to_string(), ".foo");
}
