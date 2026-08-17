//! Meta builtins tests — tests for sass:meta functions.

use sasspile::compile;

fn compile_src(src: &str) -> String {
    compile(src).expect("compilation should succeed")
}

#[test]
fn test_type_of_number() {
    let css = compile_src("a { c: type-of(42); }");
    assert!(css.contains("number"), "type-of(42) should be number, got: {}", css);
}

#[test]
fn test_type_of_string() {
    let css = compile_src("a { c: type-of(\"hello\"); }");
    assert!(css.contains("string"), "type-of(\"hello\") should be string, got: {}", css);
}

#[test]
fn test_type_of_bool() {
    let css = compile_src("a { c: type-of(true); }");
    assert!(css.contains("bool"), "type-of(true) should be bool, got: {}", css);
}

#[test]
fn test_type_of_null() {
    let css = compile_src("a { c: type-of(null); }");
    assert!(css.contains("null"), "type-of(null) should be null, got: {}", css);
}

#[test]
fn test_type_of_color() {
    let css = compile_src("a { c: type-of(red); }");
    assert!(css.contains("color"), "type-of(red) should be color, got: {}", css);
}

#[test]
fn test_inspect_number() {
    let css = compile_src("a { c: inspect(42); }");
    assert!(css.contains("42"), "inspect(42) should contain 42, got: {}", css);
}

#[test]
fn test_inspect_list() {
    let css = compile_src("a { c: inspect((1, 2, 3)); }");
    assert!(css.contains("1") && css.contains("2") && css.contains("3"), "inspect should contain all list items, got: {}", css);
}

#[test]
fn test_feature_exists() {
    let css = compile_src("a { c: feature-exists(global-variable-shadowing); }");
    assert!(css.contains("true"), "feature-exists should be true, got: {}", css);
}

#[test]
fn test_variable_exists() {
    let css = compile_src("$x: 1; a { c: variable-exists(x); }");
    assert!(css.contains("true"), "variable-exists should be true, got: {}", css);
}
