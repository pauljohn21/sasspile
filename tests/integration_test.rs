use rx_scss::builder::CompileBuilder;
use rx_scss::pipeline::from_string;
use rx_scss::serialize::Options;
use rx_scss::types::OutputStyle;

#[test]
fn compile_simple_variable_and_rule() {
    let input = r#"
$primary: blue;
body {
  color: $primary;
  margin: 0;
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains("color: blue"));
    assert!(result.contains("margin: 0"));
    assert!(result.contains("body"));
}

#[test]
fn compile_nested_rules() {
    let input = r#"
nav {
  ul {
    margin: 0;
    li {
      display: inline;
    }
  }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains("nav"));
    assert!(result.contains("display: inline"));
    assert!(result.contains("margin: 0"));
}

#[test]
fn compile_media_query() {
    let input = r#"
@media screen and (min-width: 768px) {
  body {
    padding: 16px;
  }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains("@media"));
    assert!(result.contains("padding: 16px"));
}

#[test]
fn compile_mixin_and_include() {
    let input = r#"
@mixin box($size: 10px) {
  width: $size;
  height: $size;
}
.container {
  @include box(20px);
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains("width: 20px") || result.contains("width:20px"));
    assert!(result.contains("height: 20px") || result.contains("height:20px"));
}

#[test]
fn compile_supports_query() {
    let input = r#"
@supports (display: flex) {
  .flex-container {
    display: flex;
  }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains("@supports"));
    assert!(result.contains("display: flex"));
}

#[test]
fn compile_compressed_output() {
    let input = r#"
body {
  color: red;
  margin: 0;
}
"#;
    let result = from_string(input, &Options::compressed()).expect("compilation failed");
    assert!(!result.contains('\n'));
    assert!(result.contains("color:red"));
}

#[test]
fn compile_with_builder() {
    let input = r#"
$bg: white;
.main {
  background: $bg;
}
"#;
    let result = CompileBuilder::new()
        .expanded()
        .compile_string(input)
        .expect("compilation failed");
    assert!(result.contains("background: white"));
}

#[test]
fn compile_with_builder_compressed() {
    let input = r#"
.a { color: blue; }
"#;
    let result = CompileBuilder::new()
        .compressed()
        .compile_string(input)
        .expect("compilation failed");
    assert!(!result.contains('\n'));
    assert!(result.contains("color:blue"));
}

#[test]
fn compile_traversing_at_rule() {
    let input = r#"
@for $i from 1 through 3 {
  .item-#{$i} {
    width: 10px * $i;
  }
}
"#;
    let result = from_string(input, &Options::expanded());
    assert!(result.is_ok(), "for loop compile failed: {:?}", result.err());
}

#[test]
fn compile_each_at_rule() {
    let input = r#"
@each $item in a, b, c {
  .#{$item} {
    color: red;
  }
}
"#;
    let result = from_string(input, &Options::expanded());
    assert!(result.is_ok(), "each loop compile failed: {:?}", result.err());
}

#[test]
fn compile_conditional_at_rule() {
    let input = r#"
$theme: dark;
.component {
  @if $theme == dark {
    background: black;
  }
}
"#;
    let result = from_string(input, &Options::expanded());
    assert!(result.is_ok(), "@if compile failed: {:?}", result.err());
}

#[test]
fn compile_empty_input() {
    let result = from_string("", &Options::expanded());
    assert!(result.is_ok(), "empty input failed: {:?}", result.err());
}

#[test]
fn compile_only_whitespace() {
    let result = from_string("   \n\t  ", &Options::expanded());
    assert!(result.is_ok(), "whitespace input failed: {:?}", result.err());
}

#[test]
fn compile_multiple_variables() {
    let input = r#"
$primary: red;
$secondary: blue;
$accent: green;
.a { color: $primary; }
.b { color: $secondary; }
.c { color: $accent; }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains("color: red") || result.contains("color:red"));
    assert!(result.contains("color: blue") || result.contains("color:blue"));
    assert!(result.contains("color: green") || result.contains("color:green"));
}
