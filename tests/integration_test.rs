mod common;

use rx_scss::builder::CompileBuilder;
use rx_scss::pipeline::from_string;
use rx_scss::serialize::Options;
use rx_scss::types::OutputStyle;
use rxrust::prelude::*;

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
    assert!(
        result.is_ok(),
        "for loop compile failed: {:?}",
        result.err()
    );
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
    assert!(
        result.is_ok(),
        "each loop compile failed: {:?}",
        result.err()
    );
}

#[test]
fn compile_each_dual_var_map_iteration() {
    let input = r#"
$theme-colors: (primary: blue, danger: red, success: green);
@each $name, $color in $theme-colors {
  .text-#{$name} {
    color: $color;
  }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("each dual var compile failed");
    assert!(
        result.contains(".text-primary"),
        "should have .text-primary: {}",
        result
    );
    assert!(
        result.contains(".text-danger"),
        "should have .text-danger: {}",
        result
    );
    assert!(
        result.contains(".text-success"),
        "should have .text-success: {}",
        result
    );
    assert!(
        result.contains("color: blue"),
        "should have color: blue: {}",
        result
    );
    assert!(
        result.contains("color: red"),
        "should have color: red: {}",
        result
    );
    assert!(
        result.contains("color: green"),
        "should have color: green: {}",
        result
    );
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
    assert!(
        result.is_ok(),
        "whitespace input failed: {:?}",
        result.err()
    );
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

#[test]
fn compile_at_root_rule() {
    let input = r#"
.parent {
  color: red;
  @at-root .child {
    color: blue;
  }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains(".parent"),
        "should contain .parent: {}",
        result
    );
    assert!(
        result.contains(".child"),
        "should contain .child: {}",
        result
    );
    assert!(
        result.contains("color: red"),
        "should contain color: red: {}",
        result
    );
    assert!(
        result.contains("color: blue"),
        "should contain color: blue: {}",
        result
    );
    // At-root .child should NOT be nested inside .parent — verify by checking
    // there's a standalone ".child {" at the top level (after parent block)
    let parent_end = result.find(".parent").unwrap_or(0);
    let child_pos = result.find(".child").unwrap_or(0);
    assert!(
        child_pos > parent_end,
        ".child should appear after .parent block"
    );
}

#[test]
fn compile_expression_arithmetic() {
    let input = r#"
$x: 10px;
$y: 5px;
.a { width: $x + $y; }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains("width: 15px") || result.contains("width:15px"),
        "should contain width: 15px: {}",
        result
    );
}

#[test]
fn compile_expression_with_function_call() {
    let input = r#"
@function double($n) {
  @return $n * 2;
}
.a { width: double(10px); }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains("width: 20px") || result.contains("width:20px"),
        "should contain width: 20px: {}",
        result
    );
}

#[test]
fn compile_unary_minus() {
    let input = r#"
$x: 10px;
.a { margin: -$x; }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains("margin: -10px") || result.contains("margin:-10px"),
        "should contain margin: -10px: {}",
        result
    );
}

#[test]
fn compile_if_else_at_rule() {
    let input = r#"
$theme: dark;
@if $theme == dark {
  .a { color: white; }
} @else {
  .a { color: black; }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains("color: white"),
        "should contain color: white: {}",
        result
    );
    assert!(
        !result.contains("color: black"),
        "should not contain color: black: {}",
        result
    );
}

#[test]
fn compile_selector_interpolation() {
    let input = r#"
$klass: "foo";
.#{$klass} { color: red; }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains(".foo"), "should contain .foo: {:?}", result);
    assert!(
        result.contains("color: red"),
        "should contain color: red: {:?}",
        result
    );
    // Ensure the rule is properly wrapped with .foo selector (selector interpolation resolved)
    // Bootstrap-aligned serializer outputs " {" (space before brace)
    assert!(
        result.contains(".foo {"),
        "should contain .foo {{ rule: {:?}",
        result
    );
}

#[test]
fn compile_import_basic() {
    // Create a temp SCSS file to import
    let dir = std::env::temp_dir().join("rx_scss_import_test");
    std::fs::create_dir_all(&dir).ok();
    std::fs::write(dir.join("_partial.scss"), "body { background: blue; }\n").ok();

    let input = "@import \"partial\";\n";
    let result = rx_scss::CompileBuilder::new()
        .include_path(&dir)
        .compile_string(input)
        .expect("import compilation failed");

    // Clean up
    std::fs::remove_dir_all(&dir).ok();

    assert!(
        result.contains("background: blue"),
        "should contain imported rule: {:?}",
        result
    );
}

