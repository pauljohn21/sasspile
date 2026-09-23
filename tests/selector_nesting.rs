//! 选择器嵌套展开测试 — 验证 & 父引用和嵌套规则展开

use sasspile::compile;

#[test]
fn nested_child_selector() {
    let input = ".parent {\n  .child {\n    color: red;\n  }\n}";
    let output = compile(input);
    // 嵌套规则应展开为 .parent .child { color: red; }
    assert!(output.contains(".parent .child"), "嵌套选择器展开为 .parent .child, got: {output}");
}

#[test]
fn parent_reference_ampersand() {
    let input = ".btn {\n  &:hover {\n    color: blue;\n  }\n}";
    let output = compile(input);
    // &:hover 展开为 .btn:hover
    assert!(output.contains(".btn:hover"), "& 父引用展开为 .btn:hover, got: {output}");
}

#[test]
fn parent_reference_with_modifier() {
    let input = ".el-input {\n  &--large {\n    font-size: 18px;\n  }\n}";
    let output = compile(input);
    assert!(output.contains(".el-input--large"), "& 修饰符展开, got: {output}");
}

#[test]
fn deeply_nested() {
    let input = ".a {\n  .b {\n    .c {\n      color: red;\n    }\n  }\n}";
    let output = compile(input);
    // 深层嵌套: .a .b .c (三层)
    assert!(output.contains(".a .b .c"), "深层嵌套展开, got: {output}");
}
