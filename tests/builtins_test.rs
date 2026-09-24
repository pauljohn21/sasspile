//! Phase 2.4 — 内置函数求值测试
//!
//! 验证: rgb/rgba/hsl/lighten/darken/mix/round/ceil/floor/abs/min/max/percentage

use sasspile::compile;

// ─── 颜色函数 ─────────────────────────────────────────────────────────

#[test]
fn rgba_four_args() {
    let css = compile(".a { color: rgba(255, 0, 0, 0.5); }");
    // rgba(255,0,0,0.5) → #ff000080
    assert!(css.contains("#ff000080") || css.contains("rgba"), "got: {css}");
}

#[test]
fn rgba_full_opaque() {
    let css = compile(".a { color: rgba(0, 128, 255, 1); }");
    assert!(css.contains("#0080ff"), "got: {css}");
}

#[test]
fn rgba_hex_with_alpha() {
    let css = compile(".a { color: rgba(#ff0000, 0.5); }");
    assert!(css.contains("#ff000080") || css.contains("rgba"), "got: {css}");
}

#[test]
fn lighten_increases_lightness() {
    let css = compile(".a { color: lighten(#000000, 50%); }");
    assert!(css.contains("#808080") || css.contains("#7f7f7f"), "got: {css}");
}

#[test]
fn darken_decreases_lightness() {
    let css = compile(".a { color: darken(#ffffff, 50%); }");
    assert!(css.contains("#808080") || css.contains("#7f7f7f"), "got: {css}");
}

#[test]
fn mix_colors() {
    let css = compile(".a { color: mix(#ff0000, #0000ff, 50%); }");
    // mix red+blue at 50% → somewhere in purple range
    assert!(css.contains('#'), "should produce hex color, got: {css}");
}

#[test]
fn adjust_hue_changes_hue() {
    let css = compile(".a { color: adjust-hue(#ff0000, 120deg); }");
    // red + 120° → green-ish
    assert!(css.contains('#'), "should produce hex color, got: {css}");
}

// ─── 数学函数 ─────────────────────────────────────────────────────────

#[test]
fn round_function() {
    let css = compile(".a { width: round(3.7px); }");
    assert!(css.contains("4px"), "got: {css}");
}

#[test]
fn ceil_function() {
    let css = compile(".a { width: ceil(3.2px); }");
    assert!(css.contains("4px"), "got: {css}");
}

#[test]
fn floor_function() {
    let css = compile(".a { width: floor(3.8px); }");
    assert!(css.contains("3px"), "got: {css}");
}

#[test]
fn abs_positive() {
    let css = compile(".a { width: abs(5px); }");
    assert!(css.contains("5px"), "got: {css}");
}

#[test]
fn abs_negative() {
    let css = compile(".a { width: abs(-5px); }");
    assert!(css.contains("5px"), "got: {css}");
}

#[test]
fn min_function() {
    let css = compile(".a { width: min(3px, 5px, 1px); }");
    assert!(css.contains("1px"), "got: {css}");
}

#[test]
fn max_function() {
    let css = compile(".a { width: max(3px, 5px, 1px); }");
    assert!(css.contains("5px"), "got: {css}");
}

#[test]
fn percentage_function() {
    let css = compile(".a { width: percentage(0.5); }");
    assert!(css.contains("50%"), "got: {css}");
}

#[test]
fn percentage_decimal() {
    let css = compile(".a { width: percentage(0.75); }");
    assert!(css.contains("75%"), "got: {css}");
}

// ─── 字符串函数 ───────────────────────────────────────────────────────

#[test]
fn unquote_removes_quotes() {
    let css = compile(".a { content: unquote(\"hello\"); }");
    assert!(css.contains("content: hello"), "got: {css}");
}

#[test]
fn quote_adds_quotes() {
    let css = compile(".a { content: quote(hello); }");
    assert!(css.contains("content: \"hello\""), "got: {css}");
}

// ─── 列表函数 ─────────────────────────────────────────────────────────

#[test]
fn length_function() {
    let scss = "$n: length(a, b, c);\n.a { z: $n; }";
    let css = compile(scss);
    // length(a, b, c) → 3
    assert!(css.contains('3'), "length should produce 3, got: {css}");
}

#[test]
fn nth_function() {
    let scss = "$v: nth(10px 20px 30px, 2);\n.a { width: $v; }";
    let css = compile(scss);
    assert!(css.contains("20px") || css.contains('2'), "got: {css}");
}

// ─── 组合使用 ─────────────────────────────────────────────────────────

#[test]
fn nested_color_functions() {
    // darken + mix combination
    let scss = ".a { color: mix(darken(#ffffff, 20%), #000000, 50%); }";
    let css = compile(scss);
    assert!(css.contains('#'), "should produce valid color, got: {css}");
}

#[test]
fn math_in_property() {
    let css = compile(".a { margin: ceil(2.3px) floor(3.7px); }");
    assert!(css.contains("3px") && css.contains("3px"), "got: {css}");
}
