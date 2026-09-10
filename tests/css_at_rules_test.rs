//! —— CSS at-rules 编译测试 ——
//!
//! 概要：验证 CSS at-rules 的完整编译能力。
//!
//! ## 覆盖场景
//! - @keyframes：基础、百分比、列表选择器、vendor 前缀、bubble、插值名称
//! - @font-face：字体声明
//! - @page：基础、伪类（:first）
//! - @charset / @namespace
//! - @layer：声明形式、嵌套 block
//! - @container：条件查询
//! - @supports：嵌套
//!
//! ## sass-spec 参照
//! - `at_rules/keyframes/` — 动画关键帧
//! - `at_rules/font_face/` — 字体声明
//! - `at_rules/page/` — 页规则
//! - `at_rules/layer/` — 层规则
//! - `at_rules/container/` — 容器查询

fn compile_expanded(input: &str) -> Result<String, String> {
    sasspile::compile_expanded(input).map_err(|e| e.to_string())
}

#[test]
fn test_keyframes_basic() {
    let css = compile_expanded("@keyframes fade { from { opacity: 0; } to { opacity: 1; } }").expect("unexpected failure in test");
    assert!(css.contains("@keyframes fade"), "应输出 @keyframes: {css}");
    assert!(css.contains("from"), "应包含 from: {css}");
    assert!(css.contains("to"), "应包含 to: {css}");
}

#[test]
fn test_keyframes_percentage() {
    let css = compile_expanded("@keyframes a { 0% { opacity: 0; } 50% { opacity: 0.5; } 100% { opacity: 1; } }").expect("unexpected failure in test");
    assert!(css.contains("@keyframes a"), "应输出 @keyframes: {css}");
    assert!(css.contains("0%"), "应包含 0%: {css}");
    assert!(css.contains("50%"), "应包含 50%: {css}");
    assert!(css.contains("100%"), "应包含 100%: {css}");
}

#[test]
fn test_keyframes_list_selector() {
    let css = compile_expanded("@keyframes a { from, 50%, to { opacity: 1; } }").expect("unexpected failure in test");
    assert!(css.contains("@keyframes a"), "应输出 @keyframes: {css}");
    assert!(css.contains("from, 50%, to"), "应包含列表选择器: {css}");
}

#[test]
fn test_keyframes_vendor_prefix() {
    let css = compile_expanded("@-webkit-keyframes slide { from { left: 0; } to { left: 100px; } }").expect("unexpected failure in test");
    assert!(css.contains("@-webkit-keyframes slide"), "应输出 @-webkit-keyframes: {css}");
}

#[test]
fn test_keyframes_bubble() {
    let css = compile_expanded("a { b: c; @keyframes d { to { e: f; } } }").expect("unexpected failure in test");
    assert!(css.contains("@keyframes d"), "应输出 @keyframes: {css}");
    assert!(css.contains("a {"), "应包含 a 规则: {css}");
    assert!(css.contains("b: c"), "应包含 b: c: {css}");
}

#[test]
fn test_keyframes_interpolated_name() {
    let css = compile_expanded("$name: fade; @keyframes #{$name} { from { opacity: 0; } }").expect("unexpected failure in test");
    assert!(css.contains("@keyframes fade"), "应解析插值名称: {css}");
}

#[test]
fn test_font_face() {
    let css = compile_expanded("@font-face { font-family: 'MyFont'; src: url('font.woff'); }").expect("unexpected failure in test");
    assert!(css.contains("@font-face"), "应输出 @font-face: {css}");
    assert!(css.contains("font-family:"), "应包含 font-family: {css}");
    assert!(css.contains("MyFont"), "应包含 MyFont: {css}");
    assert!(css.contains("src: url"), "应包含 src: {css}");
}

#[test]
fn test_page() {
    let css = compile_expanded("@page { margin: 1cm; }").expect("unexpected failure in test");
    assert!(css.contains("@page"), "应输出 @page: {css}");
    assert!(css.contains("margin: 1cm"), "应包含 margin: {css}");
}

#[test]
fn test_page_pseudo() {
    let css = compile_expanded("@page :first { margin: 2cm; }").expect("unexpected failure in test");
    assert!(css.contains("@page :first"), "应输出 @page :first: {css}");
}

#[test]
fn test_charset() {
    let css = compile_expanded("@charset \"UTF-8\";").expect("unexpected failure in test");
    assert!(css.contains("@charset \"UTF-8\""), "应输出 @charset: {css}");
}

#[test]
fn test_namespace() {
    let css = compile_expanded("@namespace svg \"http://www.w3.org/2000/svg\";").expect("unexpected failure in test");
    assert!(css.contains("@namespace svg"), "应输出 @namespace: {css}");
    assert!(css.contains("http://www.w3.org/2000/svg"), "应包含 URL: {css}");
}

#[test]
fn test_layer_statement() {
    let css = compile_expanded("@layer base, utilities;").expect("unexpected failure in test");
    assert!(css.contains("@layer base, utilities"), "应输出 @layer 声明: {css}");
}

#[test]
fn test_layer_block() {
    let css = compile_expanded("@layer base { .foo { color: red; } }").expect("unexpected failure in test");
    assert!(css.contains("@layer base"), "应输出 @layer: {css}");
    assert!(css.contains(".foo"), "应包含 .foo: {css}");
}

#[test]
fn test_layer_nested() {
    let css = compile_expanded("@layer framework { @layer base { .foo { color: red; } } }").expect("unexpected failure in test");
    assert!(css.contains("@layer framework"), "应输出外层 @layer: {css}");
}

#[test]
fn test_container() {
    let css = compile_expanded("@container (min-width: 700px) { .item { color: red; } }").expect("unexpected failure in test");
    assert!(css.contains("@container"), "应输出 @container: {css}");
    assert!(css.contains("min-width: 700px"), "应包含条件: {css}");
}

#[test]
fn test_supports_nested() {
    let css = compile_expanded("@supports (display: grid) { .a { display: grid; } }").expect("unexpected failure in test");
    assert!(css.contains("@supports"), "应输出 @supports: {css}");
    assert!(css.contains("display: grid"), "应包含 display: grid: {css}");
}
