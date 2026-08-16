//! Transform stage — mixin expansion specification tests.
//!
//! Verifies that @mixin/@include directives are correctly expanded
//! during the transform pipeline phase, including parameter passing,
//! @content replacement, and nested rules.

use sasspile::Compiler;

async fn compile(scss: &str) -> Result<String, sasspile::SassError> {
    let compiler = Compiler::new();
    compiler.compile(scss).await
}

// =============================================================================
// Basic @mixin and @include
// =============================================================================

#[tokio::test]
async fn mixin_no_params() {
    let scss = r#"
        @mixin reset {
            margin: 0;
            padding: 0;
        }
        @include reset;
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains("margin: 0"), "margin missing: {css}");
    assert!(css.contains("padding: 0"), "padding missing: {css}");
    assert!(!css.contains("@include"), "@include should be expanded");
}

#[tokio::test]
async fn mixin_with_params() {
    let scss = r#"
        @mixin font($size, $weight) {
            font-size: $size;
            font-weight: $weight;
        }
        @include font(14px, bold);
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains("font-size: 14px"), "font-size missing: {css}");
    // `bold` is a bare identifier → quoted in evaluator.
    assert!(css.contains("font-weight: \"bold\""), "font-weight missing: {css}");
}

#[tokio::test]
async fn mixin_with_default_params() {
    let scss = r#"
        @mixin box($color: blue) {
            color: $color;
        }
        @include box;
    "#;
    let css = compile(scss).await.unwrap();
    // Default param `blue` is a bare identifier → quoted.
    assert!(css.contains("color: \"blue\""), "default param missing: {css}");
}

#[tokio::test]
async fn mixin_with_named_args_omitted() {
    let scss = r#"
        @mixin flex($dir: row) {
            flex-direction: $dir;
        }
        @include flex;
    "#;
    let css = compile(scss).await.unwrap();
    // `row` is a bare identifier → quoted.
    assert!(css.contains("flex-direction: \"row\""), "default should apply: {css}");
}

// =============================================================================
// @content replacement
// =============================================================================

#[tokio::test]
async fn mixin_content_replacement() {
    let scss = r#"
        @mixin wrapper {
            .wrap {
                @content;
            }
        }
        @include wrapper {
            color: blue;
        }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains(".wrap"), "wrapper selector missing: {css}");
    assert!(css.contains("color: blue"), "@content not replaced: {css}");
    assert!(!css.contains("@content"), "@content should be expanded");
}

#[tokio::test]
async fn mixin_content_with_nested_rule() {
    let scss = r#"
        @mixin scope {
            .parent {
                @content;
            }
        }
        @include scope {
            .child { display: block; }
        }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains(".parent"), "parent selector missing: {css}");
    assert!(css.contains(".child"), "child in @content not nested: {css}");
}

// =============================================================================
// Nesting in mixins
// =============================================================================

#[tokio::test]
async fn mixins_nested_rules() {
    let scss = r#"
        @mixin nav {
            ul {
                list-style: none;
                li { display: inline; }
            }
        }
        @include nav;
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains("list-style: none"), "nested ul style missing: {css}");
    assert!(css.contains("inline"), "nested li display missing: {css}");
}

// =============================================================================
// Multiple @include calls
// =============================================================================

#[tokio::test]
async fn multiple_includes_same_mixin() {
    let scss = r#"
        @mixin pad($n) { padding: $n; }
        .a { @include pad(5px); }
        .b { @include pad(10px); }
        .c { @include pad(15px); }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains("padding: 5px"), "first @include missing: {css}");
    assert!(css.contains("padding: 10px"), "second @include missing: {css}");
    assert!(css.contains("padding: 15px"), "third @include missing: {css}");
}