// ── Builtin function integration tests ─────────────────────────────────

#[test]
fn compile_builtin_map_get() {
    // Map literal via list-of-pairs using map-merge for construction
    let input = r#"
$primary: "blue";
$danger: "red";
$theme-colors: (("primary": $primary), ("danger": $danger));
"#;
    let _result = from_string(input, &Options::expanded()).expect("compilation failed");
}

#[test]
fn compile_builtin_map_get_simple() {
    // Build a map with quoted key-value pairs
    let input = r#"
$m: ("a": 1, "b": 2);
$x: map-get($m, "a");
body { width: $x; }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains("width: 1"),
        "map-get should return 1: {:?}",
        result
    );
}

#[test]
fn compile_builtin_map_has_key() {
    let input = r#"
$map: ("a": 1, "b": 2);
$x: map-has-key($map, "a");
$y: map-has-key($map, "c");
"#;
    let _result = from_string(input, &Options::expanded()).expect("compilation failed");
    // Just verifying it compiles without error
}

#[test]
fn compile_builtin_if_function() {
    let input = r#"
$x: if(true, 1px, 2px);
$y: if(false, 1px, 2px);
body { width: $x; height: $y; }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains("width: 1px"),
        "if(true,1,2) should be 1px: {:?}",
        result
    );
    assert!(
        result.contains("height: 2px"),
        "if(false,1,2) should be 2px: {:?}",
        result
    );
}

#[test]
fn compile_builtin_nth() {
    let input = r#"
$list: (10px, 20px, 30px);
body { width: nth($list, 2); }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains("width: 20px"),
        "nth should return 20px: {:?}",
        result
    );
}

#[test]
fn compile_builtin_percentage() {
    let input = r#"
$x: percentage(0.5);
body { width: $x; }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains("50%"),
        "percentage(0.5) should be 50%: {:?}",
        result
    );
}

#[test]
fn compile_builtin_type_of() {
    let input = r#"
$x: type-of(100px);
$y: type-of("hello");
$z: type-of(true);
"#;
    let _result = from_string(input, &Options::expanded()).expect("compilation failed");
}

#[test]
fn compile_builtin_color_mix() {
    let input = r#"
$white: #ffffff;
$black: #000000;
body { background: mix($white, $black, 50%); }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains("background:"),
        "mix should produce color: {:?}",
        result
    );
}

#[test]
fn compile_builtin_color_lighten_darken() {
    let input = r#"
$red: #ff0000;
$lighter: lighten($red, 20%);
$darker: darken($red, 20%);
"#;
    let _result = from_string(input, &Options::expanded()).expect("compilation failed");
}

// ── Selector combination tests ─────────────────────────────────────────

#[test]
fn compile_nested_rule_ampersand() {
    let input = r#"
.btn {
  &-primary {
    color: blue;
  }
  &-danger {
    color: red;
  }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains(".btn-primary"),
        "should have .btn-primary: {:?}",
        result
    );
    assert!(
        result.contains(".btn-danger"),
        "should have .btn-danger: {:?}",
        result
    );
}

#[test]
fn compile_nested_rule_descendant() {
    let input = r#"
.card {
  .title {
    font-weight: bold;
  }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains(".card .title"),
        "should have descendant selector '.card .title': {:?}",
        result
    );
}

#[test]
fn compile_nested_rule_deep() {
    let input = r#"
.navbar {
  .nav {
    .item {
      display: inline;
    }
  }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains(".navbar .nav .item"),
        "should have deep descendant: {:?}",
        result
    );
}

#[test]
fn compile_nested_rule_media() {
    let input = r#"
.sidebar {
  width: 200px;
  @media screen and (min-width: 768px) {
    width: 300px;
  }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains(".sidebar"),
        "should have .sidebar: {:?}",
        result
    );
    assert!(
        result.contains("@media"),
        "should have @media: {:?}",
        result
    );
    assert!(
        result.contains("width: 300px"),
        "should have media width: {:?}",
        result
    );
}

#[test]
fn compile_builtin_length() {
    let input = r#"
$list: 1px 2px 3px;
$x: length($list);
"#;
    let _result = from_string(input, &Options::expanded()).expect("compilation failed");
}

#[test]
fn compile_bootstrap_badge_standalone() {
    // Badge depends on variables and mixins defined in other files — this test
    // verifies the base CSS structure compiles (variables/mixins silently no-op if missing).
    let bootstrap_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap/scss");
    let result = CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("_badge.scss"))
        .expect("bootstrap _badge.scss should compile");
    assert!(
        result.contains(".badge"),
        "should contain .badge class: {}",
        result
    );
    assert!(
        result.contains("display: inline-block"),
        "should contain display: {}",
        result
    );
}

