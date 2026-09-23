//! CSS 输出格式化测试 — 验证 token 序列正确渲染为格式化 CSS

use sasspile::compile;

#[test]
fn basic_css_structure() {
    let input = "body {\n  margin: 0;\n  padding: 0;\n}";
    let output = compile(input);
    // 期望格式化后含 { 和 }
    assert!(output.contains("{"), "output missing {{");
    assert!(output.contains("}"), "output missing }}");
}

#[test]
fn nested_rule_serialized() {
    let input = ".parent {\n  .child {\n    color: red;\n  }\n}";
    let output = compile(input);
    assert!(output.contains(".parent .child"), "nested selector expands, got: {output}");
    assert!(output.contains("color: red"), "declaration passes through: {output}");
}

#[test]
fn mixin_expansion_serialized() {
    let input = "@mixin pad($x) {\n  padding: $x;\n}\n@include pad(8px);";
    let output = compile(input);
    assert!(output.contains("padding: 8px"), "mixin expands to padding: 8px, got: {output}");
}

#[test]
fn compiled_no_directives_leak() {
    let input = "@mixin pad($x) { padding: $x; }\n@include pad(8px);\n.foo { color: red; }";
    let output = compile(input);
    assert!(!output.contains("@mixin"), "@mixin consumed");
    assert!(!output.contains("@include"), "@include consumed");
    assert!(output.contains(".foo"), "plain selector preserved");
}
