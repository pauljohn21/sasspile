//! CSS generation tests.
//!
//! Verifies that the new css module correctly transforms
//! parsed AST into CSS output.

use sasspile::css::{self, OutputStyle};
use sasspile::{Parser, Stylesheet};
use sasspile::lexer;

fn parse_and_generate(source: &str, style: OutputStyle) -> String {
    let (tokens, _) = lexer::tokenize(source);
    let parser = Parser::new(&tokens);
    let (stylesheet, _) = parser.parse();
    css::generate(&stylesheet, style).unwrap_or_default()
}

#[test]
fn simple_rule_to_css() {
    let css = parse_and_generate("a { color: blue; }", OutputStyle::Expanded);
    assert!(css.contains("a"), "should contain selector 'a', got: {css}");
    assert!(css.contains("color"), "should contain property name 'color', got: {css}");
    assert!(css.contains("blue"), "should contain value 'blue', got: {css}");
}

#[test]
fn nested_rule_expansion() {
    let css = parse_and_generate(
        "parent { color: blue; child { color: green; } }",
        OutputStyle::Expanded,
    );
    assert!(css.contains("parent"), "should contain 'parent'");
    assert!(css.contains("parent child"),
        "should contain nested selector, got: {css}");
}

#[test]
fn compressed_output() {
    let css = parse_and_generate("a { color: blue; }", OutputStyle::Compressed);
    // Compressed should have no newlines.
    assert!(!css.contains('\n'), "compressed output should have no newlines, got: {css}");
    // Compressed format: selector{props}
    assert!(css.contains("a{"), "compressed format should have 'a{{', got: {css}");
}

#[test]
fn multiple_properties() {
    let css = parse_and_generate(
        "div { color: blue; background: green; }",
        OutputStyle::Expanded,
    );
    assert!(css.contains("color"), "got: {css}");
    assert!(css.contains("background"), "got: {css}");
}

#[test]
fn important_flag() {
    let css = parse_and_generate(
        "a { color: blue !important; }",
        OutputStyle::Expanded,
    );
    assert!(css.contains("!important"), "should preserve !important flag, got: {css}");
}

#[test]
fn class_selector() {
    let css = parse_and_generate(".foo { color: blue; }", OutputStyle::Expanded);
    assert!(css.contains(".foo"), "should contain class selector, got: {css}");
}

#[test]
fn child_combinator() {
    let css = parse_and_generate("a > b { color: blue; }", OutputStyle::Expanded);
    assert!(css.contains(">"), "should preserve child combinator, got: {css}");
}

#[test]
fn media_atrule() {
    let css = parse_and_generate(
        "@media screen { a { color: blue; } }",
        OutputStyle::Expanded,
    );
    assert!(css.contains("@media"), "should contain @media, got: {css}");
    assert!(css.contains("screen"), "should contain query, got: {css}");
}

#[test]
fn empty_stylesheet() {
    let css = parse_and_generate("", OutputStyle::Expanded);
    // Empty input produces empty output.
    assert!(css.is_empty() || css.trim().is_empty(), "empty input should produce empty output");
}

#[test]
fn expanded_has_newlines() {
    let css = parse_and_generate("a { color: blue; }", OutputStyle::Expanded);
    assert!(css.contains('\n'), "expanded output should have newlines");
    assert!(css.contains('{'), "expanded should have brace on same line as selector");
    assert!(css.contains('}'), "expanded should have closing brace");
}
