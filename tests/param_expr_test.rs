//! 测试 @media/@supports 参数中的表达式求值。

use sasspile::compile_expanded;

fn assert_output(input: &str, expected: &str) {
    let result = compile_expanded(input).unwrap_or_else(|e| format!("ERROR: {e}"));
    assert_eq!(
        result.trim(),
        expected.trim(),
        "input: {input:?}\nexpected: {expected:?}\nactual: {result:?}"
    );
}

#[test]
fn test_supports_declaration_lhs_expr() {
    assert_output(
        "@supports (1 + 1: b) {\n  @c;\n}\n",
        "@supports (2: b) {\n  @c;\n}\n",
    );
}

#[test]
fn test_supports_declaration_rhs_expr() {
    assert_output(
        "@supports (a: 1 + 1) {\n  @c;\n}\n",
        "@supports (a: 2) {\n  @c;\n}\n",
    );
}

#[test]
fn test_supports_interp_name() {
    assert_output(
        "@supports #{\"a\"}(b) {\n  @c;\n}\n",
        "@supports a(b) {\n  @c;\n}\n",
    );
}

#[test]
fn test_supports_interp_partial_name() {
    assert_output(
        "@supports a#{\"b\"}c(d) {\n  @e;\n}\n",
        "@supports abc(d) {\n  @e;\n}\n",
    );
}

#[test]
fn test_supports_not() {
    assert_output(
        "@supports not (a: b) {\n  @c;\n}\n",
        "@supports not (a: b) {\n  @c;\n}\n",
    );
}

#[test]
fn test_supports_and() {
    assert_output(
        "@supports (a: b) and (c: d) and (e: f) {\n  @g;\n}\n",
        "@supports (a: b) and (c: d) and (e: f) {\n  @g;\n}\n",
    );
}

#[test]
fn test_supports_or() {
    assert_output(
        "@supports (a: b) or (c: d) or (e: f) {\n  @g;\n}\n",
        "@supports (a: b) or (c: d) or (e: f) {\n  @g;\n}\n",
    );
}

#[test]
fn test_media_range_expr() {
    assert_output(
        "@media (width < 500px + 100px) {\n  a {b: c}\n}\n",
        "@media (width < 600px) {\n  a {\n    b: c;\n  }\n}\n",
    );
}

#[test]
fn test_media_var_range() {
    assert_output(
        "$width: width;\n@media ($width < 600px) {\n  a {b: c}\n}\n",
        "@media (width < 600px) {\n  a {\n    b: c;\n  }\n}\n",
    );
}
