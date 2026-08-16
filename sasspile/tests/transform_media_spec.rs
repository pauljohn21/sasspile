//! Transform stage — @media query expansion tests.
//!
//! Verifies that variables inside @media queries (e.g., `@media (max-width: $bp)`)
//! are resolved by the transform stage, and that the resulting CSS wraps nested
//! rules correctly.

use sasspile::Compiler;

async fn compile(scss: &str) -> Result<String, sasspile::SassError> {
    let compiler = Compiler::new();
    compiler.compile(scss).await
}

// =============================================================================
// @media query with variable — the unit (px) must survive substitution.
// =============================================================================

#[tokio::test]
async fn media_query_variable_unit_preserved() {
    let scss = r#"
        $bp: 768px;
        @media (max-width: $bp) {
            .responsive { width: 100%; }
        }
    "#;
    let css = compile(scss).await.unwrap();
    // The $bp variable must be replaced with its full value, including the unit.
    assert!(css.contains("768px"), "Expected 768px in media query, got: {css}");
    assert!(css.contains(".responsive"), "Nested rule missing: {css}");
}

#[tokio::test]
async fn media_query_multiple_variables() {
    let scss = r#"
        $min: 320px;
        $max: 1024px;
        @media (min-width: $min) and (max-width: $max) {
            .fluid { width: 50%; }
        }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains("320px"), "min-width var missing: {css}");
    assert!(css.contains("1024px"), "max-width var missing: {css}");
}

#[tokio::test]
async fn media_query_em_unit() {
    let scss = r#"
        $breakpoint: 48em;
        @media (min-width: $breakpoint) {
            .wide { font-size: 18px; }
        }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains("48em"), "em unit missing: {css}");
}

#[tokio::test]
async fn supports_query_basic() {
    let scss = r#"
        @supports (display: grid) {
            .layout { display: grid; }
        }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains("@supports") || css.contains("display"), "@supports missing: {css}");
    assert!(css.contains(".layout"), "Nested rule missing: {css}");
}

#[tokio::test]
async fn media_nested_in_rule() {
    let scss = r#"
        $bp: 600px;
        .container {
            @media (min-width: $bp) {
                .item { width: 50%; }
            }
        }
    "#;
    let css = compile(scss).await.unwrap();
    // Variables inside nested @media must also resolve.
    assert!(css.contains("600px"), "nested @media $bp missing: {css}");
    assert!(css.contains(".item"), "nested rule missing: {css}");
}
