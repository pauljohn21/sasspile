//! @keyframes 格式化测试

use sasspile::compile;

#[test]
fn keyframe_percentage_basic() {
    let input = "@keyframes fade {\n  0% { opacity: 0; }\n  50% { opacity: 0.5; }\n  100% { opacity: 1; }\n}";
    let output = compile(input);
    tracing::info!(output = %output, "keyframes output");
    assert!(output.contains("0%"), "should contain 0%, got: {output}");
    assert!(output.contains("50%"), "should contain 50%, got: {output}");
    assert!(output.contains("100%"), "should contain 100%, got: {output}");
    assert!(output.contains("opacity: 0"), "should contain opacity 0, got: {output}");
    assert!(output.contains("opacity: 0.5"), "should contain opacity 0.5, got: {output}");
    assert!(output.contains("opacity: 1"), "should contain opacity 1, got: {output}");
}

#[test]
fn keyframe_from_to() {
    let input = "@keyframes slide {\n  from { left: 0; }\n  to { left: 100px; }\n}";
    let output = compile(input);
    tracing::info!(output = %output, "from/to keyframes output");
    assert!(output.contains("from"), "should contain from, got: {output}");
    assert!(output.contains("to"), "should contain to, got: {output}");
}

#[test]
fn keyframe_via_include() {
    let input = "@mixin anim {\n  @keyframes grow {\n    0% { transform: scale(1); }\n    100% { transform: scale(2); }\n  }\n}\n@include anim;";
    let output = compile(input);
    tracing::info!(output = %output, "mixin-keyframes output");
    assert!(output.contains("@keyframes grow"), "should contain keyframes, got: {output}");
}
