//! CSS Color Level 4 色彩空间序列化测试。
//!
//! 从 compile_test.rs 抽出 14 个色彩空间专项测试，保持单文件 ≤ 500 行。

use sasspile::compile_expanded;

// ─── Lab 色彩空间 ──────────────────────────────────────────────────────────

#[test]
fn test_compile_color_lab_basic() {
    let css = compile_expanded("a { color: lab(50% 20 30); }").expect("unexpected failure in test");
    assert!(css.contains("lab(50% 20 30)"), "应输出 lab(50% 20 30): {css}");
}

#[test]
fn test_compile_color_lab_with_alpha() {
    let css = compile_expanded("a { color: lab(50% 20 30 / 0.5); }").expect("unexpected failure in test");
    assert!(css.contains("lab(50% 20 30 / 0.5)"), "应输出 lab(50% 20 30 / 0.5): {css}");
}

#[test]
fn test_compile_color_lab_negative_values() {
    let css = compile_expanded("a { color: lab(75% -160 100); }").expect("unexpected failure in test");
    assert!(css.contains("lab(75% -160 100)"), "应输出 lab(75% -160 100): {css}");
}

// ─── Lch 色彩空间 ──────────────────────────────────────────────────────────

#[test]
fn test_compile_color_lch_basic() {
    let css = compile_expanded("a { color: lch(50% 30 180deg); }").expect("unexpected failure in test");
    assert!(css.contains("lch(50% 30 180deg)"), "应输出 lch(50% 30 180deg): {css}");
}

#[test]
fn test_compile_color_lch_with_alpha() {
    let css = compile_expanded("a { color: lch(50% 30 180deg / 0.8); }").expect("unexpected failure in test");
    assert!(css.contains("lch(50% 30 180deg / 0.8)"), "应输出 lch(50% 30 180deg / 0.8): {css}");
}

#[test]
fn test_compile_color_lch_zero_chroma_none_hue() {
    let css = compile_expanded("a { color: lch(50% 0 180deg); }").expect("unexpected failure in test");
    assert!(css.contains("lch(50% 0 none)"), "chroma=0 时 hue 应输出 none: {css}");
}

// ─── Oklab 色彩空间 ────────────────────────────────────────────────────────

#[test]
fn test_compile_color_oklab_basic() {
    let css = compile_expanded("a { color: oklab(50% 0.1 -0.2); }").expect("unexpected failure in test");
    assert!(css.contains("oklab(50% 0.1 -0.2)"), "应输出 oklab(50% 0.1 -0.2): {css}");
}

#[test]
fn test_compile_color_oklab_with_alpha() {
    let css = compile_expanded("a { color: oklab(75% -0.1 0.15 / 0.6); }").expect("unexpected failure in test");
    assert!(css.contains("oklab(75% -0.1 0.15 / 0.6)"), "应输出 oklab(75% -0.1 0.15 / 0.6): {css}");
}

#[test]
fn test_compile_color_oklab_lightness_scaling() {
    let css = compile_expanded("a { color: oklab(0.6 0.1 0.2); }").expect("unexpected failure in test");
    assert!(css.contains("oklab(60% 0.1 0.2)"), "oklab lightness 0.6 应输出 60%: {css}");
}

// ─── Oklch 色彩空间 ────────────────────────────────────────────────────────

#[test]
fn test_compile_color_oklch_basic() {
    let css = compile_expanded("a { color: oklch(50% 0.1 180deg); }").expect("unexpected failure in test");
    assert!(css.contains("oklch(50% 0.1 180deg)"), "应输出 oklch(50% 0.1 180deg): {css}");
}

#[test]
fn test_compile_color_oklch_with_alpha() {
    let css = compile_expanded("a { color: oklch(50% 0.1 180deg / 0.7); }").expect("unexpected failure in test");
    assert!(css.contains("oklch(50% 0.1 180deg / 0.7)"), "应输出 oklch(50% 0.1 180deg / 0.7): {css}");
}

#[test]
fn test_compile_color_oklch_zero_chroma_none_hue() {
    let css = compile_expanded("a { color: oklch(50% 0 180deg); }").expect("unexpected failure in test");
    assert!(css.contains("oklch(50% 0 none)"), "chroma=0 时 hue 应输出 none: {css}");
}

// ─── 变量使用 ──────────────────────────────────────────────────────────────

#[test]
fn test_compile_color_lab_variable_use() {
    let css = compile_expanded("$c: lab(50% 20 30); a { color: $c; }").expect("unexpected failure in test");
    assert!(css.contains("lab(50% 20 30)"), "lab 变量应正确序列化: {css}");
}

#[test]
fn test_compile_color_oklch_variable_use() {
    let css = compile_expanded("$c: oklch(60% 0.1 240deg); a { color: $c; }").expect("unexpected failure in test");
    assert!(css.contains("oklch(60% 0.1 240deg)"), "oklch 变量应正确序列化: {css}");
}
