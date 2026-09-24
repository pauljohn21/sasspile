//! Phase D.2 — 内置颜色函数

use sasspile::compile;

#[test]
fn rgb_to_hex() {
    let css = compile(".a { color: rgb(255, 0, 0); }");
    assert!(css.contains("#ff0000"), "got: {css}");
}

#[test]
fn rgb_mixed_values() {
    let css = compile(".a { color: rgb(128, 64, 32); }");
    assert!(css.contains("#804020"), "got: {css}");
}

#[test]
fn hsl_red() {
    // hsl(0, 100%, 50%) → #ff0000
    let css = compile(".a { color: hsl(0, 100%, 50%); }");
    assert!(css.contains("#ff0000"), "got: {css}");
}

#[test]
fn hsl_green() {
    // hsl(120, 100%, 50%) → #00ff00
    let css = compile(".a { color: hsl(120, 100%, 50%); }");
    assert!(css.contains("#00ff00"), "got: {css}");
}

#[test]
fn hsl_blue() {
    // hsl(240, 100%, 50%) → #0000ff
    let css = compile(".a { color: hsl(240, 100%, 50%); }");
    assert!(css.contains("#0000ff"), "got: {css}");
}

#[test]
fn rgb_in_variable() {
    let scss = "$c: rgb(0, 128, 255);\n.a { color: $c; }";
    let css = compile(scss);
    assert!(css.contains("#0080ff"), "got: {css}");
}

#[test]
fn multiple_rgb_in_one_line() {
    let scss = ".a { background: rgb(255, 255, 0); border: rgb(0, 0, 0); }";
    let css = compile(scss);
    assert!(css.contains("#ffff00"), "bg: {css}");
    assert!(css.contains("#000000"), "border: {css}");
}

#[test]
fn rgb_white() {
    let css = compile(".a { color: rgb(255, 255, 255); }");
    assert!(css.contains("#ffffff"), "got: {css}");
}

#[test]
fn rgb_black() {
    let css = compile(".a { color: rgb(0, 0, 0); }");
    assert!(css.contains("#000000"), "got: {css}");
}
