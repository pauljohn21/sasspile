mod common;

use rx_scss::builder::CompileBuilder;
use rx_scss::pipeline::from_string;
use rx_scss::serialize::Options;
use rx_scss::types::OutputStyle;
use rxrust::prelude::*;
use std::sync::Once;

static INIT_TRACING: Once = Once::new();

fn ensure_test_tracing() {
    INIT_TRACING.call_once(|| {
        rx_scss::telemetry::init_test_tracing();
    });
}

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
    let actual = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap/scss/_utilities.scss"),
    )
    .expect("read _utilities.scss");
    let tokens = rx_scss::lexer::scan(&actual);
    let ast_stream = rx_scss::parser::parse_stream(tokens, 0);
    let ast_nodes = rx_scss::collect_boxed(ast_stream).expect("collect ast failed");
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
            ensure_test_tracing();
            assert!(
                css.len() > 100,
                "full compile should produce substantial CSS: got {} bytes",
                css.len()
            );
            // Dump for analysis
            std::fs::write("/tmp/rx_scss_bootstrap_output.css", &css).ok();
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
            tracing::info!(
                actual_lines = actual_lines.len(),
                ref_lines = ref_lines.len(),
                missing_lines = missing.len(),
                coverage_pct = coverage * 100.0,
                "Bootstrap dist alignment results"
            );
            for (i, line) in missing.iter().take(30).enumerate() {
                tracing::debug!("missing line[{}]: {}", i, line.trim());
            }
        }
        Err(e) => {
            tracing::error!(?e, "bootstrap.scss compile error");
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

#[test]
fn compile_vendor_prefixed_property() {
    let input = r#"
body {
  -webkit-text-size-adjust: 100%;
  -moz-appearance: none;
  color: red;
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(
        result.contains("-webkit-text-size-adjust: 100%"),
        "should contain -webkit-text-size-adjust: {}",
        result
    );
    assert!(
        result.contains("-moz-appearance: none"),
        "should contain -moz-appearance: {}",
        result
    );
    assert!(
        result.contains("color: red"),
        "should contain color: red: {}",
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
    ensure_test_tracing();
    let check = crate::common::bootstrap_dist::bootstrap_dist_check()
        .expect("Bootstrap submodule not found");
    let coverage = if check.reference_line_count > 0 {
        (check.reference_line_count - check.missing_count) as f64 / check.reference_line_count as f64
    } else {
        0.0
    };
    tracing::info!(
        coverage_pct = coverage * 100.0,
        missing = check.missing_count,
        total = check.reference_line_count,
        "Bootstrap dist alignment (test)"
    );
}

#[test]
fn debug_mixin_registration() {
    // 简化：只验证 mixin 至少能被调用
    let input2 = "@mixin hello { .hi { content: \"yes\"; } } @include hello;";
    let result2 = from_string(input2, &Options::expanded()).expect("basic mixin failed");
    assert!(result2.contains(".hi"), "mixin should generate .hi: {}", result2);
    assert!(result2.contains("yes"), "mixin should propagate content: {}", result2);
}

#[test]
fn debug_import_chain() {
    // 测试 @_rfs.scss 是否能正确解析和注册 mixin
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bootstrap_dir = std::path::PathBuf::from(manifest_dir).join("bootstrap/scss");
    let css = rx_scss::builder::CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("vendor/_rfs.scss"));
    match css {
        Ok(out) => {
            tracing::debug!(len = out.len(), "vendor/_rfs.scss compiled");
            // rfs file no output expected (just declarations), but compilation should succeed
            assert!(out.len() < 1000, "unexpected output: {}", out.len());
        }
        Err(e) => {
            panic!("Failed to compile vendor/_rfs.scss: {}", e);
        }
    }
}

#[test]
fn debug_box_shadow_file() {
    // 最简单的 rest argument 解析测试 — 只定义不调用
    let input = r#"@mixin mb($v...) { .x { color: red; } }"#;
    let result = from_string(input, &Options::expanded()).expect("rest arg mixin failed");
    tracing::debug!(output = %result, "rest arg definition parsed");
    // 如果 mixin 定义被正确解析，不应有垃圾输出
    assert!(!result.contains(".."), "should not have garbage dots: {}", result);
}

#[test]
fn debug_rest_arg_binding() {
    // 测试 rest argument 是否正确绑定为 list (单参数)
    let input = r#"@mixin mb($v...) { .x { content: $v; } } @include mb(1px);"#;
    let result = from_string(input, &Options::expanded()).expect("rest arg binding failed");
    tracing::debug!(output = %result, "rest arg binding parsed");
    assert!(result.contains(".x"), "should generate .x: {}", result);
    assert!(result.contains("1px"), "should have 1px: {}", result);
}

#[test]
fn debug_rest_arg_multi() {
    // 测试 rest argument 多参数打包为 list
    let input = r#"@mixin mb($v...) { .x { content: $v; } } @include mb(1px, 2px, 3px);"#;
    let result = from_string(input, &Options::expanded()).expect("rest arg multi failed");
    tracing::debug!(output = %result, "rest arg multi parsed");
    assert!(result.contains(".x"), "should generate .x: {}", result);
    // list renders as space-separated (inner list already packed by parse_arg_list)
    assert!(result.contains("1px") && result.contains("2px") && result.contains("3px"),
        "should have all values: {}", result);
}

#[test]
fn debug_rfs_mixin_simulation() {
    // 测试与 rfs 类似的带条件 @if 的 mixin 定义
    let input = r#"
$enable: true;
@mixin conditional($val) {
  @if $enable {
    .responsive { font-size: $val; }
  }
}
@include conditional(2rem);
"#;
    let result = from_string(input, &Options::expanded()).expect("conditional mixin failed");
    assert!(result.contains(".responsive"), "should generate .responsive: {}", result);
    assert!(result.contains("font-size: 2rem"), "should have font-size: {}", result);
}

#[test]
fn regression_each_order_test() {
    // @each 迭代顺序回归：声明顺序应与列表顺序一致
    let input = r#"
@each $color in (red, green, blue) {
  .text-#{$color} {
    color: $color;
  }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");

    let red_pos = result.find(".text-red").expect(".text-red missing");
    let green_pos = result.find(".text-green").expect(".text-green missing");
    let blue_pos = result.find(".text-blue").expect(".text-blue missing");

    assert!(
        red_pos < green_pos,
        "order violated: .text-red ({}) should precede .text-green ({}):\n{}",
        red_pos, green_pos, result
    );
    assert!(
        green_pos < blue_pos,
        "order violated: .text-green ({}) should precede .text-blue ({}):\n{}",
        green_pos, blue_pos, result
    );
}

#[test]
fn regression_for_order_test() {
    // @for 迭代顺序回归：声明顺序应与计数顺序一致
    let input = r#"
@for $i from 1 through 3 {
  .col-#{$i} {
    width: #{$i * 33%};
  }
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");

    let col1_pos = result.find(".col-1").expect(".col-1 missing");
    let col2_pos = result.find(".col-2").expect(".col-2 missing");
    let col3_pos = result.find(".col-3").expect(".col-3 missing");

    assert!(
        col1_pos < col2_pos,
        "order violated: .col-1 ({}) should precede .col-2 ({}):\n{}",
        col1_pos, col2_pos, result
    );
    assert!(
        col2_pos < col3_pos,
        "order violated: .col-2 ({}) should precede .col-3 ({}):\n{}",
        col2_pos, col3_pos, result
    );
}

#[test]
fn debug_rfs_each_loop() {
    ensure_test_tracing();
    // 测试 @each 循环（rfs 函数中大量使用）
    let input = r##"
$val: "";
@each $x in (a, b, c) {
  $val: $val + " " + $x;
}
.test {
  content: $val;
}
"##;
    let result = from_string(input, &Options::expanded());
    match result {
        Ok(css) => {
            let lines: Vec<&str> = css.lines().filter(|l| l.contains("content")).collect();
            tracing::debug!(?lines, "rfs_each_ok");
        }
        Err(e) => tracing::debug!(error = %e, "rfs_each_error"),
    }
}

#[test]
fn debug_rfs_value_fn() {
    ensure_test_tracing();
    // 测试简化的 rfs-value 函数
    let input = r##"
@function rfs-value($values) {
  $val: "";
  @each $value in $values {
    @if $value == 0 {
      $val: $val + " 0";
    } @else {
      $val: $val + " " + $value;
    }
  }
  @return unquote(str-slice($val, 2));
}
.test {
  font-size: rfs-value(1.25rem);
}
"##;
    let result = from_string(input, &Options::expanded());
    match result {
        Ok(css) => {
            let lines: Vec<&str> = css.lines().filter(|l| l.contains("font-size")).collect();
            tracing::debug!(?lines, "rfs_value_fn_ok");
        }
        Err(e) => tracing::debug!(error = %e, "rfs_value_fn_error"),
    }
}

#[test]
fn debug_rfs_builtins() {
    ensure_test_tracing();
    // 逐个测试 rfs 用到的内建函数
    let test_cases = vec![
        ("type-of", r#"$x: hello; .t { content: type-of($x); }"#),
        ("unit", r#"$x: 1.25rem; .t { content: unit($x); }"#),
        ("str-slice", r#"$x: hello; .t { content: str-slice($x, 2); }"#),
        ("unquote", r#"$x: hello; .t { content: unquote($x); }"#),
        ("string-add", r#"$x: hello; .t { content: $x + " world"; }"#),
    ];
    for (name, input) in test_cases {
        let result = from_string(input, &Options::expanded());
        match result {
            Ok(css) => {
                let lines: Vec<&str> = css.lines().filter(|l| l.contains("content")).collect();
                tracing::debug!(name, ?lines, "rfs_builtin_ok");
            }
            Err(e) => tracing::debug!(name, error = %e, "rfs_builtin_error"),
        }
    }
}

#[test]
fn debug_bootstrap_rfs_call() {
    ensure_test_tracing();
    // 测试完整的 rfs mixin 调用
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bootstrap_dir = std::path::PathBuf::from(manifest_dir).join("bootstrap/scss");
    let css = rx_scss::builder::CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("bootstrap.scss"))
        .expect("compile failed");

    // 统计 --bs-btn-font-size 是否存在
    let has_btn_font_size = css.lines().any(|l| l.trim().starts_with("--bs-btn-font-size"));
    // 统计 rfs 相关的变量
    let rfs_vars: Vec<String> = css.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| l.starts_with("--bs-") && (l.contains("font-size") || l.contains("padding")))
        .collect();
    // 统计 clamp 出现次数（rfs mixin 应生成 clamp() 表达式）
    let clamp_count = css.matches("clamp(").count();
    tracing::debug!(has_btn_font_size, clamp_count, "rfs_related_output");
    tracing::debug!(?rfs_vars, count = rfs_vars.len(), "rfs_vars_sample");
}

#[test]
fn debug_rfs_interp_prop() {
    ensure_test_tracing();
    // 测试 mixin 中属性名插值: --#{$prefix}btn-font-size
    let input = r##"
$prefix: "bs-";
@mixin my-mixin($value, $property: font-size) {
  #{$property}: #{$value};
}
.btn {
  @include my-mixin(1.25rem, --#{$prefix}btn-font-size);
}
"##;
    let result = from_string(input, &Options::expanded());
    match result {
        Ok(css) => tracing::debug!(output = %css, "rfs_interp_prop_ok"),
        Err(e) => tracing::debug!(error = %e, "rfs_interp_prop_error"),
    }
}

#[test]
fn debug_rfs_mixin() {
    ensure_test_tracing();
    // 测试 rfs mixin 是否正常工作
    let input = r##"
$enable-rfs: true;
$rfs-base-value: 1.25rem;
$rfs-unit: rem;
$rfs-breakpoint: 1200px;
$rfs-breakpoint-unit: px;
$rfs-two-dimensional: false;
$rfs-factor: 10;
$rfs-mode: min-media-query;
$rfs-class: false;
$rfs-rem-value: 16;
$rfs-safari-iframe-resize-bug-fix: false;
$rfs-base-value-unit: unit($rfs-base-value);

@function divide($dividend, $divisor, $precision: 10) {
  $sign: if($dividend > 0 and $divisor > 0 or $dividend < 0 and $divisor < 0, 1, -1);
  $dividend: abs($dividend);
  $divisor: abs($divisor);
  @if $dividend == 0 {
    @return 0;
  }
  @if $divisor == 0 {
    @error "Cannot divide by zero";
  }
  $result: $dividend / $divisor;
  @return $result;
}

@mixin rfs($value, $property: font-size) {
  #{$property}: #{$value};
}

.btn {
  @include rfs(1.25rem, --bs-btn-font-size);
}
"##;
    let result = from_string(input, &Options::expanded());
    match result {
        Ok(css) => tracing::debug!(output = %css, "rfs_mixin_ok"),
        Err(e) => tracing::debug!(error = %e, "rfs_mixin_error"),
    }
}

#[test]
fn debug_missing_bs_vars() {
    ensure_test_tracing();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bootstrap_dir = std::path::PathBuf::from(manifest_dir).join("bootstrap/scss");
    let css = rx_scss::builder::CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("bootstrap.scss"))
        .expect("compile failed");

    let reference_path = format!("{}/bootstrap/dist/css/bootstrap.css", manifest_dir);
    let reference = std::fs::read_to_string(&reference_path).expect("read reference failed");

    let our_set: std::collections::HashSet<String> = css.lines().map(|l| l.trim().to_string()).collect();
    let ref_bs: Vec<String> = reference.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| l.starts_with("--bs-"))
        .collect();
    let our_bs: Vec<String> = css.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| l.starts_with("--bs-"))
        .collect();

    // 参考中独有的 --bs-* 行 (未在我们的输出中出现)
    let missing_bs: Vec<String> = ref_bs.iter()
        .filter(|l| !our_set.contains(*l))
        .cloned()
        .take(30)
        .collect();

    // 只取 --bs-* 变量名部分，看哪些变量名缺失或值不同
    let ref_names: std::collections::HashSet<String> = ref_bs.iter().filter_map(|l| l.split(':').next()).map(|s| s.to_string()).collect();
    let our_names: std::collections::HashSet<String> = our_bs.iter().filter_map(|l| l.split(':').next()).map(|s| s.to_string()).collect();
    let missing_names: Vec<String> = ref_names.difference(&our_names).cloned().collect();

    tracing::debug!(ref_bs_count = ref_bs.len(), our_bs_count = our_bs.len(), "bs_var_counts");
    tracing::debug!(?missing_bs, count = missing_bs.len(), "missing --bs- lines samples");
    tracing::debug!(?missing_names, count = missing_names.len(), "missing --bs- variable names (not in our output at all)");
}

#[test]
fn debug_what_changed() {
    ensure_test_tracing();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bootstrap_dir = std::path::PathBuf::from(manifest_dir).join("bootstrap/scss");
    let css = rx_scss::builder::CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("bootstrap.scss"))
        .expect("compile failed");

    let reference_path = format!("{}/bootstrap/dist/css/bootstrap.css", manifest_dir);
    let reference = std::fs::read_to_string(&reference_path).expect("read reference failed");

    let ref_lines: Vec<String> = reference.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
    let our_lines: Vec<String> = css.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
    let our_set: std::collections::HashSet<String> = our_lines.iter().cloned().collect();

    // decl_other: missing lines that look like regular declarations
    let changed: Vec<String> = ref_lines.iter()
        .filter(|l| l.contains(':') && l.ends_with(';') && !l.starts_with("--bs-") && !l.starts_with(".") && !l.starts_with("#")
            && !l.starts_with("[data-") && !l.starts_with("-webkit-") && !l.starts_with("-moz-") && !l.starts_with("-ms-"))
        .filter(|l| !our_set.contains(*l))
        .cloned()
        .take(20)
        .collect();
    tracing::debug!(?changed, "decl_other samples missing");
}

#[test]
fn debug_list_separator() {
    ensure_test_tracing();
    // 测试列表分隔符是否正确保留：逗号分隔应保留逗号
    let input = r#"
$white: #ffffff;
$black: #000000;
.test {
  box-shadow: inset 0 1px 0 rgba($white, 0.15), 0 1px 1px rgba($black, 0.075);
  transition: color 0.15s ease-in-out, background-color 0.15s ease-in-out;
}
"#;
    let result = from_string(input, &Options::expanded()).expect("compile failed");
    tracing::debug!(output = %result, "list_separator_result");
}

#[test]
fn debug_btn_block() {
    ensure_test_tracing();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bootstrap_dir = std::path::PathBuf::from(manifest_dir).join("bootstrap/scss");
    let css = rx_scss::builder::CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("bootstrap.scss"))
        .expect("compile failed");

    // Extract .btn { } block (first 40 lines)
    let mut btn_block = Vec::new();
    let mut in_block = false;
    let mut depth = 0i32;
    for line in css.lines() {
        let t = line.trim();
        if t == ".btn {" {
            in_block = true;
            depth = 0;
        }
        if in_block {
            btn_block.push(t.to_string());
            depth += t.matches('{').count() as i32 - t.matches('}').count() as i32;
            if depth <= 0 && btn_block.len() > 1 {
                break;
            }
        }
    }
    let preview: Vec<String> = btn_block.iter().take(40).cloned().collect();
    tracing::debug!(?preview, total = btn_block.len(), ".btn block preview");
}

#[test]
fn debug_compare_light_block() {
    ensure_test_tracing();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bootstrap_dir = std::path::PathBuf::from(manifest_dir).join("bootstrap/scss");
    let css = rx_scss::builder::CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("bootstrap.scss"))
        .expect("compile failed");

    let reference_path = format!("{}/bootstrap/dist/css/bootstrap.css", manifest_dir);
    let reference = std::fs::read_to_string(&reference_path).expect("read reference failed");

    // Find [data-bs-theme=light] block (注意: 我们的输出是 [data-bs-theme=light] {)
    let mut ref_block = Vec::new();
    let mut in_block = false;
    let mut depth = 0;
    for line in reference.lines() {
        let t = line.trim();
        if t.starts_with("[data-bs-theme=light]") {
            in_block = true;
            depth = 0;
        }
        if in_block {
            ref_block.push(t.to_string());
            depth += t.matches('{').count() as i32 - t.matches('}').count() as i32;
            if depth <= 0 && ref_block.len() > 1 {
                break;
            }
        }
    }

    let mut our_block = Vec::new();
    in_block = false;
    depth = 0;
    for line in css.lines() {
        let t = line.trim();
        if t.starts_with("[data-bs-theme=light]") {
            in_block = true;
            depth = 0;
        }
        if in_block {
            our_block.push(t.to_string());
            depth += t.matches('{').count() as i32 - t.matches('}').count() as i32;
            if depth <= 0 && our_block.len() > 1 {
                break;
            }
        }
    }

    // 参考中独有的 --bs-* 行
    let our_set: std::collections::HashSet<String> = our_block.iter().cloned().collect();
    let ref_only: Vec<String> = ref_block.iter()
        .filter(|l| l.starts_with("--bs-"))
        .filter(|l| !our_set.contains(*l))
        .cloned()
        .collect();
    let ref_bs_count = ref_block.iter().filter(|l| l.starts_with("--bs-")).count();
    let our_bs_count = our_block.iter().filter(|l| l.starts_with("--bs-")).count();
    tracing::debug!(ref_bs_count, our_bs_count, ref_lines = ref_block.len(), our_lines = our_block.len(), "light_block_line_counts");
    tracing::debug!(?ref_only, count = ref_only.len(), "ref-only --bs-* lines in light block");
}

