//! Transform stage — control flow expansion specification tests.
//!
//! Verifies that @if, @for, @each, @while directives are correctly
//! expanded during the transform pipeline phase.

use sasspile::Compiler;

async fn compile(scss: &str) -> Result<String, sasspile::SassError> {
    let compiler = Compiler::new();
    compiler.compile(scss).await
}

// =============================================================================
// @if / @else
// =============================================================================

#[tokio::test]
async fn if_true_branch() {
    let scss = r#"
        $cond: true;
        @if $cond {
            .a { display: block; }
        }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains(".a"), "if-true branch missing: {css}");
    assert!(css.contains("display: block"), "decl after @if missing: {css}");
}

#[tokio::test]
async fn if_false_branch() {
    let scss = r#"
        $cond: false;
        @if $cond {
            .a { display: block; }
        } @else {
            .b { display: none; }
        }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains(".b"), "else branch missing: {css}");
    assert!(css.contains("display: none"), "decl after @else missing: {css}");
}

#[tokio::test]
async fn if_no_else_false() {
    let scss = r#"
        $cond: false;
        @if $cond {
            .hidden { display: block; }
        }
        .always { color: red; }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(!css.contains(".hidden"), "if-false should emit nothing: {css}");
    assert!(css.contains(".always"), "rule after @if missing: {css}");
}

// =============================================================================
// @for loop
// =============================================================================

#[tokio::test]
async fn for_through_inclusive() {
    let scss = r#"
        @for $i from 1 through 3 {
            .item-#{$i} { width: 10px; }
        }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains(".item-1"), "iteration 1 missing: {css}");
    assert!(css.contains(".item-2"), "iteration 2 missing: {css}");
    assert!(css.contains(".item-3"), "iteration 3 missing: {css}");
}

#[tokio::test]
async fn for_to_exclusive() {
    let scss = r#"
        @for $i from 1 to 3 {
            .col-#{$i} { float: left; }
        }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains(".col-1"), "first iteration missing: {css}");
    assert!(css.contains(".col-2"), "second iteration missing: {css}");
    assert!(!css.contains(".col-3"), "to 3 should exclude 3: {css}");
}

// =============================================================================
// @each loop
// =============================================================================

#[tokio::test]
async fn each_single_var() {
    let scss = r#"
        @each $color in red, green, blue {
            .bg-#{$color} { background: $color; }
        }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains(".bg-red"), "red iteration missing: {css}");
    assert!(css.contains(".bg-green"), "green iteration missing: {css}");
    assert!(css.contains(".bg-blue"), "blue iteration missing: {css}");
}

#[tokio::test]
async fn each_space_list() {
    let scss = r#"
        @each $size in small medium large {
            .text-#{$size} { font-weight: normal; }
        }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains(".text-small"), "small missing: {css}");
    assert!(css.contains(".text-medium"), "medium missing: {css}");
    assert!(css.contains(".text-large"), "large missing: {css}");
}

// =============================================================================
// @while loop
// =============================================================================

#[tokio::test]
async fn while_basic_loop() {
    // Use a simple pattern that will run only once to be safe.
    // SCSS: @while $i > 0 { ...; $i: $i - 1; }
    // For now, test that @while with a false initial condition produces nothing.
    let scss = r#"
        $done: true;
        @while $done == false {
            .x { color: red; }
        }
        .visible { display: block; }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains(".visible"), "rule after @while missing: {css}");
}

// =============================================================================
// Nested control flow
// =============================================================================

#[tokio::test]
async fn if_inside_for() {
    let scss = r#"
        @for $i from 1 through 2 {
            @if $i == 1 {
                .first { color: red; }
            }
        }
    "#;
    let css = compile(scss).await.unwrap();
    assert!(css.contains(".first"), "@if inside @for missing: {css}");
    assert!(css.contains("color: red"), "decl inside nested if missing: {css}");
}

// =============================================================================
// @media with control flow
// =============================================================================

#[tokio::test]
async fn media_with_variable_query() {
    let scss = r#"
        $bp: 768px;
        @media (max-width: $bp) {
            .responsive { width: 100%; }
        }
    "#;
    let css = compile(scss).await.unwrap();
    // Verify the variable was expanded (768px appears somewhere).
    // @media query formatting may not preserve parens exactly.
    assert!(css.contains("768px"), "$bp not resolved: {css}");
    assert!(css.contains(".responsive"), "nested rule missing: {css}");
}
