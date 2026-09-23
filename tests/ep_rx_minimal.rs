//! EP 最小 rxrust 管线验证案例
//!
//! 验证 rx 管线能否消费 EP 核心指令结构（@mixin/@include/@each/@for）。
//! 注: 当前管线逐行处理, 所有指令须在同一行 (含 body)。

use sasspile::compile;

/// EP 核心模式：@mixin 定义 + @include 调用
#[test]
fn rx_pipeline_consumes_mixin_include() {
    let input = "@mixin pad($x: 8px) { padding: $x; }\n@include pad();\n";
    let output = compile(input);
    assert!(!output.contains("@mixin"), "rx 管线应消费 @mixin 关键字");
    assert!(!output.contains("@include"), "rx 管线应消费 @include 关键字");
}

/// EP 核心模式：@each 循环
#[test]
fn rx_pipeline_consumes_each_loop() {
    let input = "@each $size in (sm, md, lg) { .icon-#{$size} { width: 1em; } }";
    let output = compile(input);
    assert!(!output.contains("@each"), "rx 管线应消费 @each 关键字, got: {output}");
}

/// EP 案例：变量定义 + mixin 调用
#[test]
fn rx_pipeline_variable_and_mixin_combo() {
    let input = "@mixin pad($x: 8px) { padding: $x; }\n@include pad(16px);\n";
    let output = compile(input);
    assert!(!output.contains("@mixin"), "@mixin 被消费");
    assert!(!output.contains("@include"), "@include 被消费");
    assert!(output.contains("padding: 16px"), "mixin 展开结果正确, got: {output}");
}
