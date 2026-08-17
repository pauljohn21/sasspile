//! Map builtins tests — tests for sass:map functions.

use sasspile::compile;

fn compile_src(src: &str) -> String {
    compile(src).expect("compilation should succeed")
}

#[test]
fn test_map_get() {
    let css = compile_src("a { c: map-get((a: 1, b: 2), b); }");
    assert!(css.contains("2"), "map-get should return 2, got: {}", css);
}

#[test]
fn test_map_has_key() {
    let css = compile_src("a { c: map-has-key((a: 1, b: 2), b); }");
    assert!(css.contains("true"), "map-has-key should be true, got: {}", css);
}

#[test]
fn test_map_keys() {
    let css = compile_src("a { c: map-keys((a: 1, b: 2)); }");
    assert!(css.contains("a"), "map-keys should contain a, got: {}", css);
    assert!(css.contains("b"), "map-keys should contain b, got: {}", css);
}

#[test]
fn test_map_values() {
    let css = compile_src("a { c: map-values((a: 1, b: 2)); }");
    assert!(css.contains("1"), "map-values should contain 1, got: {}", css);
    assert!(css.contains("2"), "map-values should contain 2, got: {}", css);
}

#[test]
fn test_map_merge() {
    let css = compile_src("$m: map-merge((a: 1, b: 2), (b: 3, c: 4)); a { c: map-get($m, b); d: map-get($m, c); }");
    assert!(css.contains("3"), "map-merge should update b to 3, got: {}", css);
    assert!(css.contains("4"), "map-merge should add c=4, got: {}", css);
}
