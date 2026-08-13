//! 选择器函数测试——selector-is-superselector/unify/extend。
//!
//! 物理隔离：所有选择器函数相关测试集中于此。
//! 使用 tracing 进行问题追踪。

use sasspile::{compile_expanded, init_tracing};

// —— selector-is-superselector ——

#[test]
fn test_is_superselector_simple() {
    init_tracing();
    // .b 是 .a 的 superselector（.a 匹配的元素 .b 都能匹配）
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-is-superselector('.a', '.a.b'); }",
    )
    .unwrap();
    assert!(css.contains("b: true"), "expected true, got: {css}");
}

#[test]
fn test_is_superselector_false() {
    init_tracing();
    // .a.b 不是 .a 的 superselector
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-is-superselector('.a.b', '.a'); }",
    )
    .unwrap();
    assert!(css.contains("b: false"), "expected false, got: {css}");
}

#[test]
fn test_is_superselector_element() {
    init_tracing();
    // div 是 div 的 superselector
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-is-superselector('div', 'div'); }",
    )
    .unwrap();
    assert!(css.contains("b: true"), "expected true, got: {css}");
}

#[test]
fn test_is_superselector_wildcard() {
    init_tracing();
    // * 是 div 的 superselector
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-is-superselector('*', 'div'); }",
    )
    .unwrap();
    assert!(css.contains("b: true"), "expected true, got: {css}");
}

#[test]
fn test_is_superselector_with_id() {
    init_tracing();
    // #foo 是 #foo 的 superselector
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-is-superselector('#foo', '#foo'); }",
    )
    .unwrap();
    assert!(css.contains("b: true"), "expected true, got: {css}");
}

#[test]
fn test_is_superselector_complex() {
    init_tracing();
    // .a .b 是 .a .b 的 superselector
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-is-superselector('.a .b', '.a .b'); }",
    )
    .unwrap();
    assert!(css.contains("b: true"), "expected true, got: {css}");
}

// —— selector-unify ——

#[test]
fn test_unify_same_class() {
    init_tracing();
    // .a 与 .a 统一为 .a
    let css = compile_expanded("@use 'sass:selector'; a { b: selector-unify('.a', '.a'); }")
        .unwrap();
    assert!(css.contains(".a"), "expected .a, got: {css}");
}

#[test]
fn test_unify_different_classes() {
    init_tracing();
    // .a 与 .b 统一为 .a.b
    let css = compile_expanded("@use 'sass:selector'; a { b: selector-unify('.a', '.b'); }")
        .unwrap();
    assert!(css.contains(".a.b") || css.contains(".b.a"), "expected unified, got: {css}");
}

#[test]
fn test_unify_with_element() {
    init_tracing();
    // div 与 .a 统一为 div.a
    let css = compile_expanded("@use 'sass:selector'; a { b: selector-unify('div', '.a'); }")
        .unwrap();
    assert!(css.contains("div.a"), "expected div.a, got: {css}");
}

#[test]
fn test_unify_wildcard() {
    init_tracing();
    // * 与 .a 统一为 .a
    let css = compile_expanded("@use 'sass:selector'; a { b: selector-unify('*', '.a'); }")
        .unwrap();
    assert!(css.contains(".a"), "expected .a, got: {css}");
}

#[test]
fn test_unify_conflict_returns_null() {
    init_tracing();
    // div 与 span 无法统一，返回 null（不输出）
    let css = compile_expanded("@use 'sass:selector'; a { b: selector-unify('div', 'span'); }")
        .unwrap();
    // null 值声明不输出
    assert!(!css.contains("b:"), "expected null (no output), got: {css}");
}

// —— selector-extend ——

#[test]
fn test_extend_simple() {
    init_tracing();
    // 将 .a 中的 .b 替换为 .c
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-extend('.a .b', '.b', '.c'); }",
    )
    .unwrap();
    // 预期输出应包含 .a .c 或类似形式
    assert!(css.contains(".c"), "expected .c in output, got: {css}");
}

#[test]
fn test_extend_no_match() {
    init_tracing();
    // .a 中不包含 .x，不替换
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-extend('.a', '.x', '.y'); }",
    )
    .unwrap();
    assert!(css.contains(".a"), "expected .a in output, got: {css}");
}

#[test]
fn test_extend_with_element() {
    init_tracing();
    // 将 div 中的 span 替换为 p
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-extend('div span', 'span', 'p'); }",
    )
    .unwrap();
    assert!(css.contains("p"), "expected p in output, got: {css}");
}

// —— selector-append ——

#[test]
fn test_selector_append() {
    init_tracing();
    let css = compile_expanded("@use 'sass:selector'; a { b: selector-append('.a', '.b'); }")
        .unwrap();
    assert!(css.contains(".a.b"), "expected .a.b, got: {css}");
}

#[test]
fn test_selector_append_multiple() {
    init_tracing();
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-append('.a', '.b', '.c'); }",
    )
    .unwrap();
    assert!(css.contains(".a.b.c"), "expected .a.b.c, got: {css}");
}

// —— selector-nest ——

#[test]
fn test_selector_nest() {
    init_tracing();
    let css = compile_expanded("@use 'sass:selector'; a { b: selector-nest('.a', '.b'); }")
        .unwrap();
    assert!(css.contains(".a .b"), "expected .a .b, got: {css}");
}

#[test]
fn test_selector_nest_multiple() {
    init_tracing();
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-nest('.a', '.b', '.c'); }",
    )
    .unwrap();
    assert!(css.contains(".a .b .c"), "expected .a .b .c, got: {css}");
}

// —— selector-parse ——

#[test]
fn test_selector_parse_single() {
    init_tracing();
    let css = compile_expanded("@use 'sass:selector'; a { b: selector-parse('.a'); }")
        .unwrap();
    assert!(css.contains(".a"), "expected .a, got: {css}");
}

#[test]
fn test_selector_parse_list() {
    init_tracing();
    let css = compile_expanded("@use 'sass:selector'; a { b: selector-parse('.a, .b'); }")
        .unwrap();
    assert!(css.contains(".a") && css.contains(".b"), "expected .a and .b, got: {css}");
}

// —— 边界条件 ——

#[test]
fn test_is_superselector_empty_sub() {
    init_tracing();
    // 空选择器是任何选择器的 sub
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-is-superselector('.a', ''); }",
    )
    .unwrap();
    // 空 sub 应该返回 true
    assert!(css.contains("b: true"), "expected true, got: {css}");
}

#[test]
fn test_unify_with_pseudo() {
    init_tracing();
    // .a 与 :hover 统一
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-unify('.a', ':hover'); }",
    )
    .unwrap();
    assert!(css.contains(".a") && css.contains(":hover"), "expected .a:hover, got: {css}");
}

#[test]
fn test_extend_complex() {
    init_tracing();
    // 复杂选择器扩展
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-extend('.a .b .c', '.b', '.x'); }",
    )
    .unwrap();
    assert!(css.contains(".x"), "expected .x in output, got: {css}");
}

// —— selector-replace ——

#[test]
fn test_selector_replace() {
    init_tracing();
    let css = compile_expanded(
        "@use 'sass:selector'; a { b: selector-replace('.a.b', '.b', '.c'); }",
    )
    .unwrap();
    assert!(css.contains(".a.c"), "expected .a.c, got: {css}");
}
