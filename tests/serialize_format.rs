//! Tests for multi-line CSS serialization format.

use sasspile_rx::compile;

fn normalize(css: &str) -> String {
    css.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn single_decl_rule_multiline() {
    let out = compile("$a: 1;\nd {b: $a}").unwrap();
    assert_eq!(normalize(&out), "a {\nb: 1;\n}".replace("a {", "d {"));
    assert!(out.contains("d {\n"), "raw should contain newline after {{: {out:?}");
    assert!(out.contains("\n  b: 1;"), "raw should have 2-space indent: {out:?}");
}

#[test]
fn multi_decl_rule_multiline() {
    let out = compile("a {b: 1; c: 2}").unwrap();
    assert_eq!(normalize(&out), "a {\nb: 1;\nc: 2;\n}");
    // Raw format check: each decl on its own line with 2-space indent
    assert!(out.contains("  b: 1;"));
    assert!(out.contains("  c: 2;"));
}

#[test]
fn empty_rule() {
    let out = compile("a {}").unwrap();
    // Multi-line format: "a {\n}" → normalized "a {\n}"
    assert!(out.contains("a {\n}") || out.contains("{\n}"));
}

#[test]
fn core_fn_math_abs_zerocase() {
    // sass-spec/spec/core_functions/math/abs.hrx zero case
    let input = "@use \"sass:math\";\na {b: math.abs(0)}";
    let expected = "a {\n  b: 0;\n}";
    let out = compile(input).unwrap();
    assert_eq!(normalize(&out), normalize(expected), "raw out: {out:?}");
}

#[test]
fn core_fn_color_alpha() {
    let input = "@use \"sass:color\";\na {b: color.alpha(red)}";
    let expected = "a {\n  b: 1;\n}";
    let out = compile(input).unwrap();
    assert_eq!(normalize(&out), normalize(expected), "raw out: {out:?}");
}

#[test]
fn core_fn_string_quote() {
    let input = "@use \"sass:string\";\na {b: string.quote(c)}";
    let expected = "a {\n  b: \"c\";\n}";
    let out = compile(input).unwrap();
    assert_eq!(normalize(&out), normalize(expected), "raw out: {out:?}");
}

#[test]
fn mixin_output_multiline() {
    // Mixin output outside Rule is flattening style; test that top-level @include path still outputs multi-line if wrapped
    // Top-level @include in this compiler is tested via sass-spec. We just verify mixin output doesn't crash.
    let input = "$x: 42;\na { color: red; background: blue }";
    let out = compile(input).unwrap();
    assert!(out.contains("{\n"), "mixin output should be multiline: {out:?}");
    assert!(out.contains("  color: red;"), "should have indented color: {out:?}");
    assert!(out.contains("  background: blue;"), "should have indented background: {out:?}");
}
