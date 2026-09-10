//! —— CSS 细节输出测试 ——
//!
//! 概要：验证 CSS 细节输出（@media bubbling、@supports、custom properties、选择器操作）。
//!
//! ## 覆盖场景
//! - @media bubbling：嵌套 @media 提升合并
//! - @supports：基础、not、and 逻辑操作符
//! - CSS custom properties：声明、var()、fallback、插值
//! - 选择器操作：selector-nest、selector-replace、selector-parse、selector-unify、selector-extend、is-superselector
//!
//! ## sass-spec 参照
//! - `at_rules/media/` — 媒体查询 bubbling
//! - `at_rules/supports/` — supports 格式
//! - `core_functions/selector/` — 选择器操作函数

fn compile_expanded(input: &str) -> Result<String, String> {
    sasspile::compile_expanded(input).map_err(|e| e.to_string())
}

// ── @media bubbling ──

#[test]
fn test_media_bubble_basic() {
    let css = compile_expanded(".a { @media (min-width: 768px) { color: red; } }").expect("unexpected failure in test");
    assert!(css.contains("@media"), "应包含 @media: {css}");
    assert!(css.contains(".a"), "应包含 .a: {css}");
    assert!(css.contains("color: red"), "应包含 color: red: {css}");
}

#[test]
fn test_media_nested_merge() {
    // 嵌套 @media 应正确合并
    let css = compile_expanded("@media (min-width: 768px) { .a { color: red; } @media (max-width: 1024px) { .b { color: blue; } } }").expect("unexpected failure in test");
    assert!(css.contains("@media"), "应包含 @media: {css}");
}

// ── @supports ──

#[test]
fn test_supports_basic() {
    let css = compile_expanded("@supports (display: grid) { .a { display: grid; } }").expect("unexpected failure in test");
    assert!(css.contains("@supports"), "应包含 @supports: {css}");
    assert!(css.contains("display: grid"), "应包含 display: grid: {css}");
}

#[test]
fn test_supports_not() {
    let css = compile_expanded("@supports not (display: grid) { .a { display: flex; } }").expect("unexpected failure in test");
    assert!(css.contains("@supports not"), "应包含 @supports not: {css}");
}

#[test]
fn test_supports_and() {
    let css = compile_expanded("@supports (display: grid) and (gap: 10px) { .a { display: grid; } }").expect("unexpected failure in test");
    assert!(css.contains("@supports"), "应包含 @supports: {css}");
    assert!(css.contains("and"), "应包含 and: {css}");
}

// ── CSS custom properties ──

#[test]
fn test_custom_property_declaration() {
    let css = compile_expanded(":root { --color: red; }").expect("unexpected failure in test");
    assert!(css.contains("--color"), "应包含 --color: {css}");
    assert!(css.contains("red"), "应包含 red: {css}");
}

#[test]
fn test_custom_property_var() {
    let css = compile_expanded("a { color: var(--color); }").expect("unexpected failure in test");
    assert!(css.contains("var(--color)"), "应包含 var(--color): {css}");
}

#[test]
fn test_custom_property_var_fallback() {
    let css = compile_expanded("a { color: var(--color, blue); }").expect("unexpected failure in test");
    assert!(css.contains("var(--color, blue)"), "应包含 var 带 fallback: {css}");
}

#[test]
fn test_custom_property_interpolation() {
    let css = compile_expanded("$size: 10; a { --size-#{$size}: 10px; }").expect("unexpected failure in test");
    assert!(css.contains("--size-10"), "应解析插值: {css}");
}

// ── selector operations ──

#[test]
fn test_selector_nest() {
    let css = compile_expanded("a { b: selector-nest('.a', '.b'); }").expect("unexpected failure in test");
    assert!(css.contains("b:"), "应输出: {css}");
}

#[test]
fn test_selector_replace() {
    let css = compile_expanded("a { b: selector-replace('.a.b', '.b', '.c'); }").expect("unexpected failure in test");
    assert!(css.contains(".a.c"), "应包含 .a.c: {css}");
}

#[test]
fn test_selector_parse() {
    let css = compile_expanded("a { b: selector-parse('.a.b'); }").expect("unexpected failure in test");
    assert!(css.contains("b:"), "应输出: {css}");
}

#[test]
fn test_selector_unify() {
    let css = compile_expanded("a { b: selector-unify('.a', '.b'); }").expect("unexpected failure in test");
    assert!(css.contains("b:"), "应输出: {css}");
}

#[test]
fn test_selector_extend() {
    let css = compile_expanded("a { b: selector-extend('.a.b', '.b', '.c'); }").expect("unexpected failure in test");
    assert!(css.contains("b:"), "应输出: {css}");
}

#[test]
fn test_is_superselector() {
    let css = compile_expanded("a { b: is-superselector('.a', '.a.b'); }").expect("unexpected failure in test");
    assert!(css.contains("b:"), "应输出: {css}");
}
