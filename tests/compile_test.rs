//! 编译管线端到端测试——从源码到 CSS 的完整验证。
//!
//! 物理隔离：所有编译相关测试集中于此，不使用内联 #[cfg(test)] 模块。
//! 使用 tracing 进行问题追踪。

use sasspile::{compile_expanded, compile_file, init_tracing, OutputStyle};

#[test]
fn test_compile_simple() {
    let css = compile_expanded("a { color: red; }").unwrap();
    assert_eq!(css, "a {\n  color: red;\n}\n");
}

#[test]
fn test_compile_variable() {
    let css = compile_expanded("$w: 10px; a { width: $w; }").unwrap();
    assert!(css.contains("width: 10px"));
}

#[test]
fn test_compile_nested() {
    let css = compile_expanded(".outer { color: red; .inner { color: blue; } }").unwrap();
    assert!(css.contains(".outer"));
    assert!(css.contains(".outer .inner"));
}

#[test]
fn test_compile_amp() {
    let css = compile_expanded(".btn { &:hover { color: red; } }").unwrap();
    assert!(css.contains(".btn:hover"));
}

#[test]
fn test_compile_if() {
    let css = compile_expanded("@if true { a { color: red; } }").unwrap();
    assert!(css.contains("color: red"));
}

#[test]
fn test_compile_for() {
    let css = compile_expanded("@for $i from 1 through 3 { .col-#{$i} { width: $i * 100%; } }")
        .unwrap();
    assert!(css.contains("col-1"));
}

#[test]
fn test_compile_mixin() {
    let css = compile_expanded("@mixin bold { font-weight: bold; } .title { @include bold; }")
        .unwrap();
    assert!(css.contains("font-weight: bold"));
}

#[test]
fn test_compile_content() {
    let css = compile_expanded(
        "@mixin wrapper { .inner { @content; } } @include wrapper { color: red; }",
    )
    .unwrap();
    assert!(css.contains(".inner"));
    assert!(css.contains("color: red"));
}

#[test]
fn test_compile_each_map() {
    let css =
        compile_expanded("@each $key, $val in (a: 1, b: 2) { .#{$key} { width: $val; } }")
            .unwrap();
    assert!(css.contains(".a"));
    assert!(css.contains("width: 1"));
}

#[test]
fn test_compile_math_round() {
    let css = compile_expanded("@use 'sass:math' as math; a { w: math.round(3.7); }").unwrap();
    assert!(css.contains("w: 4"));
}

#[test]
fn test_compile_string_slice() {
    let css =
        compile_expanded("@use 'sass:string' as string; a { s: string.slice('hello', 2, 4); }")
            .unwrap();
    assert!(css.contains("ell"));
}

#[test]
fn test_compile_map_get() {
    let css = compile_expanded("@use 'sass:map' as map; $m: (a: 1); a { v: map.get($m, a); }")
        .unwrap();
    assert!(css.contains("v: 1"));
}

#[test]
fn test_compile_at_root() {
    let css = compile_expanded(".parent { @at-root { .child { color: red; } } }").unwrap();
    assert!(css.contains(".child"));
    assert!(!css.contains(".parent .child"));
}

#[test]
fn test_compile_user_function() {
    let css =
        compile_expanded("@function double($x) { @return $x * 2; } a { w: double(5px); }")
            .unwrap();
    assert!(css.contains("w: 10px"));
}

#[test]
fn test_compile_use_file() {
    let dir = std::env::temp_dir().join("sasspile_test_use");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_config.scss"), "$primary: #ff0000;\n").unwrap();
    let main = dir.join("main.scss");
    std::fs::write(&main, "@use 'config';\na { color: config.$primary; }\n").unwrap();
    let css = compile_file(&main, OutputStyle::Expanded).unwrap();
    assert!(
        css.contains("red") || css.contains("#ff0000"),
        "应该包含 config.$primary 的值: {css}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_compile_use_star() {
    let dir = std::env::temp_dir().join("sasspile_test_star");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_vars.scss"), "$w: 100px;\n").unwrap();
    let main = dir.join("main.scss");
    std::fs::write(&main, "@use 'vars' as *;\na { width: $w; }\n").unwrap();
    let css = compile_file(&main, OutputStyle::Expanded).unwrap();
    assert!(css.contains("100px"), "应该包含 $w 的值: {css}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_compile_extend() {
    let css =
        compile_expanded(".btn { color: blue; } .large { @extend .btn; font-size: 20px; }")
            .unwrap();
    assert!(css.contains(".btn"), "应该包含 .btn: {css}");
    assert!(css.contains(".large"), "应该包含 .large: {css}");
    assert!(css.contains("color: blue"), "应该包含 color: blue: {css}");
    assert!(css.contains("font-size: 20px"), "应该包含 font-size: {css}");
}

#[test]
fn test_compile_extend_placeholder() {
    let css = compile_expanded("%base { color: red; } .child { @extend %base; }").unwrap();
    assert!(!css.contains("%base"), "占位符不应出现: {css}");
    assert!(css.contains(".child"), "应该包含 .child: {css}");
    assert!(css.contains("color: red"), "应该包含 color: red: {css}");
}

#[test]
fn test_compile_hsl() {
    let css = compile_expanded("a { color: hsl(120, 50%, 50%); }").unwrap();
    assert!(css.contains("#"), "应该包含 hex 颜色: {css}");
}

#[test]
fn test_compile_append() {
    let css = compile_expanded("$l: append((1 2), 3); a { v: $l; }").unwrap();
    assert!(css.contains("1"), "应该包含 1: {css}");
    assert!(css.contains("3"), "应该包含 3: {css}");
}

#[test]
fn test_compile_clamp() {
    let css = compile_expanded("a { w: clamp(1, 5, 10); }").unwrap();
    assert!(css.contains("5"), "应该包含 5: {css}");
}

#[test]
fn test_debug_interleaved() {
    init_tracing();
    let css = compile_expanded(".a { b: c; .d { e: f; } }").unwrap();
    tracing::info!("DEBUG OUTPUT:\n{}", css);
    assert!(css.contains(".a .d"), "Missing .a .d: {css}");
}

#[test]
fn test_debug_atfoo() {
    init_tracing();
    let css = compile_expanded("@foo {}").unwrap();
    tracing::info!("@foo OUTPUT: [{}]", css);
    assert!(!css.is_empty(), "Output should not be empty");
    assert!(css.contains("@foo"), "Should contain @foo: [{css}]");
}

#[test]
fn test_debug_minus() {
    init_tracing();
    let result = compile_expanded("a {b: c - d}");
    match &result {
        Ok(css) => tracing::info!("MINUS OUTPUT: [{}]", css),
        Err(e) => tracing::error!("MINUS ERROR: {}", e),
    }
}

#[test]
fn test_debug_bs_close() {
    init_tracing();
    let input = "$m: (c: d);\na {b: map-remove($m, x)}";
    let result = compile_expanded(input);
    match &result {
        Ok(css) => tracing::info!(css = css.as_str(), "MAP OUTPUT"),
        Err(e) => tracing::error!(error = %e, "MAP ERROR"),
    }
}

#[test]
fn test_init_tracing_shows_target() {
    init_tracing();
    tracing::info!(target: "test_target_check", "tracing target visibility test");
}

#[test]
fn test_lib_adjust_color() {
    let css = compile_expanded("a { b: adjust-color(red, $red: -50); }").unwrap();
    assert!(css.contains("#cd0000"), "should contain #cd0000: {css}");
}

#[test]
fn test_cli_compile() {
    let input = "a { color: red; }";
    let css = compile_expanded(input).unwrap();
    assert!(css.contains("color: red"));
}