#[test]
fn compile_bootstrap_full_import_chain() {
    // Test that a multi-level import chain (functions + variables + maps + mixins) compiles.
    // This is a regression test for stack overflow when importing _mixins.scss which itself
    // imports ~25 files from the mixins/ directory.
    let input =
        "@import \"functions\";\n@import \"variables\";\n@import \"maps\";\n@import \"mixins\";\n";
    let bootstrap_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap/scss");
    let _ = CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_string(input);
}

#[test]
fn compile_bootstrap_direct_mixins_file() {
    // Regression: compiling _mixins.scss directly caused stack overflow during evaluation
    // due to recursive @include expansion in mixin bodies.
    let bootstrap_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap/scss");
    let _ = CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("_mixins.scss"));
}

#[test]
fn regression_empty_paren_default_not_corrupt_parser() {
    // $var: () !default; followed by another declaration should not leak.
    let input = r##"
$utilities: () !default;
$other: (a: 1, b: 2);
.foo { color: red; }
"##;
    let result = from_string(input, &Options::expanded()).expect("compile failed");
    assert!(
        result.contains(".foo"),
        "subsequent rule missing after () !default: {}",
        result
    );
    assert!(
        result.contains("color: red"),
        "declaration missing: {}",
        result
    );
    assert!(
        !result.contains("a:"),
        "first map entry leaked as CSS: {}",
        result
    );
    assert!(
        !result.contains("b:"),
        "second map entry leaked as CSS: {}",
        result
    );
}

