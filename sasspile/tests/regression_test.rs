//! Regression tests for pipeline eval integration bug.
//!
//! These tests verify that the transform stage correctly processes
//! variables, mixins, and @media nesting — the three core bug areas
//! identified in the fix-pipeline-eval-integration change.

use sasspile::Compiler;

/// Helper: compile SCSS source to CSS expanded output.
async fn compile(scss: &str) -> Result<String, sasspile::SassError> {
    let compiler = Compiler::new();
    compiler.compile(scss).await
}

// =============================================================================
// Test 1: Variable substitution
// =============================================================================

#[tokio::test]
async fn variable_substitution() {
    let scss = r#"
        $primary: #3498db;
        .btn {
            color: $primary;
            background: $primary;
        }
    "#;
    let css = compile(scss).await.unwrap();

    // After transform, $primary should be replaced with #3498db.
    assert!(
        css.contains("#3498db"),
        "Expected #3498db in output, got: {css}"
    );
    assert!(
        !css.contains("$primary"),
        "Variable $primary should be resolved, got: {css}"
    );
}

// =============================================================================
// Test 2: BEM mixin with @include and @content
// =============================================================================

#[tokio::test]
async fn bem_mixin_include() {
    let scss = r#"
        @mixin b($name) {
            .el-#{$name} {
                @content;
            }
        }
        @include b(button) {
            color: red;
        }
    "#;
    let css = compile(scss).await.unwrap();

    // After transform, @include should be expanded.
    assert!(
        css.contains(".el-button"),
        "Expected .el-button in output, got: {css}"
    );
    assert!(
        css.contains("color: red"),
        "Expected 'color: red' in output, got: {css}"
    );
    assert!(
        !css.contains("@include"),
        "@include should be expanded, got: {css}"
    );
}

// =============================================================================
// Test 3: @media nesting with variables
// =============================================================================

#[tokio::test]
async fn media_with_variable() {
    let scss = r#"
        $bp: 768px;
        @media (min-width: $bp) {
            .container {
                width: $bp;
            }
        }
    "#;
    let css = compile(scss).await.unwrap();

    // After transform, variables in @media should be resolved.
    assert!(
        css.contains("768px"),
        "Expected 768px in output, got: {css}"
    );
    assert!(
        !css.contains("$bp"),
        "Variable $bp should be resolved, got: {css}"
    );
    // @media should be present and properly formatted.
    assert!(
        css.contains("@media"),
        "Expected @media in output, got: {css}"
    );
}
