use sasspile::eval::reactor::Reactor;
use sasspile::OutputStyle;

fn parse(input: &str) -> String {
    Reactor::new(input)
        .lex()
        .expect("lex failed")
        .parse()
        .expect("parse failed")
        .evaluate()
        .expect("eval failed")
        .serialize(OutputStyle::Expanded)
        .finish()
        .expect("finish failed")
}

#[test]
fn test_parse_rule() {
    let css = parse("a { color: red; }");
    assert!(css.contains("color: red"), "got: {css}");
    assert!(css.contains("a {"), "got: {css}");
}

#[test]
fn test_parse_variable() {
    let css = parse("$w: 10px; .x { width: $w; }");
    assert!(css.contains("width: 10px"), "got: {css}");
}

#[test]
fn test_parse_if() {
    let css = parse("@if true { a { color: red; } }");
    assert!(css.contains("color: red"), "got: {css}");
}

#[test]
fn test_parse_mixin() {
    let css = parse("@mixin foo($x) { color: $x; } @include foo(blue);");
    assert!(css.contains("color: blue"), "got: {css}");
}

#[test]
fn test_parse_expr_precedence() {
    // $x: 1 + 2 * 3; → $x = 7 → width: 7px
    let css = parse("$x: 1 + 2 * 3; .x { width: $x * 1px; }");
    assert!(css.contains("width: 7px"), "got: {css}");
}

#[test]
fn test_parse_string_interp() {
    let css = parse("$name: world; .x { content: \"hello #{$name}\"; }");
    assert!(css.contains("hello world"), "got: {css}");
}

#[test]
fn test_parse_each() {
    let css = parse("@each $i in 1, 2, 3 { .x-#{$i} { width: $i px; } }");
    assert!(css.contains(".x-1"), "got: {css}");
    assert!(css.contains(".x-3"), "got: {css}");
}

#[test]
fn test_parse_for() {
    let css = parse("@for $i from 1 through 3 { .x-#{$i} { --i: $i; } }");
    assert!(css.contains(".x-3 {"), "got: {css}");
}

#[test]
fn test_parse_while() {
    let css = parse("$i: 1; @while $i <= 3 { .x-#{$i} { --i: $i; } $i: $i + 1; }");
    assert!(css.contains(".x-3"), "got: {css}");
}
