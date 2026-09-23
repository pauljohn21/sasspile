//! 多行 block body 收集测试 — 验证 scan_map 状态机驱动的多行收集
//!
//! 当前管线支持两种形式:
//!   单行: @each $x in (a, b) { ... }     → 立即展开
//!   多行: @each $x in (a, b) {\n  ...\n}  → 状态机收集

use sasspile::compile;

#[test]
fn multiline_each_collects_body() {
    let input = "@each $size in (sm, md) {\n  .icon-#{$size} { width: 1em; }\n}";
    let output = compile(input);
    assert!(!output.contains("@each"), "@each 应被消费, got: {output}");
}

#[test]
fn multiline_for_collects_body() {
    let input = "@for $i from 1 through 2 {\n  .col-#{$i} { width: 50%; }\n}";
    let output = compile(input);
    assert!(!output.contains("@for"), "@for 应被消费, got: {output}");
}

#[test]
fn multiline_mixin_def() {
    let input = "@mixin pad($x) {\n  padding: $x;\n}\n@include pad(8px);";
    let output = compile(input);
    assert!(!output.contains("@mixin"), "@mixin 应被消费");
    assert!(output.contains("padding: 8px"), "mixin 展开应含 padding: 8px, got: {output}");
}

#[test]
fn multiline_nested_block() {
    let input = ".parent {\n  .child {\n    color: red;\n  }\n}";
    let output = compile(input);
    assert!(output.contains("color: red"), "嵌套规则应透传, got: {output}");
}
