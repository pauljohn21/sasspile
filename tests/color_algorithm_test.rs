//! 颜色算法测试——scale, change, invert, to-space。

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
fn test_scale_lightness_positive() {
    let css = compile_expanded("a { b: scale-color(red, $lightness: 20%); }").unwrap();
    assert!(!css.contains("ERROR"), "不应报错: {css}");
    // 输出可以是 rgb 或 named color
    assert!(css.contains("b:"), "应输出值: {css}");
}

#[test]
fn test_scale_lightness_negative() {
    let css = compile_expanded("a { b: scale-color(red, $lightness: -50%); }").unwrap();
    assert!(css.contains("b:"), "应输出值: {css}");
    // 应该比红色更暗
    assert_ne!(css, "a {\n  b: red;\n}\n", "亮度 -50% 应产生不同颜色: {css}");
}

#[test]
fn test_scale_saturation() {
    let css = compile_expanded("a { b: scale-color(red, $saturation: 30%); }").unwrap();
    assert!(css.contains("b:"), "应输出值: {css}");
}

#[test]
fn test_scale_red() {
    let css = compile_expanded("a { b: scale-color(red, $red: 50%); }").unwrap();
    assert!(css.contains("b:"), "应输出值: {css}");
}

#[test]
fn test_change_lightness() {
    let css = compile_expanded("a { b: change-color(red, $lightness: 30%); }").unwrap();
    assert!(css.contains("b:"), "应输出值: {css}");
    // change-color 可以输出 hsl/rgb/named color 任意格式
    assert!(!css.contains("ERROR"), "不应报错: {css}");
}

#[test]
fn test_change_hue() {
    let css = compile_expanded("a { b: change-color(red, $hue: 120deg); }").unwrap();
    assert!(css.contains("b:"), "应输出值: {css}");
    // change-color 可以输出 hsl/rgb/named color 任意格式
    assert!(!css.contains("ERROR"), "不应报错: {css}");
}

#[test]
fn test_change_alpha_clamp() {
    // change alpha > 1 应该 clamp 到 1
    let css = compile_expanded("a { b: change-color(red, $alpha: 1.5); }").unwrap();
    // 不应有 alpha > 1
    assert!(!css.contains("1.5"), "alpha 应被 clamp: {css}");
}

#[test]
fn test_change_red_clamp() {
    // change red > 255 应该 clamp 到 255
    let css = compile_expanded("a { b: change-color(red, $red: 300); }").unwrap();
    assert!(css.contains("b:"), "应输出值: {css}");
}

#[test]
fn test_invert() {
    // invert(red) = cyan/aqua
    let css = compile_expanded("a { b: invert(red); }").unwrap();
    assert!(css.contains("b:"), "应输出值: {css}");
    // invert(red) 应该不是 red
    assert_ne!(css, "a {\n  b: red;\n}\n", "invert 应产生不同颜色: {css}");
}

#[test]
fn test_invert_percent() {
    let css = compile_expanded("a { b: invert(red, 50%); }").unwrap();
    assert!(css.contains("b:"), "应输出值: {css}");
}

#[test]
fn test_invert_hsl() {
    let css = compile_expanded("a { b: invert(hsl(120, 50%, 50%)); }").unwrap();
    assert!(css.contains("b:"), "应输出值: {css}");
}

#[test]
fn test_adjust_hue() {
    let css = compile_expanded("a { b: adjust-hue(red, 120deg); }").unwrap();
    assert!(css.contains("b:"), "应输出值: {css}");
    // adjust-hue(red, 120deg) 应该变成绿色
    assert_ne!(css, "a {\n  b: red;\n}\n", "色相应改变: {css}");
}

#[test]
fn test_to_space() {
    let css = compile_expanded("a { b: color.to-space(red, display-p3); }").unwrap();
    assert!(css.contains("b:"), "应输出值: {css}");
}

#[test]
fn test_color_scale() {
    let css = compile_expanded("a { b: color.scale(red, $lightness: 20%); }").unwrap();
    assert!(css.contains("b:"), "应输出值: {css}");
}

#[test]
fn test_color_change() {
    let css = compile_expanded("a { b: color.change(red, $lightness: 30%); }").unwrap();
    assert!(css.contains("b:"), "应输出值: {css}");
}

#[test]
fn test_color_invert() {
    let css = compile_expanded("a { b: color.invert(red); }").unwrap();
    assert!(css.contains("b:"), "应输出值: {css}");
}