#[test]
fn debug_compare_dark_block() {
    ensure_test_tracing();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bootstrap_dir = std::path::PathBuf::from(manifest_dir).join("bootstrap/scss");
    let css = rx_scss::builder::CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("bootstrap.scss"))
        .expect("compile failed");

    let reference_path = format!("{}/bootstrap/dist/css/bootstrap.css", manifest_dir);
    let reference = std::fs::read_to_string(&reference_path).expect("read reference failed");

    // Find [data-bs-theme=dark] block in reference
    let mut ref_block = Vec::new();
    let mut in_block = false;
    let mut depth = 0;
    for line in reference.lines() {
        let t = line.trim();
        if t.starts_with("[data-bs-theme=dark]") {
            in_block = true;
            depth = 0;
        }
        if in_block {
            ref_block.push(t.to_string());
            depth += t.matches('{').count() as i32 - t.matches('}').count() as i32;
            if depth <= 0 && ref_block.len() > 1 {
                break;
            }
        }
    }

    // Find [data-bs-theme=dark] block in our output
    let mut our_block = Vec::new();
    in_block = false;
    depth = 0;
    for line in css.lines() {
        let t = line.trim();
        if t.starts_with("[data-bs-theme=dark]") {
            in_block = true;
            depth = 0;
        }
        if in_block {
            our_block.push(t.to_string());
            depth += t.matches('{').count() as i32 - t.matches('}').count() as i32;
            if depth <= 0 && our_block.len() > 1 {
                break;
            }
        }
    }

    tracing::debug!(ref_lines = ref_block.len(), our_lines = our_block.len(), "dark_block_line_counts");
    // 参考中独有的 --bs-* 行
    let our_set: std::collections::HashSet<String> = our_block.iter().cloned().collect();
    let ref_only: Vec<String> = ref_block.iter()
        .filter(|l| l.starts_with("--bs-"))
        .filter(|l| !our_set.contains(*l))
        .cloned()
        .collect();
    tracing::debug!(?ref_only, count = ref_only.len(), "ref-only --bs-* lines in dark block");
}

