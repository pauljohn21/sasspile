//! CSS at-rules 功能测试——@keyframes, @font-face, @page, @charset, @namespace, @layer, @container。

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

#[test]
fn test_keyframes_basic() {
    let css = compile_expanded("@keyframes fade { from { opacity: 0; } to { opacity: 1; } }").unwrap();
    assert!(css.contains("@keyframes fade"), "应输出 @keyframes: {css}");
    assert!(css.contains("from"), "应包含 from: {css}");
    assert!(css.contains("to"), "应包含 to: {css}");
}

#[test]
fn test_keyframes_percentage() {
    let css = compile_expanded("@keyframes a { 0% { opacity: 0; } 50% { opacity: 0.5; } 100% { opacity: 1; } }").unwrap();
    assert!(css.contains("@keyframes a"), "应输出 @keyframes: {css}");
    assert!(css.contains("0%"), "应包含 0%: {css}");
    assert!(css.contains("50%"), "应包含 50%: {css}");
    assert!(css.contains("100%"), "应包含 100%: {css}");
}

#[test]
fn test_keyframes_list_selector() {
    let css = compile_expanded("@keyframes a { from, 50%, to { opacity: 1; } }").unwrap();
    assert!(css.contains("@keyframes a"), "应输出 @keyframes: {css}");
    assert!(css.contains("from, 50%, to"), "应包含列表选择器: {css}");
}

#[test]
fn test_keyframes_vendor_prefix() {
    let css = compile_expanded("@-webkit-keyframes slide { from { left: 0; } to { left: 100px; } }").unwrap();
    assert!(css.contains("@-webkit-keyframes slide"), "应输出 @-webkit-keyframes: {css}");
}

#[test]
fn test_keyframes_bubble() {
    let css = compile_expanded("a { b: c; @keyframes d { to { e: f; } } }").unwrap();
    assert!(css.contains("@keyframes d"), "应输出 @keyframes: {css}");
    assert!(css.contains("a {"), "应包含 a 规则: {css}");
    assert!(css.contains("b: c"), "应包含 b: c: {css}");
}

#[test]
fn test_keyframes_interpolated_name() {
    let css = compile_expanded("$name: fade; @keyframes #{$name} { from { opacity: 0; } }").unwrap();
    assert!(css.contains("@keyframes fade"), "应解析插值名称: {css}");
}

#[test]
fn test_font_face() {
    let css = compile_expanded("@font-face { font-family: 'MyFont'; src: url('font.woff'); }").unwrap();
    assert!(css.contains("@font-face"), "应输出 @font-face: {css}");
    assert!(css.contains("font-family:"), "应包含 font-family: {css}");
    assert!(css.contains("MyFont"), "应包含 MyFont: {css}");
    assert!(css.contains("src: url"), "应包含 src: {css}");
}

#[test]
fn test_page() {
    let css = compile_expanded("@page { margin: 1cm; }").unwrap();
    assert!(css.contains("@page"), "应输出 @page: {css}");
    assert!(css.contains("margin: 1cm"), "应包含 margin: {css}");
}

#[test]
fn test_page_pseudo() {
    let css = compile_expanded("@page :first { margin: 2cm; }").unwrap();
    assert!(css.contains("@page :first"), "应输出 @page :first: {css}");
}

#[test]
fn test_charset() {
    let css = compile_expanded("@charset \"UTF-8\";").unwrap();
    assert!(css.contains("@charset \"UTF-8\""), "应输出 @charset: {css}");
}

#[test]
fn test_namespace() {
    let css = compile_expanded("@namespace svg \"http://www.w3.org/2000/svg\";").unwrap();
    assert!(css.contains("@namespace svg"), "应输出 @namespace: {css}");
    assert!(css.contains("http://www.w3.org/2000/svg"), "应包含 URL: {css}");
}

#[test]
fn test_layer_statement() {
    let css = compile_expanded("@layer base, utilities;").unwrap();
    assert!(css.contains("@layer base, utilities"), "应输出 @layer 声明: {css}");
}

#[test]
fn test_layer_block() {
    let css = compile_expanded("@layer base { .foo { color: red; } }").unwrap();
    assert!(css.contains("@layer base"), "应输出 @layer: {css}");
    assert!(css.contains(".foo"), "应包含 .foo: {css}");
}

#[test]
fn test_layer_nested() {
    let css = compile_expanded("@layer framework { @layer base { .foo { color: red; } } }").unwrap();
    assert!(css.contains("@layer framework"), "应输出外层 @layer: {css}");
}

#[test]
fn test_container() {
    let css = compile_expanded("@container (min-width: 700px) { .item { color: red; } }").unwrap();
    assert!(css.contains("@container"), "应输出 @container: {css}");
    assert!(css.contains("min-width: 700px"), "应包含条件: {css}");
}

#[test]
fn test_supports_nested() {
    let css = compile_expanded("@supports (display: grid) { .a { display: grid; } }").unwrap();
    assert!(css.contains("@supports"), "应输出 @supports: {css}");
    assert!(css.contains("display: grid"), "应包含 display: grid: {css}");
}
