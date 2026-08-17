//! List builtins tests — tests for sass:list functions.

use sasspile::compile;

fn compile_src(src: &str) -> String {
    compile(src).expect("compilation should succeed")
}

#[test]
fn test_list_length() {
    let css = compile_src("a { c: length(1 2 3); }");
    assert!(css.contains("3"), "length(1 2 3) should be 3, got: {}", css);
}

#[test]
fn test_list_nth() {
    let css = compile_src("a { c: nth(1 2 3, 2); }");
    assert!(css.contains("2"), "nth(1 2 3, 2) should be 2, got: {}", css);
}

#[test]
fn test_list_join() {
    let css = compile_src("a { c: join((1, 2), (3, 4)); }");
    assert!(css.contains("1, 2, 3, 4"), "join should produce 1, 2, 3, 4, got: {}", css);
}

#[test]
fn test_list_append() {
    let css = compile_src("a { c: append(1 2, 3); }");
    assert!(css.contains("1 2 3"), "append should produce 1 2 3, got: {}", css);
}

#[test]
fn test_list_index() {
    let css = compile_src("a { c: index(1 2 3, 2); }");
    assert!(css.contains("2"), "index(1 2 3, 2) should be 2, got: {}", css);
}

#[test]
fn test_list_separator() {
    let css = compile_src("a { c: list-separator(1, 2, 3); }");
    assert!(css.contains("comma"), "separator should be comma, got: {}", css);
}

#[test]
fn test_list_is_bracketed() {
    let css = compile_src("a { c: is-bracketed([1 2 3]); }");
    assert!(css.contains("true"), "is-bracketed should be true, got: {}", css);
}