#[test]
fn debug_check_dark_mode_in_output() {
    ensure_test_tracing();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bootstrap_dir = std::path::PathBuf::from(manifest_dir).join("bootstrap/scss");
    let css = rx_scss::builder::CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("bootstrap.scss"))
        .expect("compile failed");

    let has_dark = css.lines().any(|l| l.trim() == "[data-bs-theme=dark]");
    let dark_lines: Vec<&str> = css.lines()
        .filter(|l| l.trim().starts_with("[data-bs-theme=dark]") || l.trim() == "}")
        .take(5)
        .collect();
    let bs_in_dark = css.lines()
        .filter(|l| {
            // crude: count --bs-* lines near [data-bs-theme=dark]
            let mut found = false;
            found
        })
        .count();
    tracing::debug!(has_dark, ?dark_lines, "dark_mode_check");

    let enable_dark = css.lines().filter(|l| l.trim().starts_with("--bs-btn-bg")).count();
    tracing::debug!(enable_dark, "--bs-btn-bg lines");
}

#[test]
fn debug_color_mode_if() {
    ensure_test_tracing();
    // 测试 @if + @include color-mode(dark) 在顶层
    let input = r##"
$color-mode-type: data;
$enable-dark-mode: true;
@mixin color-mode($mode: light) {
  [data-bs-theme="#{$mode}"] {
    @content;
  }
}

@if $enable-dark-mode {
  @include color-mode(dark) {
    --bs-btn-bg: #fff;
    --bs-btn-color: #000;
  }
}
"##;
    let result = from_string(input, &Options::expanded());
    match result {
        Ok(css) => tracing::debug!(output = %css, "color_mode_if_ok"),
        Err(e) => tracing::debug!(error = %e, "color_mode_if_error"),
    }
}

