//! 综合端到端测试 — 验证 rx 分支完整迁移能力
//!
//! 覆盖能力:
//!   - 变量定义 + 替换
//!   - @mixin 定义 + @include 参数替换
//!   - @extend %placeholder 选择器分组
//!   - @media 查询合并
//!   - @at-root 提升
//!   - @if/@else 条件分支
//!   - @for/@each 循环展开
//!   - 选择器嵌套展开
//!   - 内置函数求值 (lighten/darken/rgba)

use sasspile::compile;

// ─── 场景 1: 变量 + mixin + 选择器嵌套 ─────────────────────────────────────

#[test]
fn vars_mixin_nesting_combined() {
    let input = "$primary: blue;\n$pad: 10px;\n@mixin card($bg) {\n  background: $bg;\n  padding: $pad;\n}\n.container {\n  @include card($primary);\n  .item {\n    color: white;\n  }\n}";
    let output = compile(input);
    tracing::info!(output = %output, "vars_mixin_nesting_combined");
    assert!(output.contains("background: blue"), "变量 mixin 参数传递失败: {output}");
    assert!(output.contains("padding: 10px"), "外部变量引用失败: {output}");
    assert!(output.contains("color: white"), "嵌套选择器失败: {output}");
}

// ─── 场景 2: @extend + @media 合并 ─────────────────────────────────────────

#[test]
fn extend_with_media_merge() {
    let input = "%base { font-size: 14px; }\n@media (max-width: 768px) {\n  .a {\n    @extend %base;\n  }\n}\n@media (max-width: 768px) {\n  .b {\n    color: red;\n  }\n}";
    let output = compile(input);
    tracing::info!(output = %output, "extend_with_media_merge");
    assert!(output.contains(".a"), ".a 应存在: {output}");
    assert!(output.contains(".b"), ".b 应存在: {output}");
    assert!(output.contains("font-size: 14px"), "extend 属性应注入: {output}");
    // @media 合并
    let media_count = output.matches("@media (max-width: 768px)").count();
    assert_eq!(media_count, 1, "@media 应合并: {output}");
}

// ─── 场景 3: @at-root + @if 条件 ────────────────────────────────────────────

#[test]
fn at_root_with_condition() {
    let input = "$theme: dark;\n.wrapper {\n  @if $theme == dark {\n    @at-root .overlay {\n      background: black;\n    }\n  }\n}";
    let output = compile(input);
    tracing::info!(output = %output, "at_root_with_condition");
    assert!(output.contains(".overlay"), "@at-root 应输出 .overlay: {output}");
    assert!(output.contains("background: black"), "@if 分支应为 dark: {output}");
    assert!(!output.contains(".wrapper .overlay"), "@at-root 应脱离嵌套: {output}");
}

// ─── 场景 4: @for 循环 ──────────────────────────────────────────────────────

#[test]
fn for_loop_generates_classes() {
    let input = "@for $i from 1 through 3 {\n  .col-#{$i} {\n    width: #{$i}0%;\n  }\n}";
    let output = compile(input);
    tracing::info!(output = %output, "for_loop_generates_classes");
    assert!(output.contains(".col-1"), "@for 应生成 .col-1: {output}");
    assert!(output.contains(".col-2"), "@for 应生成 .col-2: {output}");
    assert!(output.contains(".col-3"), "@for 应生成 .col-3: {output}");
    assert!(output.contains("width: 10%"), "@for 变量替换失败: {output}");
}

// ─── 场景 5: @each 循环 ─────────────────────────────────────────────────────

#[test]
fn each_loop_iterates_list() {
    let input = "@each $color in red, green, blue {\n  .btn-#{$color} {\n    color: $color;\n  }\n}";
    let output = compile(input);
    tracing::info!(output = %output, "each_loop_iterates_list");
    assert!(output.contains(".btn-red"), "@each 应生成 .btn-red: {output}");
    assert!(output.contains(".btn-green"), "@each 应生成 .btn-green: {output}");
    assert!(output.contains(".btn-blue"), "@each 应生成 .btn-blue: {output}");
}

// ─── 场景 6: 内置颜色函数 ──────────────────────────────────────────────────

#[test]
fn builtin_color_functions() {
    let input = ".foo { color: rgba(#ff0000, 0.5); }";
    let output = compile(input);
    tracing::info!(output = %output, "builtin_color_functions");
    // rgba(#hex, alpha) 输出格式: #hex + alpha_hex (80 = 0.5 * 255)
    assert!(
        output.contains("#ff000080"),
        "rgba(#hex, alpha) 应求值: {output}"
    );
}

// ─── 场景 7: @keyframes 完整结构 ────────────────────────────────────────────

#[test]
fn keyframes_full_structure() {
    let input = "@keyframes pulse {\n  0% { opacity: 0; }\n  50% { opacity: 0.5; }\n  100% { opacity: 1; }\n}\n.anim { animation: pulse 1s; }";
    let output = compile(input);
    tracing::info!(output = %output, "keyframes_full_structure");
    assert!(output.contains("@keyframes pulse"), "应保留 @keyframes: {output}");
    assert!(output.contains("0%"), "应包含 0%: {output}");
    assert!(output.contains("50%"), "应包含 50%: {output}");
    assert!(output.contains("100%"), "应包含 100%: {output}");
    assert!(output.contains("animation: pulse 1s"), "应保留 animation 声明: {output}");
}

// ─── 场景 8: @if/@else if/@else 分支链 ──────────────────────────────────────

#[test]
fn if_elseif_else_chain() {
    let input = "$size: medium;\n@if $size == small {\n  .box { width: 50px; }\n} @else if $size == medium {\n  .box { width: 100px; }\n} @else {\n  .box { width: 200px; }\n}";
    let output = compile(input);
    tracing::info!(output = %output, "if_elseif_else_chain");
    assert!(output.contains("width: 100px"), "@else if medium 应命中: {output}");
    assert!(!output.contains("width: 50px"), "small 分支不应命中: {output}");
    assert!(!output.contains("width: 200px"), "else 分支不应命中: {output}");
}

// ─── 场景 9: 深层选择器嵌套 + & 引用 ────────────────────────────────────────

#[test]
fn deep_nesting_with_ampersand() {
    let input = ".nav {\n  &__list {\n    display: flex;\n    &__item {\n      padding: 10px;\n    }\n  }\n}";
    let output = compile(input);
    tracing::info!(output = %output, "deep_nesting_with_ampersand");
    assert!(output.contains(".nav__list"), "& 展开失败: {output}");
    assert!(output.contains(".nav__list__item"), "深层 & 展开失败: {output}");
}

// ─── 场景 10: @mixin + @content 传递 ────────────────────────────────────────

#[test]
fn mixin_with_content_block() {
    let input = "@mixin responsive {\n  @media (min-width: 768px) {\n    @content;\n  }\n}\n@include responsive {\n  .sidebar {\n    width: 300px;\n  }\n}";
    let output = compile(input);
    tracing::info!(output = %output, "mixin_with_content_block");
    assert!(output.contains("@media"), "content block 应保留 @media: {output}");
    assert!(output.contains(".sidebar"), "content body 应展开: {output}");
}
