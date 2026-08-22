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

#[test]
fn css_custom_property() {
    let input = "a { --my-var: 42px; color: red; }";
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
    let css = result.unwrap();
    assert!(css.contains("--my-var"), "Expected --my-var in: {css}");
}

#[test]
fn arithmetic_expression() {
    let input = "$w: 10px; a { width: $w + 5px; }";
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
    let css = result.unwrap();
    assert!(css.contains("15px"), "Expected 15px in: {css}");
}

#[test]
fn function_call() {
    let input = "a { width: max(10px, 20px); }";
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
    let css = result.unwrap();
    assert!(css.contains("20px"), "Expected 20px in: {css}");
}

#[test]
fn list_value() {
    let input = "$list: 1px 2px 3px; a { margin: $list; }";
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn map_value() {
    let input = "$map: (a: 1, b: 2); a { content: length($map); }";
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn slash_separator() {
    let input = "a { font: 10px/20px sans-serif; }";
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
    let css = result.unwrap();
    assert!(css.contains("10px/20px"), "Expected 10px/20px in: {css}");
}

#[test]
fn for_loop() {
    let input = r#"
@for $i from 1 through 3 {
  .item-#{$i} { width: $i * 10px; }
}
"#;
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
    let css = result.unwrap();
    assert!(css.contains(".item-1"), "Expected .item-1 in: {css}");
    assert!(css.contains(".item-3"), "Expected .item-3 in: {css}");
    assert!(css.contains("30px"), "Expected 30px in: {css}");
}

#[test]
fn each_loop() {
    let input = r#"
$colors: red green blue;
@each $color in $colors {
  .#{$color} { color: $color; }
}
"#;
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn if_else() {
    let input = r#"
$debug: true;
a {
  @if $debug {
    border: 1px solid red;
  } @else {
    border: none;
  }
}
"#;
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
    let css = result.unwrap();
    assert!(css.contains("border"), "Expected border in: {css}");
}

#[test]
fn mixin_with_args() {
    let input = r#"
@mixin button($color, $size: 10px) {
  color: $color;
  font-size: $size;
}
.btn {
  @include button(blue);
}
"#;
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
    let css = result.unwrap();
    assert!(css.contains("color: blue"), "Expected color: blue in: {css}");
    assert!(css.contains("10px"), "Expected 10px in: {css}");
}

#[test]
fn mixin_named_args() {
    let input = r#"
@mixin button($color, $size) {
  color: $color;
  font-size: $size;
}
.btn {
  @include button($size: 20px, $color: red);
}
"#;
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
    let css = result.unwrap();
    assert!(css.contains("color: red"), "Expected color: red in: {css}");
    assert!(css.contains("20px"), "Expected 20px in: {css}");
}

#[test]
fn user_function() {
    let input = r#"
@function double($n) {
  @return $n * 2;
}
a { width: double(10px); }
"#;
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
    let css = result.unwrap();
    assert!(css.contains("20px"), "Expected 20px in: {css}");
}

#[test]
fn content_block() {
    let input = r#"
@mixin responsive {
  @media (max-width: 600px) {
    @content;
  }
}
a {
  @include responsive {
    color: red;
  }
}
"#;
    let result = compile(input, OutputStyle::Expanded);
    assert!(result.is_ok(), "{:?}", result.err());
    let css = result.unwrap();
    assert!(css.contains("max-width"), "Expected max-width in: {css}");
    assert!(css.contains("color: red"), "Expected color: red in: {css}");
}
