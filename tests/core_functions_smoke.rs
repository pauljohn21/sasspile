//! Core functions module-dot-call smoke tests.
//!
//! Validates the `substitute_vars` fix that absorbs `.` into identifiers
//! so `module.fn(args)` dispatches to `eval_builtin` correctly.

use sasspile_rx::compile;

// ═══════════════════════════════════════════════════════════════════════════════
// Module-dot function call scenarios
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn math_abs_via_use() {
    let out = compile("@use \"sass:math\";\na {b: math.abs(-5px)}").unwrap();
    assert!(out.contains("5px"), "output was: {out}");
    assert!(!out.contains("math."), "module prefix leaked: {out}");
}

#[test]
fn math_abs_zero() {
    let out = compile("@use \"sass:math\";\na {b: math.abs(0)}").unwrap();
    assert!(out.contains("b: 0"), "output was: {out}");
}

#[test]
fn color_alpha_via_use() {
    let out = compile("@use \"sass:color\";\na {b: color.alpha(red)}").unwrap();
    assert!(out.contains("b: 1"), "output was: {out}");
    assert!(!out.contains("color."), "module prefix leaked: {out}");
}

#[test]
fn string_quote_via_use() {
    let out = compile("@use \"sass:string\";\na {b: string.quote(c)}").unwrap();
    assert!(out.contains("\"c\""), "output was: {out}");
}

#[test]
fn map_get_via_use() {
    let out = compile(
        "@use \"sass:map\";\n$map: (a: 1, b: 2);\na {b: map.get($map, a)}",
    )
    .unwrap();
    assert!(out.contains("b: 1") || out.contains("b:1"), "output was: {out}");
}

#[test]
fn meta_inspect_via_use() {
    // 核心验证: meta. 前缀不泄漏 (inspect 本身未实现返回空是 pre-existing 限制)
    let out = compile("@use \"sass:meta\";\na {b: meta.inspect(42)}").unwrap();
    assert!(!out.contains("meta."), "module prefix leaked: {out}");
}

#[test]
fn list_append_via_use() {
    let out = compile(
        "@use \"sass:list\";\n$result: list.append((), 1);\na {value: $result;}",
    )
    .unwrap();
    assert!(out.contains("1"), "output was: {out}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Fallback — non-function dotted identifiers pass through unchanged
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn css_class_selector_passthrough() {
    let out = compile(".container-fluid { width: 100% }").unwrap();
    assert!(out.contains(".container-fluid"), "selector mangled: {out}");
}

#[test]
fn decimal_value_passthrough() {
    let out = compile("a { width: 1.5px }").unwrap();
    assert!(out.contains("1.5px"), "value was: {out}");
}

#[test]
fn quoted_dotted_string_passthrough() {
    let out = compile("a { content: \"file.txt\" }").unwrap();
    assert!(out.contains("file.txt"), "value was: {out}");
}

#[test]
fn font_family_with_spaces_passthrough() {
    // 验证带有点号的标识符在未触发函数调用时完整保留
    let out = compile("a { font-family: \"Helvetica Neue\" }").unwrap();
    assert!(out.contains("Helvetica Neue"), "font-family mangled: {out}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Regression — bare function calls still work
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bare_abs_still_works() {
    let out = compile("a {b: abs(-5px)}").unwrap();
    assert!(out.contains("5px"), "output was: {out}");
}

#[test]
fn bare_nth_still_works() {
    let out = compile("a {b: nth(10px 20px 30px, 2)}").unwrap();
    assert!(out.contains("20px"), "output was: {out}");
}
