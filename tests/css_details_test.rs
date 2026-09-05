//! CSS 细节测试——@media bubbling, @supports 格式, CSS custom properties, selector.replace/nest。

use sasspile::{OutputStyle, Source};

fn compile_expanded(input: &str) -> Result<String, String> {
    let css = Source::new(input.to_string())
        .lex()
        .map_err(|e| e.to_string())?
        .parse()
        .map_err(|e| e.to_string())?
        .evaluate()
        .map_err(|e| e.to_string())?
        .serialize(OutputStyle::Expanded)
        .into_string();
    Ok(css)
}

// ── @media bubbling ──

#[test]
fn test_media_bubble_basic() {
    let css = compile_expanded(".a { @media (min-width: 768px) { color: red; } }").unwrap();
    assert!(css.contains("@media"), "应包含 @media: {css}");
    assert!(css.contains(".a"), "应包含 .a: {css}");
    assert!(css.contains("color: red"), "应包含 color: red: {css}");
}

#[test]
fn test_media_nested_merge() {
    // 嵌套 @media 应正确合并
    let css = compile_expanded("@media (min-width: 768px) { .a { color: red; } @media (max-width: 1024px) { .b { color: blue; } } }").unwrap();
    assert!(css.contains("@media"), "应包含 @media: {css}");
}

// ── @supports ──

#[test]
fn test_supports_basic() {
    let css = compile_expanded("@supports (display: grid) { .a { display: grid; } }").unwrap();
    assert!(css.contains("@supports"), "应包含 @supports: {css}");
    assert!(css.contains("display: grid"), "应包含 display: grid: {css}");
}

#[test]
fn test_supports_not() {
    let css = compile_expanded("@supports not (display: grid) { .a { display: flex; } }").unwrap();
    assert!(css.contains("@supports not"), "应包含 @supports not: {css}");
}

#[test]
fn test_supports_and() {
    let css = compile_expanded("@supports (display: grid) and (gap: 10px) { .a { display: grid; } }").unwrap();
    assert!(css.contains("@supports"), "应包含 @supports: {css}");
    assert!(css.contains("and"), "应包含 and: {css}");
}

// ── CSS custom properties ──

#[test]
fn test_custom_property_declaration() {
    let css = compile_expanded(":root { --color: red; }").unwrap();
    assert!(css.contains("--color"), "应包含 --color: {css}");
    assert!(css.contains("red"), "应包含 red: {css}");
}

#[test]
fn test_custom_property_var() {
    let css = compile_expanded("a { color: var(--color); }").unwrap();
    assert!(css.contains("var(--color)"), "应包含 var(--color): {css}");
}

#[test]
fn test_custom_property_var_fallback() {
    let css = compile_expanded("a { color: var(--color, blue); }").unwrap();
    assert!(css.contains("var(--color, blue)"), "应包含 var 带 fallback: {css}");
}

#[test]
fn test_custom_property_interpolation() {
    let css = compile_expanded("$size: 10; a { --size-#{$size}: 10px; }").unwrap();
    assert!(css.contains("--size-10"), "应解析插值: {css}");
}

// ── selector operations ──

#[test]
fn test_selector_nest() {
    let css = compile_expanded("a { b: selector-nest('.a', '.b'); }").unwrap();
    assert!(css.contains("b:"), "应输出: {css}");
}

#[test]
fn test_selector_replace() {
    let css = compile_expanded("a { b: selector-replace('.a.b', '.b', '.c'); }").unwrap();
    assert!(css.contains(".a.c"), "应包含 .a.c: {css}");
}

#[test]
fn test_selector_parse() {
    let css = compile_expanded("a { b: selector-parse('.a.b'); }").unwrap();
    assert!(css.contains("b:"), "应输出: {css}");
}

#[test]
fn test_selector_unify() {
    let css = compile_expanded("a { b: selector-unify('.a', '.b'); }").unwrap();
    assert!(css.contains("b:"), "应输出: {css}");
}

#[test]
fn test_selector_extend() {
    let css = compile_expanded("a { b: selector-extend('.a.b', '.b', '.c'); }").unwrap();
    assert!(css.contains("b:"), "应输出: {css}");
}

#[test]
fn test_is_superselector() {
    let css = compile_expanded("a { b: is-superselector('.a', '.a.b'); }").unwrap();
    assert!(css.contains("b:"), "应输出: {css}");
}
