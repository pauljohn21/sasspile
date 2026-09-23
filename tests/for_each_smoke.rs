//! @for/@each 指令展开测试 (单行 form — 管线逐行处理)
//! 注: 当前管线按行处理, 要求 @for/@each + body 在同一行

use sasspile::compile;

#[test]
fn scss_for_through() {
    let input = "@for $i from 1 through 3 { b: $i; }";
    let output = compile(input);
    assert!(output.contains("b: 1"), "should contain b: 1, got: {output}");
    assert!(output.contains("b: 2"), "should contain b: 2, got: {output}");
    assert!(output.contains("b: 3"), "should contain b: 3, got: {output}");
    assert!(!output.contains("@for"), "@for should be consumed");
}

#[test]
fn scss_for_to_exclusive() {
    let input = "@for $i from 1 to 3 { b: $i; }";
    let output = compile(input);
    assert!(output.contains("b: 1"), "should contain b: 1, got: {output}");
    assert!(output.contains("b: 2"), "should contain b: 2, got: {output}");
    assert!(!output.contains("b: 3"), "should NOT contain b: 3, got: {output}");
    assert!(!output.contains("@for"), "@for should be consumed");
}

#[test]
fn scss_for_backward() {
    let input = "@for $i from 3 through 1 { b: $i; }";
    let output = compile(input);
    assert!(output.contains("b: 3"), "should contain b: 3, got: {output}");
    assert!(output.contains("b: 2"), "should contain b: 2, got: {output}");
    assert!(output.contains("b: 1"), "should contain b: 1, got: {output}");
}

#[test]
fn scss_each_list() {
    let input = "@each $c in (red, green, blue) { color: $c; }";
    let output = compile(input);
    assert!(!output.contains("@each"), "@each should be consumed, got: {output}");
    assert!(output.contains("color: red") || output.contains("color: green") || output.contains("color: blue"),
        "@each should expand, got: {output}");
}

#[test]
fn sass_for_inclusive() {
    let input = "@for $i from 1 through 3 { b: $i; }";
    let output = compile(input);
    assert!(output.contains("b: 1"), "should contain b: 1, got: {output}");
    assert!(output.contains("b: 2"), "should contain b: 2, got: {output}");
    assert!(output.contains("b: 3"), "should contain b: 3, got: {output}");
}
