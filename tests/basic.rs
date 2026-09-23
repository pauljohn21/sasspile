//! 基础管线测试 — 验证 scan_map / flat_map / collect 链路正确运行

use sasspile::compile;

#[test]
fn compile_empty() {
    let output = compile("");
    assert!(
        output.is_empty() || output.trim().is_empty(),
        "空输入应产出空输出: got {output:?}"
    );
}

#[test]
fn compile_plain_css() {
    let input = "body { margin: 0; }";
    let output = compile(input);
    assert!(
        output.contains("body"),
        "plain CSS 通过管线: got {output:?}"
    );
    assert!(!output.contains("@mixin"), "@mixin 关键字不应泄漏");
    assert!(!output.contains("@include"), "@include 关键字不应泄漏");
}

/// BEM 模式：验证 @mixin 定义和 @include 调用被管线消费
#[test]
fn bem_mixin_consumed() {
    let input = "@mixin b($block) { .el-#{$block} { @content; } }\n\n@include b(button) { color: red; }\n";
    let output = compile(input);
    assert!(!output.contains("@mixin"), "@mixin 被消费");
    assert!(!output.contains("@include"), "@include 被消费");
}

/// @each 循环：验证迭代指令被管线消费
/// 注: 当前管线逐行展开 @each, 不支持多行 block body 收集
#[test]
fn each_loop_consumed() {
    let input = "@each $size in (sm, md, lg) { color: $size; }";
    let output = compile(input);
    // @each 关键字被消费
    assert!(!output.contains("@each"), "@each 被消费, got: {output}");
    // 展开后的 token 应包含具体值 (sm/md/lg)
    assert!(output.contains("sm") || output.contains("md") || output.contains("lg"),
        "@each 应展开一个值, got: {output}");
}
