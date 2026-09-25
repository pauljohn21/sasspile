//! @extend %placeholder 选择器分组测试

use sasspile::compile;

#[test]
fn extend_basic_placeholder_single_line() {
    let input = "%foo { color: red; }\n.bar { @extend %foo; }";
    let output = compile(input);
    tracing::info!(output = %output, "extend basic output");
    assert!(!output.contains("%foo"), "placeholder %foo should not appear in output: {output}");
    assert!(output.contains(".bar"), "extender .bar should appear: {output}");
    assert!(output.contains("color: red"), "extended property not found: {output}");
}

#[test]
fn extend_placeholder_multi_line() {
    let input = "%spacer {\n  margin: 0;\n  padding: 0;\n}\n.box { @extend %spacer; }";
    let output = compile(input);
    tracing::info!(output = %output, "extend multi-line output");
    assert!(!output.contains("%spacer"), "placeholder should not appear: {output}");
    assert!(output.contains(".box"), "extender .box should appear: {output}");
    assert!(output.contains("margin: 0"), "margin property missing: {output}");
    assert!(output.contains("padding: 0"), "padding property missing: {output}");
}

#[test]
fn extend_optional_safe() {
    // !optional + undefined placeholder: extender 保留自身声明, 移除 @extend 行
    let input = ".bar { @extend %undefined !optional; color: red; }";
    let output = compile(input);
    tracing::info!(output = %output, "extend optional output");
    assert!(output.contains(".bar"), "extender should appear: {output}");
    assert!(output.contains("color: red"), "existing declaration should remain: {output}");
    assert!(!output.contains("@extend"), "@extend 应被消费: {output}");
}

#[test]
fn extend_multiple_extenders() {
    let input = "%base { font-size: 14px; }\n.a { @extend %base; }\n.b { @extend %base; }";
    let output = compile(input);
    tracing::info!(output = %output, "extend multiple output");
    assert!(!output.contains("%base"), "placeholder should not appear: {output}");
    assert!(output.contains(".a"), ".a extender missing: {output}");
    assert!(output.contains(".b"), ".b extender missing: {output}");
    assert!(output.contains("font-size: 14px"), "extended prop missing: {output}");
}

#[test]
fn extend_in_nested_selector() {
    let input = "%hover-state { background: blue; }\n.nav {\n  &:hover { @extend %hover-state; }\n}";
    let output = compile(input);
    tracing::info!(output = %output, "extend nested output");
    assert!(output.contains("background: blue"), "extended property missing: {output}");
}

#[test]
fn extend_multiline_body() {
    let input = "%base { font-weight: bold; }\n.wrapper {\n  @extend %base;\n  color: red;\n}";
    let output = compile(input);
    tracing::info!(output = %output, "extend multiline body output");
    assert!(output.contains("font-weight: bold"), "extended prop missing: {output}");
    assert!(output.contains("color: red"), "original prop missing: {output}");
}

#[test]
fn extend_empty_placeholder() {
    let input = "%empty { }\n.x { @extend %empty; }";
    let output = compile(input);
    tracing::info!(output = %output, "extend empty placeholder");
    assert!(output.contains(".x"), "extender .x should appear: {output}");
}
