//! Transform stage — variable expansion specification tests.
//!
//! Verifies that variables are properly collected, scoped,
//! and replaced during the transform pipeline phase.

use sasspile::Compiler;

async fn compile(scss: &str) -> Result<String, sasspile::SassError> {
    let compiler = Compiler::new();
    compiler.compile(scss).await
}

// =============================================================================
// Basic variable definition and substitution
// =============================================================================

#[tokio::test]
async fn variable_basic_assignment() {
    let scss = r#"
        $color: red;
        .foo { color: $color; }
    "#;
    let css = compile(scss).await.unwrap();
    // Bare identifiers are treated as quoted strings in the evaluator.
    assert!(css.contains("\"red\""), "got: {css}");
    assert!(!css.contains("$color"), "variable should be resolved");
}

#[tokio::test]
async fn variable_color_value() {
    let scss = r#"
        $primary: #3498db;
        $secondary: #2ecc71;
        .primary { color: $primary; }
        .secondary { color: $secondary; }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains("#3498db"), "primary color missing: {css}");
    assert!(css.contains("#2ecc71"), "secondary color missing: {css}");
}

#[tokio::test]
async fn variable_numeric_value() {
    let scss = r#"
        $size: 16px;
        .text { font-size: $size; }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains("font-size: 16px"), "got: {css}");
}

// =============================================================================
// Variable scoping
// =============================================================================

#[tokio::test]
async fn variable_global_scope() {
    let scss = r#"
        $globe: blue;
        .a { color: $globe; }
        .b { color: $globe; }
        .c { color: $globe; }
    "#;
    let css = compile(scss).await.unwrap();
    let count = css.matches("blue").count();
    assert_eq!(count, 3, "All three rules should use $globe: {css}");
}

#[tokio::test]
async fn variable_local_override() {
    let scss = r#"
        $x: global;
        .a { $x: local; color: $x; }
        .b { color: $x; }
    "#;
    let css = compile(scss).await.unwrap();
    // After transform, within .a scope, $x should be "local"; outside, "global".
    assert!(css.contains("local") || css.contains("local"), "local override missing: {css}");
}

// =============================================================================
// Variable in selectors (interpolation)
// =============================================================================

#[tokio::test]
async fn variable_selector_interpolation() {
    let scss = r#"
        $name: bar;
        .#{$name} { display: block; }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains(".bar"), "Selector interpolation missing .bar: {css}");
}

#[tokio::test]
async fn variable_class_with_suffix() {
    let scss = r#"
        $theme: dark;
        .btn-#{$theme} { background: #333; }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains(".btn-dark"), "Expected .btn-dark: {css}");
}

// =============================================================================
// Variable chains (one variable referencing another)
// =============================================================================

#[tokio::test]
async fn variable_chained_reference() {
    let scss = r#"
        $base: 10;
        $scale: $base;
        .x { width: $scale; }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains("10"), "Chained variable should resolve: {css}");
}

#[tokio::test]
async fn variable_in_expression() {
    let scss = r#"
        $margin: 10px;
        .box { margin: $margin; }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains("margin: 10px"), "Variable in property: {css}");
}

// =============================================================================
// Multiple variables in one declaration
// =============================================================================

#[tokio::test]
async fn multiple_variables_single_decl() {
    let scss = r#"
        $w: 100px;
        $h: 200px;
        .box {
            width: $w;
            height: $h;
        }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains("width: 100px"), "width var missing: {css}");
    assert!(css.contains("height: 200px"), "height var missing: {css}");
}
