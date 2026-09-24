//! Sass 缩进语法测试: +name → @include name + 多行 @include (using / content)

use sasspile::compile;

#[test]
fn test_indented_include_simple() {
    let input = "@mixin foo {\n  color: red;\n}\n.bar {\n  +foo\n}\n";
    let output = compile(input);
    assert!(
        output.contains("color: red"),
        "expected 'color: red' in output, got:\n{output}"
    );
}

#[test]
fn test_indented_include_inside_rules() {
    let input = "@mixin bold {\n  font-weight: bold;\n}\n.wrapper {\n  .inner {\n    +bold\n  }\n}\n";
    let output = compile(input);
    assert!(
        output.contains("font-weight: bold"),
        "expected 'font-weight: bold' in output, got:\n{output}"
    );
}

#[test]
fn test_indented_include_with_args() {
    let input = "@mixin pad($n) {\n  padding: $n;\n}\n.box {\n  +pad(10px)\n}\n";
    let output = compile(input);
    assert!(
        output.contains("padding: 10px"),
        "expected 'padding: 10px' in output, got:\n{output}"
    );
}

#[test]
fn test_regular_at_include_still_works() {
    let input = "@mixin theme {\n  background: blue;\n}\n@include theme;\n";
    let output = compile(input);
    assert!(
        output.contains("background: blue"),
        "expected 'background: blue' in output, got:\n{output}"
    );
}

#[test]
fn test_multiline_include_with_content() {
    let input = "@mixin media {\n  @content;\n}\n@include media {\n  .resp {\n    display: flex;\n  }\n}\n";
    let output = compile(input);
    assert!(
        output.contains("display: flex"),
        "expected 'display: flex' in output, got:\n{output}"
    );
}