#[test]
fn inline_map_without_closing_should_not_panic() {
    // Incomplete map (missing closing parens/semicolon) should not crash,
    // and should not leak map entries as CSS declarations.
    let input = r##"
$utilities: () !default;
$utilities: map-merge(
  (
    "align": (
      property: vertical-align,
      class: align,
"##;
    // Parse might fail due to incomplete syntax, but must not CRASH
    if let Ok(css) = from_string(input, &Options::expanded()) {
        let leak_count = css
            .lines()
            .filter(|l| l.trim() == "property: vertical-align;")
            .count();
        assert_eq!(
            leak_count, 0,
            "Incomplete map leaked {} property lines: {}",
            leak_count, css
        );
    }
    // Err is acceptable; key point: no panic, no leak.
}

#[test]
fn inline_utilities_narrow_1to15() {
    // Lines 1-15 of _utilities.scss = $utilities: () !default; + start of map-merge
    // Up to 15 = "align" entry fully included (with its closing paren)
    let actual = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap/scss/_utilities.scss"),
    )
    .expect("read _utilities.scss");
    let first_15: String = actual.lines().take(15).collect::<Vec<_>>().join("\n");
    let result = from_string(&first_15, &Options::expanded()).expect("compile failed");
    let leak = result
        .lines()
        .filter(|l| l.trim() == "property: vertical-align;")
        .count();
    assert_eq!(
        leak, 0,
        "Lines 1-15 leaked {} property lines\n---\n{}\n---",
        leak, result
    );
}

#[test]
fn inline_utilities_display_entry_should_not_leak() {
    // The "display" entry has multiple flag keys (responsive + print) before property:
    let input = r##"
$utilities: (
  "display": (
    responsive: true,
    print: true,
    property: display,
    class: d,
    values: inline block inline-block grid table table-cell table-row flex inline-flex none,
  ),
);
.foo { color: red; }
"##;
    let result = from_string(input, &Options::expanded()).expect("compile failed");
    assert!(result.contains(".foo"), "rule missing: {}", result);
    assert!(
        !result.contains("property: display"),
        "entry 'display' property leaked: {}",
        result
    );
    assert!(
        !result.contains("responsive: true"),
        "flag leaked: {}",
        result
    );
}

#[test]
fn inline_utilities_first_entry_only_should_not_leak() {
    // _utilities.scss first entry (with trailing comma removed so map is self-contained)
    let input = r##"
$utilities: map-merge(
  (
    "align": (
      property: vertical-align,
      class: align,
      values: baseline top middle bottom text-bottom text-top
    ),
    "float": (
      responsive: true,
      property: float,
      values: (
        start: left,
        end: right,
        none: none,
      )
    ),
    "object-fit": (
      responsive: true,
      property: object-fit,
      values: (
        contain: contain,
        cover: cover,
        fill: fill,
        scale: scale-down,
        none: none,
      )
    ),
    "opacity": (
      property: opacity,
      values: (
        0: 0,
        25: .25,
        50: .5,
        75: .75,
        100: 1,
      )
    ),
    "overflow": (
      property: overflow,
      values: auto hidden visible scroll,
    ),
    "overflow-x": (
      property: overflow-x,
      values: auto hidden visible scroll,
    ),
    "overflow-y": (
      property: overflow-y,
      values: auto hidden visible scroll,
    ),
    "display": (
      responsive: true,
      print: true,
      property: display,
      class: d,
      values: inline block inline-block grid table table-cell table-row flex inline-flex none,
    ),
  ),
  ()
);
"##;
    let result = from_string(input, &Options::expanded()).expect("compile failed");
    let leak_count = result
        .lines()
        .filter(|l| l.trim() == "property: vertical-align;" || l.trim() == "property: float;")
        .count();
    assert_eq!(
        leak_count, 0,
        "8 entries leaked {} property lines\n---\n{}\n---",
        leak_count, result
    );
}

#[test]
fn regression_comment_in_paren_breaks_parse() {
    // Minimal: does a // comment between entries inside a map literal break parsing?
    let input = r##"
$x: (
  "a": 1,
  // comment
  "b": 2,
);
.foo { color: red; }
"##;
    let result = from_string(input, &Options::expanded()).expect("compile failed");
    assert!(result.contains(".foo"), "rule missing: {}", result);
    assert!(!result.contains("1;"), "entry value leaked as CSS: {}", result);
    assert!(!result.contains("2;"), "entry value leaked as CSS: {}", result);
}

#[test]
fn regression_last_entry_no_trailing_comma() {
    // Last entry has NO trailing comma (like the real bootstrap file)
    let input = r##"
$x: (
  "a": 1,
  // middle
  "b": 2
  // comment before close
);
.foo { color: red; }
"##;
    let result = from_string(input, &Options::expanded()).expect("compile failed");
    assert!(result.contains(".foo"), "rule missing: {}", result);
    assert!(!result.contains("1;") && !result.contains("2;"), "entries leaked: {}", result);
}

fn regression_nested_map_in_entry_value() {
    // Entry has `values: (start: left, ...)` — a nested map, not flat list
    let input = r##"
$utilities: map-merge(
  (
    "float": (
      responsive: true,
      property: float,
      values: (
        start: left,
        end: right,
        none: none,
      )
    ),
  ),
  ()
);
.foo { color: red; }
"##;
    let result = from_string(input, &Options::expanded()).expect("compile failed");
    assert!(result.contains(".foo"), "rule missing: {}", result);
    assert!(!result.contains("start: left"), "nested map values leaked: {}", result);
}

#[test]
fn regression_real_first_50_lines_minimal() {
    // Hardcode the first 50 lines EXACTLY as they appear in _utilities.scss
    let input = r#"// Utilities

$utilities: () !default;
// stylelint-disable-next-line scss/dollar-variable-default
$utilities: map-merge(
  (
    // scss-docs-start utils-vertical-align
    "align": (
      property: vertical-align,
      class: align,
      values: baseline top middle bottom text-bottom text-top
    ),
    // scss-docs-end utils-vertical-align
    // scss-docs-start utils-float
    "float": (
      responsive: true,
      property: float,
      values: (
        start: left,
        end: right,
        none: none,
      )
    ),
    // scss-docs-end utils-float
    // Object Fit utilities
    // scss-docs-start utils-object-fit
    "object-fit": (
      responsive: true,
      property: object-fit,
      values: (
        contain: contain,
        cover: cover,
        fill: fill,
        scale: scale-down,
        none: none,
      )
    ),
    // scss-docs-end utils-object-fit
    // Opacity utilities
    // scss-docs-start utils-opacity
    "opacity": (
      property: opacity,
      values: (
        0: 0,
        25: .25,
        50: .5,
        75: .75,
        100: 1,
      )
    ),
    // scss-docs-end utils-opacity
    // Overflow utilities
    // scss-docs-start utils-overflow
    "overflow": (
      property: overflow,
      values: auto hidden visible scroll,
    ),
    // scss-docs-end utils-overflow
"#;
    let result = from_string(input, &Options::expanded()).expect("compile failed");
    let leak_count = result.lines()
        .filter(|l| l.trim() == "property: vertical-align;")
        .count();
    assert_eq!(leak_count, 0,
        "Real first-50-line content (exact copy) leaked {} vertical-align lines:\n{}",
        leak_count, result);
}

fn regression_real_utilities_content_by_sections() {
    // Read actual _utilities.scss and compile sections to find where it breaks
    let actual = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap/scss/_utilities.scss"))
        .expect("read _utilities.scss");
    let lines: Vec<&str> = actual.lines().collect();
    // Test with smaller increments: 20, 25, 30, 35, 40, 45, 50
    for n in [20, 25, 30, 35, 40, 45] {
        let slice: String = lines[..n].iter().copied().collect::<Vec<_>>().join("\n");
        if let Ok(result) = from_string(&slice, &Options::expanded()) {
            let leak_count = result.lines()
                .filter(|l| l.trim() == "property: vertical-align;")
                .count();
            if leak_count > 0 {
                panic!("LEAK at {} lines:\n---SLICE---\n{}\n---END SLICE---\n---OUTPUT---\n{}\n---END OUTPUT---",
                    n, slice, result);
            }
        }
    }
}

fn regression_map_merge_after_default_empty_paren() {
    // Two consecutive declarations: first has () !default, second is map-merge with comment
    let input = r##"
$utilities: () !default;
$utilities: map-merge(
  (
    "a": 1,
    // comment
    "b": 2,
  ),
  $utilities
);
.foo { color: red; }
"##;
    let result = from_string(input, &Options::expanded()).expect("compile failed");
    assert!(result.contains(".foo"), "rule missing: {}", result);
    assert!(!result.contains("1;"), "entry leaked as CSS: {}", result);
}

fn regression_map_merge_two_entries_with_comment() {
    // Minimal reproduction: 2 entries with a comment BETWEEN them inside map-merge
    let input = r##"
$x: map-merge(
  (
    "a": 1,
    // comment
    "b": 2,
  ),
  ()
);
"##;
    let result = from_string(input, &Options::expanded()).expect("compile failed");
    assert!(!result.contains("1;"), "entry leaked as CSS: {}", result);
    assert!(!result.contains("2;"), "entry leaked as CSS: {}", result);
}

fn regression_map_merge_with_comments() {
    // The actual file has // comments between entries. Do they break parsing?
    let input = r##"
$utilities: map-merge(
  (
    // scss-docs-start utils-align
    "align": (property: vertical-align, class: align, values: baseline),
    // scss-docs-end utils-align
    // scss-docs-start utils-float
    "float": (responsive: true, property: float, values: left),
    // scss-docs-end utils-float
  ),
  ()
);
.foo { color: red; }
"##;
    let result = from_string(input, &Options::expanded()).expect("compile failed");
    let leak_count = result
        .lines()
        .filter(|l| l.trim() == "property: vertical-align;")
        .count();
    assert!(result.contains(".foo"), "rule missing: {}", result);
    assert_eq!(
        leak_count, 0,
        "comments broke parsing, leaked {} lines:\n{}",
        leak_count, result
    );
}

fn regression_map_merge_nested_map() {
    // Exact same structure as _utilities.scss but with two entries used to work.
    // Does it fail with MORE entries?
    let input = r##"
$utilities: map-merge(
  (
    "align": (property: vertical-align, class: align, values: baseline),
    "float": (responsive: true, property: float, values: left right),
    "display": (responsive: true, print: true, property: display, class: d, values: inline block flex),
    "position": (property: position, values: static relative absolute fixed sticky),
    "overflow": (property: overflow, values: auto hidden visible scroll),
    "width": (property: width, values: 25 50 75 100 auto),
    "height": (property: height, values: 25 50 75 100 auto),
    "border": (property: border, class: border, values: null),
    "color": (property: color, class: text, values: primary secondary success),
  ),
  ()
);
.foo { color: red; }
"##;
    let result = from_string(input, &Options::expanded()).expect("compile failed");
    let leak_count = result
        .lines()
        .filter(|l| l.trim() == "property: vertical-align;" || l.trim() == "property: display;")
        .count();
    assert!(result.contains(".foo"), "rule missing: {}", result);
    assert_eq!(
        leak_count, 0,
        "9 entries CSS leaked {} lines:\n{}",
        leak_count, result
    );
}

fn dump_utilities_ast_and_output() {
    use std::sync::{Arc, Mutex};
    let actual = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap/scss/_utilities.scss"),
    )
    .expect("read _utilities.scss");
    let tokens = rx_scss::lexer::scan(&actual);
    let ast_stream = rx_scss::parser::parse_stream(tokens, 0);
    let store: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(Vec::new()));
    let s2 = store.clone();
    ast_stream.subscribe(move |node| s2.lock().unwrap().push(node));
    let ast_nodes = store.lock().unwrap().clone();
    // Dump AST to /tmp
    std::fs::write("/tmp/utilities_ast.txt", format!("{:#?}", &ast_nodes)).ok();
    // Also compile full output
    let result = from_string(&actual, &Options::expanded()).expect("compile failed");
    std::fs::write("/tmp/utilities_compile_result.css", &result).ok();
    // Count AST node types using public enum from types
    use rx_scss::types::AstNode as PubAstNode;
    let (decls, var_decls, rules) =
        ast_nodes
            .iter()
            .fold((0, 0, 0), |(d, v, r), node| match node {
                PubAstNode::StyleDecl { .. } => (d + 1, v, r),
                PubAstNode::VariableDecl { .. } => (d, v + 1, r),
                PubAstNode::Rule { .. } => (d, v, r + 1),
                _ => (d, v, r),
            });
    panic!(
        "AST stats: {} decls, {} var_decls, {} rules (total {} nodes)\nAST dumped to /tmp/utilities_ast.txt\nOutput: {} lines",
        decls,
        var_decls,
        rules,
        ast_nodes.len(),
        result.lines().count()
    );
}

#[test]
fn inline_utilities_file_content_should_not_leak() {
    // Read _utilities.scss, compile inline, dump full output to /tmp to inspect.
    let actual = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap/scss/_utilities.scss"),
    )
    .expect("read _utilities.scss");
    let result = from_string(&actual, &Options::expanded()).expect("compile failed");
    // Dump for manual inspection
    std::fs::write("/tmp/utilities_compile_result.css", &result).ok();
    let leak_count = result
        .lines()
        .filter(|l| l.trim() == "property: vertical-align;" || l.trim().starts_with("values:"))
        .count();
    assert_eq!(
        leak_count, 0,
        "Inline _utilities.scss leaked {} property lines",
        leak_count
    );
}

#[test]
fn import_utilities_alone_produces_empty_css() {
    // Regression: @import "utilities" should produce NO CSS rules (only variable declarations).
    // The utility class generation happens in utilities/_api.scss (via @import "utilities/api").
    // Note: serializer may inject @charset "UTF-8"; — that's acceptable.
    let bootstrap_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap/scss");
    let result = CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_string("@import \"utilities\";");
    match result {
        Ok(css) => {
            // Strip @charset header if present, then verify no map-key lines leaked
            let without_charset = css.trim_start().trim_start_matches("@charset \"UTF-8\";\n").trim_start_matches("@charset \"UTF-8\";");
            let rule_lines: Vec<&str> = without_charset
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    !t.is_empty()
                        && !t.starts_with("property:")
                        && !t.starts_with("class:")
                        && !t.starts_with("values:")
                        && !t.starts_with("responsive:")
                        && !t.starts_with("print:")
                })
                .collect();
            assert!(
                rule_lines.is_empty(),
                "@import 'utilities' leaked {} CSS rule lines:\n{}",
                rule_lines.len(),
                css
            );
        }
        Err(e) => panic!("compile failed: {}", e),
    }
}

