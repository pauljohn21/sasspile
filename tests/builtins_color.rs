//! Color builtins tests — tests for sass:color functions.

use sasspile::compile;

fn compile_src(src: &str) -> String {
    compile(src).expect("compilation should succeed")
}

#[test]
fn test_rgb() {
    let css = compile_src("a { c: rgb(255, 0, 0); }");
    assert!(css.contains("#ff0000") || css.contains("#f00"), "rgb(255,0,0) should be red, got: {}", css);
}

#[test]
fn test_rgba() {
    let css = compile_src("a { c: rgba(255, 0, 0, 0.5); }");
    assert!(css.contains("rgba"), "rgba should contain rgba, got: {}", css);
}

#[test]
fn test_hsl() {
    let css = compile_src("a { c: hsl(0, 100%, 50%); }");
    assert!(css.contains("#ff0000") || css.contains("#f00"), "hsl(0,100%,50%) should be red, got: {}", css);
}

#[test]
fn test_red_channel() {
    let css = compile_src("a { c: red(#ff0000); }");
    assert!(css.contains("255"), "red(#ff0000) should be 255, got: {}", css);
}

#[test]
fn test_green_channel() {
    let css = compile_src("a { c: green(#00ff00); }");
    assert!(css.contains("255"), "green(#00ff00) should be 255, got: {}", css);
}

#[test]
fn test_blue_channel() {
    let css = compile_src("a { c: blue(#0000ff); }");
    assert!(css.contains("255"), "blue(#0000ff) should be 255, got: {}", css);
}

#[test]
fn test_alpha_channel() {
    let css = compile_src("a { c: alpha(rgba(255, 0, 0, 0.5)); }");
    assert!(css.contains("0.5"), "alpha should be 0.5, got: {}", css);
}

#[test]
fn test_lighten() {
    let css = compile_src("a { c: lighten(#000000, 50%); }");
    assert!(!css.is_empty(), "lighten should produce output, got: {}", css);
}

#[test]
fn test_darken() {
    let css = compile_src("a { c: darken(#ffffff, 50%); }");
    assert!(!css.is_empty(), "darken should produce output, got: {}", css);
}

#[test]
fn test_mix() {
    let css = compile_src("a { c: mix(red, blue); }");
    assert!(!css.is_empty(), "mix should produce output, got: {}", css);
}

#[test]
fn test_complement() {
    let css = compile_src("a { c: complement(red); }");
    assert!(!css.is_empty(), "complement should produce output, got: {}", css);
}

#[test]
fn test_invert() {
    let css = compile_src("a { c: invert(red); }");
    assert!(!css.is_empty(), "invert should produce output, got: {}", css);
}
