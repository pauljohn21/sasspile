//! 端到端编译测试。

use scss_rs::{compile, compile_expanded, OutputStyle};

#[test]
fn hello_world() {
    let input = r#"
body {
  color: red;
}"#;
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
    let css = result.unwrap();
    assert!(css.contains("body"), "Expected body in: {css}");
    assert!(css.contains("red"), "Expected red in: {css}");
}

#[test]
fn simple_declaration() {
    let input = "a { color: blue; }";
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok());
    let css = result.unwrap();
    assert!(css.contains("a {"));
    assert!(css.contains("color: blue"));
}

#[test]
fn nested_rules() {
    let input = r#"
nav {
  ul {
    margin: 0;
  }
}"#;
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
    let css = result.unwrap();
    assert!(css.contains("nav ul"), "Expected 'nav ul' in: {css}");
}

#[test]
fn variable_substitution() {
    let input = r#"
$primary: #ff0000;
body {
  color: $primary;
}"#;
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
    let css = result.unwrap();
    assert!(css.contains("#ff0000") || css.contains("red"), "Expected color in: {css}");
}

#[test]
fn mixin_include() {
    let input = r#"
@mixin large-text {
  font-size: 20px;
}
.header {
  @include large-text;
}"#;
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
    let css = result.unwrap();
    assert!(css.contains("font-size: 20px"), "Expected font-size in: {css}");
}

#[test]
fn compile_expanded_simple() {
    let input = "div { width: 100px; }";
    let css = compile_expanded(input).unwrap();
    assert!(css.contains("div {"));
    assert!(css.contains("width: 100px"));
}
