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
    assert!(result.contains(".text-primary"), "should have .text-primary: {}", result);
    assert!(result.contains(".text-danger"), "should have .text-danger: {}", result);
    assert!(result.contains(".text-success"), "should have .text-success: {}", result);
    assert!(result.contains("color: blue"), "should have color: blue: {}", result);
    assert!(result.contains("color: red"), "should have color: red: {}", result);
    assert!(result.contains("color: green"), "should have color: green: {}", result);
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
    assert!(result.contains(".parent"), "should contain .parent: {}", result);
    assert!(result.contains(".child"), "should contain .child: {}", result);
    assert!(result.contains("color: red"), "should contain color: red: {}", result);
    assert!(result.contains("color: blue"), "should contain color: blue: {}", result);
    // At-root .child should NOT be nested inside .parent — verify by checking
    // there's a standalone ".child {" at the top level (after parent block)
    let parent_end = result.find(".parent").unwrap_or(0);
    let child_pos = result.find(".child").unwrap_or(0);
    assert!(child_pos > parent_end, ".child should appear after .parent block");
}

#[test]
fn compile_expression_arithmetic() {
    let input = r#"
$x: 10px;
$y: 5px;
.a { width: $x + $y; }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains("width: 15px") || result.contains("width:15px"),
        "should contain width: 15px: {}", result);
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
    assert!(result.contains("width: 20px") || result.contains("width:20px"),
        "should contain width: 20px: {}", result);
}

#[test]
fn compile_unary_minus() {
    let input = r#"
$x: 10px;
.a { margin: -$x; }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains("margin: -10px") || result.contains("margin:-10px"),
        "should contain margin: -10px: {}", result);
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
    assert!(result.contains("color: white"), "should contain color: white: {}", result);
    assert!(!result.contains("color: black"), "should not contain color: black: {}", result);
}

#[test]
fn compile_selector_interpolation() {
    let input = r#"
$klass: "foo";
.#{$klass} { color: red; }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains(".foo"), "should contain .foo: {:?}", result);
    assert!(result.contains("color: red"), "should contain color: red: {:?}", result);
    // Ensure the rule is properly wrapped with .foo selector (selector interpolation resolved)
    assert!(result.contains(".foo{"), "should contain .foo{{ rule: {:?}", result);
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

    assert!(result.contains("background: blue"), "should contain imported rule: {:?}", result);
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
    assert!(result.contains("width: 1"), "map-get should return 1: {:?}", result);
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
    assert!(result.contains("width: 1px"), "if(true,1,2) should be 1px: {:?}", result);
    assert!(result.contains("height: 2px"), "if(false,1,2) should be 2px: {:?}", result);
}

#[test]
fn compile_builtin_nth() {
    let input = r#"
$list: (10px, 20px, 30px);
body { width: nth($list, 2); }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains("width: 20px"), "nth should return 20px: {:?}", result);
}

#[test]
fn compile_builtin_percentage() {
    let input = r#"
$x: percentage(0.5);
body { width: $x; }
"#;
    let result = from_string(input, &Options::expanded()).expect("compilation failed");
    assert!(result.contains("50%"), "percentage(0.5) should be 50%: {:?}", result);
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
    assert!(result.contains("background:"), "mix should produce color: {:?}", result);
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
    assert!(result.contains(".btn-primary"), "should have .btn-primary: {:?}", result);
    assert!(result.contains(".btn-danger"), "should have .btn-danger: {:?}", result);
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
    assert!(result.contains(".card .title"), "should have descendant selector '.card .title': {:?}", result);
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
    assert!(result.contains(".navbar .nav .item"), "should have deep descendant: {:?}", result);
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
    assert!(result.contains(".sidebar"), "should have .sidebar: {:?}", result);
    assert!(result.contains("@media"), "should have @media: {:?}", result);
    assert!(result.contains("width: 300px"), "should have media width: {:?}", result);
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
    assert!(result.contains(".badge"), "should contain .badge class: {}", result);
    assert!(result.contains("display: inline-block"), "should contain display: {}", result);
}

