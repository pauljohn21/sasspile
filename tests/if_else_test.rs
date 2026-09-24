//! Phase D.1 — @if / @else 控制流

use sasspile::compile;

#[test]
fn if_true_literal() {
    let scss = "@if true {\n  .a { color: red; }\n}";
    let css = compile(scss);
    assert!(css.contains(".a"), "got: {css}");
    assert!(css.contains("color: red"), "got: {css}");
}

#[test]
fn if_false_literal_else_taken() {
    let scss = "@if false {\n  .a { color: red; }\n} @else {\n  .b { color: blue; }\n}";
    let css = compile(scss);
    assert!(!css.contains(".a"), "a should be skipped: {css}");
    assert!(css.contains(".b"), "b should be emitted: {css}");
    assert!(css.contains("color: blue"), "got: {css}");
}

#[test]
fn if_true_else_skipped() {
    let scss = "@if true {\n  .a { color: red; }\n} @else {\n  .b { color: blue; }\n}";
    let css = compile(scss);
    assert!(css.contains(".a"), "a should be emitted: {css}");
    assert!(!css.contains(".b"), "b should be skipped: {css}");
}

#[test]
fn if_else_if_ladder() {
    let scss = "@if false {\n  .a { x: 1; }\n} @else if true {\n  .b { x: 2; }\n} @else {\n  .c { x: 3; }\n}";
    let css = compile(scss);
    assert!(!css.contains(".a"), "got: {css}");
    assert!(css.contains(".b"), "got: {css}");
    assert!(!css.contains(".c"), "got: {css}");
}

#[test]
fn if_with_variable_truthy() {
    let scss = "$on: true;\n@if $on {\n  .a { color: green; }\n} @else {\n  .b { color: red; }\n}";
    let css = compile(scss);
    assert!(css.contains(".a"), "got: {css}");
    assert!(!css.contains(".b"), "got: {css}");
}

#[test]
fn if_with_variable_falsy() {
    let scss = "$on: false;\n@if $on {\n  .a { color: green; }\n} @else {\n  .b { color: red; }\n}";
    let css = compile(scss);
    assert!(!css.contains(".a"), "got: {css}");
    assert!(css.contains(".b"), "got: {css}");
}

#[test]
fn if_condition_equality() {
    let scss = "$theme: dark;\n@if $theme == dark {\n  .a { bg: black; }\n} @else {\n  .b { bg: white; }\n}";
    let css = compile(scss);
    assert!(css.contains(".a"), "got: {css}");
    assert!(!css.contains(".b"), "got: {css}");
}

#[test]
fn if_condition_negation_with_not() {
    let scss = "@if not false {\n  .a { color: red; }\n}";
    let css = compile(scss);
    assert!(css.contains(".a"), "got: {css}");
}
