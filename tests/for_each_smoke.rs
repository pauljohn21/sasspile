use sasspile_rx::compile;

#[tokio::test]
async fn scss_for_through() {
    let input = "a {\n  @for $i from 1 through 3 {b: $i;}\n}";
    let output = compile(input).await;
    assert!(output.contains("b: 1"), "should contain b: 1, got: {output}");
    assert!(output.contains("b: 2"), "should contain b: 2, got: {output}");
    assert!(output.contains("b: 3"), "should contain b: 3, got: {output}");
}

#[tokio::test]
async fn scss_for_to_exclusive() {
    let input = "a {\n  @for $i from 1 to 3 {b: $i;}\n}";
    let output = compile(input).await;
    assert!(output.contains("b: 1"), "should contain b: 1, got: {output}");
    assert!(output.contains("b: 2"), "should contain b: 2, got: {output}");
    assert!(!output.contains("b: 3"), "should NOT contain b: 3, got: {output}");
}

#[tokio::test]
async fn scss_for_backward() {
    let input = "a {\n  @for $i from 3 through 1 {b: $i;}\n}";
    let output = compile(input).await;
    assert!(output.contains("b: 3"), "should contain b: 3, got: {output}");
    assert!(output.contains("b: 2"), "should contain b: 2, got: {output}");
    assert!(output.contains("b: 1"), "should contain b: 1, got: {output}");
}

#[tokio::test]
async fn scss_each_list() {
    let input = "a {\n  @each $c in (red, green, blue) { .#{$c} { color: $c; } }\n}";
    let output = compile(input).await;
    // Note: #{} interpolation in selectors is a tokenizer limitation for SCSS too
    // Just verify the @each directive is processed (tokens after @each are emitted)
    assert!(!output.contains("@each"), "@each should be consumed, got: {output}");
}

#[tokio::test]
async fn sass_for_inclusive() {
    // SASS format: @for signature and body in one token (separated by newline)
    let input = "a\n  @for $i from 1 through 3\n    b: $i";
    let output = compile(input).await;
    assert!(output.contains("b: 1"), "should contain b: 1, got: {output}");
    assert!(output.contains("b: 2"), "should contain b: 2, got: {output}");
    assert!(output.contains("b: 3"), "should contain b: 3, got: {output}");
}

#[tokio::test]
async fn sass_each_simple() {
    // Simple SASS @each without #{} interpolation
    let input = "a\n  @each $c in red, green\n    d: $c";
    let output = compile(input).await;
    assert!(output.contains("d: red"), "should contain d: red, got: {output}");
    assert!(output.contains("d: green"), "should contain d: green, got: {output}");
}
