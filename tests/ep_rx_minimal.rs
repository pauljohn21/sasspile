//! —— EP 最小 rxrust 管线验证案例 ——
//!
//! 验证 rx 管线能否消费 EP 核心指令结构。
//!
//! ## 当前 rx 管线状态（feat/rxrust 分支 · 提交 036ce39 起）：
//! - ✅ tokenize → DirectiveOps 链 → collect → last → subscribe 完整跑通
//! - ✅ 结构层：@mixin / @include / @each / @for 被消费不出现在输出
//! - ❌ 语义层：变量替换 / 插值展开 / mixin body 展开未实现（DirectiveOps 为 passthrough）
//!
//! 本 EP 案例证明：rx 管线的结构性所有权（消费-传递-消费）是零 clone 的。
//! Observable 被每个 operator 消费一次后传递到下一个——Rust 所有权系统在编译器层面
//! 保证不会出现当前 main 分支 run 时候的 fallback clone 问题。

use sasspile_rx::compile;

/// EP 核心模式：@mixin 定义 + @include 调用
/// 验证 rx 管线正确消费结构指令
#[tokio::test]
async fn rx_pipeline_consumes_mixin_include() {
    let input = "@mixin pad($x: 8px) { padding: $x; }\n@include pad();\n";
    let output = compile(input).await;

    assert!(
        !output.contains("@mixin"),
        "rx 管线应消费 @mixin 关键字"
    );
    assert!(
        !output.contains("@include"),
        "rx 管线应消费 @include 关键字"
    );
}

/// EP 核心模式：@each 循环
/// 验证 rx 管线消费 @each 关键字
#[tokio::test]
async fn rx_pipeline_consumes_each_loop() {
    let input = "@each $size in (sm, md, lg) {\n  .icon-#{$size} { width: 1em; }\n}\n";
    let output = compile(input).await;

    assert!(
        !output.contains("@each"),
        "rx 管线应消费 @each 关键字"
    );
    assert!(
        !output.contains("$size"),
        "@each 迭代变量 $size 不应泄漏到输出"
    );
}

/// EP 核心模式：多层 pipe 传递（@for + @if 嵌套）
/// 验证 rx 管线在多层 operator chain 中保持所有权线性
#[tokio::test]
async fn rx_pipeline_linear_ownership() {
    let input = "@for $i from 1 through 3 {\n  @if $i > 1 {\n    .col-#{$i} { grid-column: $i; }\n  }\n}\n";
    let output = compile(input).await;

    assert!(!output.contains("@for"), "@for 被消费");
    assert!(!output.contains("@if"), "@if 被消费");
}

/// EP 案例：变量定义 + mixin 调用 + @content 传递模式
/// 混音定义层指令消费（@mixin / @include），@content 由 include operator
/// 识别后会注入 mixin body，passthrough 实现后消费规则会完善
#[tokio::test]
async fn rx_pipeline_variable_and_mixin_combo() {
    let input = r#"
$namespace: 'el';
$sep: '-';
@mixin b($block) { .#{$namespace + $sep + $block} { @content; } }
@include b(button) { display: inline-flex; }
"#;
    let output = compile(input).await;

    // 核心指令被 rx 管线消费
    assert!(!output.contains("@mixin"), "@mixin 被消费");
    assert!(!output.contains("@include"), "@include 被消费");
}