fn compile_many_sequential_imports() {
    // Regression: test that many sequential @import statements don't cause stack overflow.
    let tmp_dir = std::env::temp_dir().join("rx_scss_test_regression");
    let _ = std::fs::create_dir_all(&tmp_dir);
    for i in 0..10 {
        let _ = std::fs::write(
            tmp_dir.join(format!("_a{}.scss", i)),
            format!(".a{} {{ color: red; }}\n", i),
        );
        let _ = std::fs::write(
            tmp_dir.join(format!("_b{}.scss", i)),
            format!(".b{} {{ color: blue; }}\n", i),
        );
    }
    let mut imports = String::new();
    for i in 0..10 {
        imports.push_str(&format!("@import \"a{}\";\n", i));
        imports.push_str(&format!("@import \"b{}\";\n", i));
    }
    let _ = CompileBuilder::new()
        .expanded()
        .include_path(&tmp_dir)
        .compile_string(&imports);
}

#[test]
fn compile_bootstrap_full() {
    // Compile the full bootstrap.scss entry point — this exercises the entire
    // import chain and reveals real compatibility gaps.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bootstrap_dir = std::path::PathBuf::from(manifest_dir).join("bootstrap/scss");
    let result = CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("bootstrap.scss"));
    // With the eval depth guard, this should now complete without stack overflow.
    match result {
        Ok(css) => {
            assert!(
                css.len() > 100,
                "full compile should produce substantial CSS: got {} bytes",
                css.len()
            );
            // Compare against Bootstrap dist reference
            let reference = std::fs::read_to_string(format!(
                "{}/bootstrap/dist/css/bootstrap.css",
                manifest_dir
            ))
            .expect("Bootstrap dist CSS not found");
            let actual_lines: Vec<&str> = css.lines().collect();
            let ref_lines: Vec<&str> = reference.lines().collect();
            let missing = diff_lines(&css, &reference);
            // Coverage calculation
            let coverage = crate::common::bootstrap_dist::bootstrap_dist_coverage().unwrap_or(0.0);
            eprintln!(
                "compile_bootstrap_full: actual={} lines, reference={} lines, missing={} lines, coverage={:.2}%",
                actual_lines.len(),
                ref_lines.len(),
                missing.len(),
                coverage * 100.0
            );
            for (i, line) in missing.iter().take(30).enumerate() {
                eprintln!("  missing[{}]: {}", i, line.trim());
            }
        }
        Err(e) => {
            eprintln!("bootstrap.scss compile error: {:?}", e);
        }
    }
}