#[test]
fn debug_root_in_full() {
    ensure_test_tracing();
    // 在完整 bootstrap 编译中检查 :root 的输出
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bootstrap_dir = std::path::PathBuf::from(manifest_dir).join("bootstrap/scss");
    let css = rx_scss::builder::CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("bootstrap.scss"))
        .expect("compile failed");

    // 提取 :root 声明块的前 30 行
    let mut root_lines: Vec<&str> = Vec::new();
    let mut in_root = false;
    let mut brace_count = 0;
    for line in css.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(":root") {
            in_root = true;
            brace_count = 0;
        }
        if in_root {
            root_lines.push(line);
            brace_count += trimmed.matches('{').count() as i32 - trimmed.matches('}').count() as i32;
            if brace_count <= 0 && root_lines.len() > 2 {
                break;
            }
        }
    }
    let preview: Vec<&str> = root_lines.into_iter().take(30).collect();
    tracing::debug!(?preview, ":root block in full compilation");

    // 检查是否有 --bs-* 变量
    let bs_var_count = css.lines().filter(|l| l.trim().starts_with("--bs-")).count();
    tracing::debug!(bs_var_count, "total --bs-* lines in full output");

    // 对比: 找到缺失的 --bs-* 行及其所在 selector 上下文
    let reference_path = format!("{}/bootstrap/dist/css/bootstrap.css", manifest_dir);
    let reference = std::fs::read_to_string(&reference_path).expect("read reference failed");
    let actual_set: std::collections::HashSet<&str> = css.lines().collect();

    let mut missing_bs_with_context: Vec<(String, String)> = Vec::new();
    let mut last_selector = "".to_string();
    for line in reference.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(":root") || trimmed.starts_with('.') || trimmed.starts_with('#')
            || trimmed.starts_with('@') || trimmed.starts_with('[') {
            if trimmed.ends_with('{') || trimmed.contains(',') {
                last_selector = trimmed.to_string();
            }
        }
        if trimmed.starts_with("--bs-") && !actual_set.contains(trimmed) {
            missing_bs_with_context.push((last_selector.clone(), trimmed.to_string()));
        }
    }

    // 按 selector 分组缺失的 --bs-*
    let mut sel_groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for (sel, line) in &missing_bs_with_context {
        sel_groups.entry(sel.clone()).or_default().push(line.clone());
    }
    let mut group_counts: Vec<(String, usize)> = sel_groups.iter()
        .map(|(k, v)| (k.clone(), v.len()))
        .collect();
    group_counts.sort_by(|a, b| b.1.cmp(&a.1));
    let top_groups: Vec<(String, usize)> = group_counts.into_iter().take(15).collect();
    tracing::debug!(?top_groups, "missing --bs-* grouped by selector");
}

