//! Math builtins tests — tests for sass:math functions.

use sasspile::compile;

fn compile_src(src: &str) -> String {
    compile(src).expect("compilation should succeed")
}

#[test]
fn test_abs() {
    let css = compile_src("a { w: abs(-5); }");
    assert!(css.contains("5"), "abs(-5) should be 5, got: {}", css);
}

#[test]
fn test_ceil() {
    let css = compile_src("a { w: ceil(3.2); }");
    assert!(css.contains("4"), "ceil(3.2) should be 4, got: {}", css);
}

#[test]
fn test_floor() {
    let css = compile_src("a { w: floor(3.8); }");
    assert!(css.contains("3"), "floor(3.8) should be 3, got: {}", css);
}

#[test]
fn test_round() {
    let css = compile_src("a { w: round(3.6); }");
    assert!(css.contains("4"), "round(3.6) should be 4, got: {}", css);
    let css = compile_src("a { w: round(3.4); }");
    assert!(css.contains("3"), "round(3.4) should be 3, got: {}", css);
}

#[test]
fn test_max() {
    let css = compile_src("a { w: max(1, 5, 3); }");
    assert!(css.contains("5"), "max(1,5,3) should be 5, got: {}", css);
}

#[test]
fn test_min() {
    let css = compile_src("a { w: min(1, 5, 3); }");
    assert!(css.contains("1"), "min(1,5,3) should be 1, got: {}", css);
}

#[test]
fn test_percentage() {
    let css = compile_src("a { w: math-percentage(0.5); }");
    assert!(css.contains("50%"), "percentage(0.5) should be 50%, got: {}", css);
}

#[test]
fn test_unit() {
    let css = compile_src("a { w: math-unit(10px); }");
    assert!(css.contains("px"), "unit(10px) should be px, got: {}", css);
}

#[test]
fn test_unitless() {
    let css = compile_src("a { w: math-unitless(10); }");
    assert!(css.contains("true"), "unitless(10) should be true, got: {}", css);
}

#[test]
fn test_sqrt() {
    let css = compile_src("a { w: math-sqrt(9); }");
    assert!(css.contains("3"), "sqrt(9) should be 3, got: {}", css);
}

#[test]
fn test_pow() {
    let css = compile_src("a { w: math-pow(2, 3); }");
    assert!(css.contains("8"), "pow(2,3) should be 8, got: {}", css);
}

#[test]
fn test_pi_var() {
    let css = compile_src("a { w: $pi; }");
    assert!(css.contains("3.14"), "$pi should contain 3.14, got: {}", css);
}