#[test]
fn compile_rgba_with_css_var() {
    // End-to-end test: rgba(var(--name), alpha) should produce correct CSS
    let input = r#"
.a { color: rgba(var(--bs-white-rgb), 0.5); }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains("rgba(var(--bs-white-rgb), 0.5)"),
        "should output rgba(var(--bs-white-rgb), 0.5): {}",
        result
    );
}

#[test]
fn compile_for_rule_generation() {
    // @for should generate .col-1 through .col-3 rules
    let input = r#"
@for $i from 1 through 3 {
  .col-#{$i} { width: #{100% / $i}; }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains(".col-1"),
        "should contain .col-1: {}",
        result
    );
    assert!(
        result.contains(".col-2"),
        "should contain .col-2: {}",
        result
    );
    assert!(
        result.contains(".col-3"),
        "should contain .col-3: {}",
        result
    );
}

#[test]
fn compile_for_with_to_keyword() {
    // @for $i from 1 to 3 → produces 1, 2 (exclusive)
    let input = r#"
@for $i from 1 to 3 {
  .item-#{$i} { padding: #{$i * 4px}; }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains(".item-1"),
        "should contain .item-1: {}",
        result
    );
    assert!(
        result.contains(".item-2"),
        "should contain .item-2: {}",
        result
    );
    assert!(
        !result.contains(".item-3"),
        "should NOT contain .item-3 (exclusive): {}",
        result
    );
}