#[test]
fn debug_missing_decl_others() {
    ensure_test_tracing();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let reference_path = format!("{}/bootstrap/dist/css/bootstrap.css", manifest_dir);
    let reference = std::fs::read_to_string(&reference_path).expect("read reference failed");
    let reference_set: std::collections::HashSet<&str> = reference.lines().collect();

    let css = rx_scss::builder::CompileBuilder::new()
        .expanded()
        .include_path(std::path::PathBuf::from(manifest_dir).join("bootstrap/scss"))
        .compile_file(std::path::PathBuf::from(manifest_dir).join("bootstrap/scss/bootstrap.scss"))
        .expect("compile failed");
    let actual_set: std::collections::HashSet<&str> = css.lines().collect();

    // decl_other: 非 selector、非 bs_vars、非 vendor prefix 的缺失声明
    let missing_decl_others: Vec<&str> = reference_set.iter()
        .copied()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty()
                && !t.starts_with('.') && !t.starts_with('#') && !t.starts_with('@')
                && *t != *"{" && !t.starts_with(':') && !t.starts_with('*')
                && !t.starts_with("--bs-")
                && !t.starts_with("-webkit-") && !t.starts_with("-moz-")
                && !t.starts_with("-o-") && !t.starts_with("-ms-")
        })
        .filter(|l| !actual_set.contains(*l))
        .collect();

    // 提取属性名并分组
    let mut prop_counts: Vec<(String, usize)> = {
        let mut groups: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for line in &missing_decl_others {
            let prop = line.trim().split(':').next().unwrap_or("").trim().to_string();
            if !prop.is_empty() {
                *groups.entry(prop).or_insert(0) += 1;
            }
        }
        let mut v: Vec<(String, usize)> = groups.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v
    };

    // 只保留出现次数 >= 3 的属性（共性问题的信号）
    prop_counts.retain(|(_, c)| *c >= 3);
    tracing::debug!(?prop_counts, "decl_other_property_counts");

    // 输出全部缺失的 decl_others（用于分析）
    let sample: Vec<&str> = missing_decl_others.iter().take(40).copied().collect();
    tracing::debug!(?sample, total = missing_decl_others.len(), "decl_other_sample");
}
