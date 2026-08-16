//! Tests for semantic/extend.

use sasspile::{Node, Rule, Selector, Stylesheet};
use sasspile::semantic::SelectorRegistry;

#[test]
fn new_registry_is_empty() {
    let reg = SelectorRegistry::new();
    assert!(reg.is_empty());
    assert_eq!(reg.len(), 0);
}

#[test]
fn register_class_selector() {
    let mut reg = SelectorRegistry::new();
    let sel = Selector::Class("button".to_string());
    reg.register_selector(&sel);

    assert!(reg.has_class("button"));
    assert!(!reg.has_class("link"));
    assert_eq!(reg.len(), 1);
}

#[test]
fn register_id_selector() {
    let mut reg = SelectorRegistry::new();
    let sel = Selector::Id("main".to_string());
    reg.register_selector(&sel);

    assert!(reg.has_id("main"));
    assert!(!reg.has_id("sidebar"));
}

#[test]
fn register_type_selector() {
    let mut reg = SelectorRegistry::new();
    let sel = Selector::Type("div".to_string());
    reg.register_selector(&sel);

    assert!(reg.has_type("div"));
    assert!(!reg.has_type("span"));
}

#[test]
fn register_compound_selector_registers_parts() {
    let mut reg = SelectorRegistry::new();
    let compound = Selector::Compound(vec![
        Selector::Class("btn".to_string()),
        Selector::Type("button".to_string()),
    ]);
    reg.register_selector(&compound);

    assert!(reg.has_class("btn"));
    assert!(reg.has_type("button"));
}

#[test]
fn validate_extend_known_class() {
    let mut reg = SelectorRegistry::new();
    reg.register_selector(&Selector::Class("btn".to_string()));

    let mut diags = sasspile::diagnostics::Diagnostics::new();
    let valid = reg.validate_extend(&Selector::Class("btn".to_string()), &mut diags);
    assert!(valid);
    assert!(diags.is_empty());
}

#[test]
fn validate_extend_unknown_class() {
    let reg = SelectorRegistry::new();
    let mut diags = sasspile::diagnostics::Diagnostics::new();
    let valid = reg.validate_extend(
        &Selector::Class("unknown".to_string()),
        &mut diags,
    );
    assert!(!valid);
    assert!(diags.has_errors());
}

#[test]
fn from_stylesheet_collects_selectors() {
    let stylesheet = Stylesheet {
        nodes: vec![Node::Rule(Rule {
            selector: Selector::Class("card".to_string()),
            nodes: vec![],
        })],
    };

    let reg = SelectorRegistry::from_stylesheet(&stylesheet);
    assert!(reg.has_class("card"));
    assert!(!reg.has_class("unknown"));
}
