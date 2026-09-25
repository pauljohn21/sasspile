//! @function / @return 单元测试

use sasspile::compile;

#[test]
fn test_basic_function_def_and_call() {
    let result = compile("@function calc() {@return 1}\na {b: calc()}");
    assert_eq!(result, "a {\n  b: 1;\n}\n", "basic function call should evaluate to return value");
}

#[test]
fn test_function_with_args() {
    let result = compile("@function double($x) {@return $x * 2}\na {b: double(5)}");
    assert_eq!(result, "a {\n  b: 5 * 2;\n}\n", "function arg should substitute into return value");
}

#[test]
fn test_function_clamp() {
    let result = compile("@function clamp() {@return 1}\na {b: clamp()}");
    assert_eq!(result, "a {\n  b: 1;\n}\n");
}

#[test]
fn test_function_and_keyword() {
    let result = compile("@function and() {@return 1}\na {b: and()}");
    assert_eq!(result, "a {\n  b: 1;\n}\n");
}

#[test]
fn test_function_or_keyword() {
    let result = compile("@function or() {@return 1}\na {b: or()}");
    assert_eq!(result, "a {\n  b: 1;\n}\n");
}

#[test]
fn test_function_not_keyword() {
    let result = compile("@function not() {@return 1}\na {b: not()}");
    assert_eq!(result, "a {\n  b: 1;\n}\n");
}