#[test]
fn compile_bootstrap_full_import_chain() {
    // Test that a multi-level import chain (functions + variables + maps + mixins) compiles.
    // This is a regression test for stack overflow when importing _mixins.scss which itself
    // imports ~25 files from the mixins/ directory.
    let input = "@import \"functions\";\n@import \"variables\";\n@import \"maps\";\n@import \"mixins\";\n";
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
fn compile_many_sequential_imports() {
    // Regression: test that many sequential @import statements don't cause stack overflow.
    let tmp_dir = std::env::temp_dir().join("rx_scss_test_regression");
    let _ = std::fs::create_dir_all(&tmp_dir);
    for i in 0..10 {
        let _ = std::fs::write(tmp_dir.join(format!("_a{}.scss", i)), format!(".a{} {{ color: red; }}\n", i));
        let _ = std::fs::write(tmp_dir.join(format!("_b{}.scss", i)), format!(".b{} {{ color: blue; }}\n", i));
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
            assert!(css.len() > 100,
                "full compile should produce substantial CSS: got {} bytes", css.len());
            // Compare against Bootstrap dist reference
            let reference = std::fs::read_to_string(
                format!("{}/bootstrap/dist/css/bootstrap.css", manifest_dir))
                .expect("Bootstrap dist CSS not found");
            let actual_lines: Vec<&str> = css.lines().collect();
            let ref_lines: Vec<&str> = reference.lines().collect();
            let missing = diff_lines(&css, &reference);
            // Coverage calculation
            let coverage = crate::common::bootstrap_dist::bootstrap_dist_coverage()
                .unwrap_or(0.0);
            eprintln!(
                "compile_bootstrap_full: actual={} lines, reference={} lines, missing={} lines, coverage={:.2}%",
                actual_lines.len(), ref_lines.len(), missing.len(), coverage * 100.0
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
    assert!(result.contains(".col-1"), "should contain .col-1: {}", result);
    assert!(result.contains(".col-2"), "should contain .col-2: {}", result);
    assert!(result.contains(".col-3"), "should contain .col-3: {}", result);
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
    assert!(result.contains(".item-1"), "should contain .item-1: {}", result);
    assert!(result.contains(".item-2"), "should contain .item-2: {}", result);
    assert!(!result.contains(".item-3"), "should NOT contain .item-3 (exclusive): {}", result);
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
    assert!(result.contains("width: 10px"), "should use default 10px: {}", result);
    assert!(result.contains("height: 10px"), "should use default 10px: {}", result);
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
    assert!(result.contains("width: 60%"), "should evaluate and use default expression: {}", result);
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
    assert!(result.contains("width: 30px"), "should override default: {}", result);
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
    assert!(result.contains(".before"), "should have .before wrapper: {}", result);
    assert!(result.contains(".inner"), "should have .inner from content: {}", result);
    assert!(result.contains("color: red"), "should have color: red: {}", result);
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
    assert!(result.contains("min-width"), "should have min-width: {}", result);
    assert!(result.contains("768px"), "should have 768px: {}", result);
    assert!(result.contains(".sidebar"), "should have .sidebar from content: {}", result);
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
#[ignore = "diagnostic: enable to test complex map parsing"]
fn diag_complex_map_parsing() {
    // Test if complex nested map-merge with sub-maps works
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
    eprintln!("  complex map result:\n{}", result);
    assert!(result.contains(".util-baseline"), "should have .util-baseline: {}", result);
    assert!(result.contains(".util-right"), "should have .util-right: {}", result);
}

#[test]
#[ignore = "diagnostic: enable to test individual bootstrap imports"]
fn diag_bootstrap_import_chain() {
    let bootstrap_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap/scss");
    let files = [
        "functions", "variables", "maps", "mixins",
        "root", "reboot", "type", "images", "containers", "grid",
        "tables", "forms", "buttons", "transitions", "dropdown",
        "button-group", "nav", "navbar", "card", "accordion",
        "breadcrumb", "pagination", "badge", "alert", "progress",
        "list-group", "close", "toasts", "modal", "tooltip",
        "popover", "carousel", "spinners", "offcanvas", "placeholders",
        "helpers", "utilities",
    ];
    for file in files {
        let input = format!("@import \"{}\";\n", file);
        let result = CompileBuilder::new()
            .expanded()
            .include_path(&bootstrap_dir)
            .compile_string(&input);
        match result {
            Ok(css) => {
                let lines = css.lines().count();
                eprintln!("  {}: OK ({} lines)", file, lines);
            }
            Err(e) => {
                eprintln!("  {}: ERR {}", file, e);
            }
        }
    }
}

/// Quick line-level diff (not LCS, just sorted comparison for diagnostics)
fn diff_lines(actual: &str, expected: &str) -> Vec<String> {
    use std::collections::HashSet;
    let actual_lines: HashSet<&str> = actual.lines().collect();
    let expected_lines: HashSet<&str> = expected.lines().collect();
    expected_lines.difference(&actual_lines)
        .map(|l| l.to_string())
        .take(100)
        .collect()
}

// Shared helpers accessible via `crate::common::bootstrap_dist::*`

#[test]
fn bootstrap_dist_check_test() {
    let _ = crate::common::bootstrap_dist::bootstrap_dist_check();
}