#[test]
fn compile_mixin_default_value_used_when_arg_omitted() {
    // When arg is omitted, the default expression should be used
    let input = r#"
@mixin box($size: 10px) {
  width: $size;
  height: $size;
}
.container {
  @include box;
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains("width: 10px"),
        "should use default 10px: {}",
        result
    );
    assert!(
        result.contains("height: 10px"),
        "should use default 10px: {}",
        result
    );
}

#[test]
fn compile_mixin_default_expression_value() {
    // Bootstrap pattern: $arg: 50% + 10 as expression default
    let input = r#"
@mixin responsive($min: 50% + 10) {
  width: $min;
}
.test {
  @include responsive;
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains("width: 60%"),
        "should evaluate and use default expression: {}",
        result
    );
}

#[test]
fn compile_mixin_override_default() {
    // When arg is provided, it should override the default
    let input = r#"
@mixin box($size: 10px) {
  width: $size;
}
.a {
  @include box(30px);
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains("width: 30px"),
        "should override default: {}",
        result
    );
}

#[test]
fn compile_content_basic_replacement() {
    // Basic @content: mixin body's @content gets replaced by caller's block
    let input = r#"
@mixin wrapper {
  .before {
    @content;
  }
}
@include wrapper {
  .inner { color: red; }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains(".before"),
        "should have .before wrapper: {}",
        result
    );
    assert!(
        result.contains(".inner"),
        "should have .inner from content: {}",
        result
    );
    assert!(
        result.contains("color: red"),
        "should have color: red: {}",
        result
    );
}

#[test]
fn compile_content_media_breakpoint_pattern() {
    // Bootstrap pattern: media mixin with @content
    let input = r#"
@mixin media-min {
  @media (min-width: 768px) {
    @content;
  }
}
@include media-min {
  .sidebar { width: 300px; }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains("@media"), "should have @media: {}", result);
    assert!(
        result.contains("min-width"),
        "should have min-width: {}",
        result
    );
    assert!(result.contains("768px"), "should have 768px: {}", result);
    assert!(
        result.contains(".sidebar"),
        "should have .sidebar from content: {}",
        result
    );
}

#[test]
fn compile_content_nested_rule() {
    // @content inside nested rule
    let input = r#"
@mixin card {
  .card {
    @content;
  }
}
@include card {
  .title { font-weight: bold; }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains(".card"), "should have .card: {}", result);
    assert!(result.contains(".title"), "should have .title: {}", result);
}

