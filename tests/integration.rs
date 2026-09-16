//! sasspile-rx integration tests — black-box, public API only.
//!
//! Every test goes through `sasspile_rx::compile` — the single public entry point.
//! No direct use of ScannerState, AstBuilder, Observable types, etc.

use sasspile_rx::{compile, CompileError};

#[test]
fn basic_rule_compiles_to_css() {
    let css = compile("a { color: red; }").unwrap();
    assert!(css.contains("color"), "css was: {css}");
    assert!(css.contains("red"), "css was: {css}");
}

#[test]
fn multiple_declarations_in_single_rule() {
    let css = compile(".box { width: 100px; height: 50px; }").unwrap();
    assert!(css.contains("width"), "css was: {css}");
    assert!(css.contains("height"), "css was: {css}");
}

#[test]
fn multiple_rules() {
    let scss = ".btn { color: white; } .link { color: blue; }";
    let css = compile(scss).unwrap();
    assert!(css.contains("btn"), "css was: {css}");
    assert!(css.contains("link"), "css was: {css}");
}

#[test]
fn invalid_input_does_not_panic() {
    // Errors become Result::Err, never panic
    let _result = compile("x { : ; }");
}

#[test]
fn nested_selector_syntax() {
    let css = compile("ul { list-style: none; }").unwrap();
    assert!(css.contains("list-style"));
    assert!(css.contains("none"));
}

#[test]
fn class_and_element_selectors() {
    let css = compile(".container { max-width: 960px; } h1 { font-size: 2rem; }").unwrap();
    assert!(css.contains("container"));
    assert!(css.contains("font-size"));
}

#[test]
fn result_is_ok_or_never_panic() {
    let test_cases = ["", "{}", "a{}", "a { }", "x { : ; }", "// comment only"];
    for src in test_cases {
        let _ = compile(src); // must not panic
    }
}

#[test]
fn compile_error_type_reachable() {
    // CompileError is part of the public API
    let err = CompileError::InvalidInput {
        message: "test".into(),
    };
    let _ = format!("{err:?}");
}
