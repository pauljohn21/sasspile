//! Evaluator tests — tests end-to-end SCSS compilation.

fn compile_src(src: &str) -> String {
    sasspile::compile(src).expect("compilation should succeed")
}

#[test]
fn test_compile_simple_rule() {
    let css = compile_src("a { color: red; }");
    assert!(css.contains("a {"));
    assert!(css.contains("color: red"));
}

#[test]
fn test_compile_variable() {
    let css = compile_src("$color: red; a { color: $color; }");
    assert!(css.contains("color: red"));
}

#[test]
fn test_compile_arithmetic() {
    let css = compile_src("a { width: 10px + 20px; }");
    assert!(css.contains("30px"));
}

#[test]
fn test_compile_nested_rules() {
    let css = compile_src(".parent { color: red; .child { color: blue; } }");
    assert!(css.contains(".parent {"));
    assert!(css.contains(".parent .child {"));
}

#[test]
fn test_compile_parent_selector() {
    let css = compile_src("a { &:hover { color: red; } }");
    assert!(css.contains("a:hover"));
}

#[test]
fn test_compile_mixin() {
    let scss = "@mixin red { color: red; } a { @include red; }";
    let css = compile_src(scss);
    assert!(css.contains("color: red"));
}

#[test]
fn test_compile_mixin_with_params() {
    let scss = "@mixin color($c) { color: $c; } a { @include color(blue); }";
    let css = compile_src(scss);
    assert!(css.contains("color: blue"));
}

#[test]
fn test_compile_function() {
    let scss = "@function double($n) { @return $n * 2; } a { width: double(5px); }";
    let result = sasspile::compile(scss);
    if let Ok(css) = result {
        tracing::info!(stage = "test", css = %css, "function test output");
    }
}

#[test]
fn test_compile_if() {
    let scss = "@if true { a { color: red; } }";
    let css = compile_src(scss);
    assert!(css.contains("a {"));
    assert!(css.contains("color: red"));
}

#[test]
fn test_compile_for() {
    let scss = "@for $i from 1 through 3 { .item-#{$i} { width: $i; } }";
    let css = compile_src(scss);
    assert!(css.contains(".item-1"), "should contain .item-1: {}", css);
    assert!(css.contains(".item-2"), "should contain .item-2: {}", css);
    assert!(css.contains(".item-3"), "should contain .item-3: {}", css);
}

#[test]
fn test_compile_each() {
    let scss = "@each $c in red, blue { .x { color: $c; } }";
    let css = compile_src(scss);
    assert!(css.contains("red"));
    assert!(css.contains("blue"));
}

#[test]
fn test_compile_hex_color() {
    let css = compile_src("a { color: #fff; }");
    assert!(css.contains("#fff"));
}

#[test]
fn test_compile_multiple_declarations() {
    let css = compile_src("a { color: red; background: blue; }");
    assert!(css.contains("color: red"));
    assert!(css.contains("background: blue"));
}

#[test]
fn test_compile_empty_rule() {
    let css = compile_src("a { }");
    assert!(!css.contains("a {"));
}

#[test]
fn test_compile_comment() {
    let css = compile_src("/* hello */");
    assert!(css.contains("hello"));
}

#[test]
fn test_compile_media() {
    let css = compile_src("@media screen { a { color: red; } }");
    assert!(css.contains("@media"));
}

#[test]
fn test_compile_if_else() {
    let scss = "@if false { a { color: red; } } @else { b { color: blue; } }";
    let css = compile_src(scss);
    assert!(!css.contains("color: red"));
    assert!(css.contains("b {"));
    assert!(css.contains("color: blue"));
}

#[test]
fn test_compile_while() {
    let scss = "$i: 1; @while $i <= 3 { .item-#{$i} { width: $i; } $i: $i + 1; }";
    let css = compile_src(scss);
    assert!(css.contains(".item-1"), "should contain .item-1: {}", css);
    assert!(css.contains(".item-2"), "should contain .item-2: {}", css);
    assert!(css.contains(".item-3"), "should contain .item-3: {}", css);
}

#[test]
fn test_compile_mixin_content() {
    let scss = "@mixin wrapper { .inner { @content; } } @include wrapper { color: red; }";
    let css = compile_src(scss);
    assert!(css.contains(".inner"), "should contain .inner: {}", css);
    assert!(css.contains("color: red"), "should contain color: red: {}", css);
}

#[test]
fn test_compile_supports() {
    let scss = "@supports (display: flex) { .flex { display: flex; } }";
    let css = compile_src(scss);
    assert!(css.contains("@supports"), "should contain @supports: {}", css);
    assert!(css.contains("display: flex"), "should contain display: flex: {}", css);
}

#[test]
fn test_compile_extend() {
    let scss = ".foo { color: red; } .bar { @extend .foo; }";
    let css = compile_src(scss);
    // @extend should produce .foo, .bar { color: red; }
    assert!(css.contains("color: red"), "should contain color: red: {}", css);
    assert!(css.contains(".bar"), "should contain .bar: {}", css);
}

#[test]
fn test_compile_selector_interpolation() {
    let scss = "$name: foo; .#{$name} { color: red; }";
    let css = compile_src(scss);
    assert!(css.contains(".foo {"), "should contain .foo: {}", css);
    assert!(css.contains("color: red"), "should contain color: red: {}", css);
}

#[test]
fn test_compile_property_interpolation() {
    let scss = "$prop: color; a { #{$prop}: red; }";
    let css = compile_src(scss);
    assert!(css.contains("color: red"), "should contain color: red: {}", css);
}

#[test]
fn test_compile_at_root() {
    let scss = ".parent { @at-root { .child { color: red; } } }";
    let css = compile_src(scss);
    assert!(css.contains(".child"), "should contain .child: {}", css);
    // @at-root should NOT prefix with .parent
    assert!(!css.contains(".parent .child"), "should not contain nested .parent .child: {}", css);
}

#[test]
fn test_compile_variable_default() {
    let scss = "$color: red; $color: blue !default; a { color: $color; }";
    let css = compile_src(scss);
    assert!(css.contains("color: red"), "!default should not override: {}", css);
}

#[test]
fn test_compile_variable_global() {
    let scss = ".parent { $color: red !global; } a { color: $color; }";
    let css = compile_src(scss);
    assert!(css.contains("color: red"), "!global should be visible: {}", css);
}

#[test]
fn test_compile_compressed() {
    use sasspile::{serialize_with_style, OutputStyle};
    // Use the internal API for compressed mode
    let scss = "a { color: red; }";
    let source = scss;
    let tokens = sasspile::tokenize(source, "test_compressed").unwrap();
    let ast = sasspile::parse(tokens).unwrap();
    let css_tree = sasspile::evaluate(ast, &mut sasspile::FileResolver::new()).unwrap();
    let compressed = serialize_with_style(&css_tree, OutputStyle::Compressed).unwrap();
    assert!(compressed.contains("a{color:red}"), "compressed output: {}", compressed);
}

#[test]
fn test_compile_error_stmt() {
    let scss = "@error \"test error\";";
    let result = sasspile::compile(scss);
    assert!(result.is_err(), "should error: {:?}", result);
}

#[test]
fn test_compile_debug_stmt() {
    let scss = "@debug \"test\"; a { color: red; }";
    let css = compile_src(scss);
    assert!(css.contains("color: red"), "should still output: {}", css);
}

#[test]
fn test_compile_warn_stmt() {
    let scss = "@warn \"test warning\"; a { color: red; }";
    let css = compile_src(scss);
    assert!(css.contains("color: red"), "should still output: {}", css);
}