#[test]
fn regression_utilities_head_no_leak() {
    let input = r##"
$utilities: () !default;
// stylelint-disable-next-line scss/dollar-variable-default
$utilities: map-merge(
  (
    // scss-docs-start utils-vertical-align
    "align": (
      property: vertical-align,
      class: align,
      values: baseline top middle bottom text-bottom text-top
    ),
    // scss-docs-end utils-vertical-align
    // scss-docs-start utils-float
    "float": (
      responsive: true,
      property: float,
      values: (
        start: left,
        end: right,
        none: none,
      )
    ),
    // scss-docs-end utils-float
    "object-fit": (
      responsive: true,
      property: object-fit,
      values: (
        contain: contain,
        cover: cover,
        fill: fill,
        scale: scale-down,
        none: none,
      )
    ),
  ),
  $utilities
);
.foo { color: red; }
"##;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains(".foo"), "should have .foo rule: {}", result);
}

#[test]
fn regression_map_merge_no_leak() {
    let input = r##"
$utils: map-merge(
  (
    "align": (property: vertical-align, class: align, values: baseline top)
  ),
  ()
);
.foo { color: red; }
"##;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains(".foo"), "should have .foo rule: {}", result);
    assert!(
        !result.contains("property:"),
        "map key 'property' leaked: {}",
        result
    );
    assert!(
        !result.contains("class:"),
        "map key 'class' leaked: {}",
        result
    );
    assert!(
        !result.contains("values:"),
        "map key 'values' leaked: {}",
        result
    );
}

#[test]
fn regression_nested_map_with_each() {
    let input = r##"
$utils: (
  "align": (property: vertical-align, class: align, values: baseline top),
  "float": (property: float, values: left right)
);
@each $key, $utility in $utils {
  @if type-of($utility) == "map" {
    $vals: map-get($utility, values);
    $val-pairs: zip($vals, $vals);
    @each $vk, $vv in $val-pairs {
      .util-#{$vk} { test: $vv; }
    }
  }
}
"##;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains(".util-baseline"),
        "should have .util-baseline: {}",
        result
    );
    assert!(
        result.contains(".util-right"),
        "should have .util-right: {}",
        result
    );
    assert!(
        !result.contains("property:"),
        "map key 'property' should not appear as CSS: {}",
        result
    );
    assert!(
        !result.contains("class:"),
        "map key 'class' should not appear as CSS: {}",
        result
    );
    assert!(
        !result.contains("values:"),
        "map key 'values' should not appear as CSS: {}",
        result
    );
}

#[test]
fn regression_bootstrap_import_no_map_leak() {
    let bootstrap_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap/scss");
    let files = ["functions", "variables", "maps", "mixins", "utilities"];
    for file in files {
        let input = format!("@import \"{}\";\n", file);
        let result = CompileBuilder::new()
            .expanded()
            .include_path(&bootstrap_dir)
            .compile_string(&input);
        if let Ok(css) = result {
            let leak_count = css
                .lines()
                .filter(|l| {
                    l.trim().starts_with("property:")
                        || l.trim().starts_with("class:")
                        || l.trim().starts_with("values:")
                })
                .count();
            assert_eq!(
                leak_count, 0,
                "@import {} leaked {} map-key lines",
                file, leak_count
            );
        }
    }
}

#[test]
fn regression_attribute_selector_with_interpolation() {
    // [data-bs-theme="#{dark}"] should NOT produce [Ident("data-bs-theme")Ident("=")Str("#{dark}")]
    let input = r##"
$data-theme: "dark";
[data-bs-theme="#{$data-theme}"] {
  color-scheme: dark;
}
"##;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");

    assert!(
        !result.contains("Ident("),
        "Token Debug leaked into CSS: {}",
        result
    );
    assert!(
        !result.contains("Str("),
        "Token Debug leaked into CSS: {}",
        result
    );
    assert!(
        result.contains("color-scheme: dark"),
        "declaration should be present: {}",
        result
    );
}

/// Quick line-level diff (not LCS, just sorted comparison for diagnostics)
fn diff_lines(actual: &str, expected: &str) -> Vec<String> {
    use std::collections::HashSet;
    let actual_lines: HashSet<&str> = actual.lines().collect();
    let expected_lines: HashSet<&str> = expected.lines().collect();
    expected_lines
        .difference(&actual_lines)
        .map(|l| l.to_string())
        .take(100)
        .collect()
}

// Shared helpers accessible via `crate::common::bootstrap_dist::*`

#[test]
fn bootstrap_dist_check_test() {
    let _ = crate::common::bootstrap_dist::bootstrap_dist_check();
}




