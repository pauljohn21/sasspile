//! String builtins tests — tests for sass:string functions.

use sasspile::compile;

fn compile_src(src: &str) -> String {
    compile(src).expect("compilation should succeed")
}

#[test]
fn test_quote() {
    let css = compile_src("a { c: quote(hello); }");
    assert!(css.contains("\"hello\""), "quote(hello) should be \"hello\", got: {}", css);
}

#[test]
fn test_unquote() {
    let css = compile_src("a { c: unquote(\"hello\"); }");
    assert!(css.contains("hello") && !css.contains("\"hello\""), "unquote should remove quotes, got: {}", css);
}

#[test]
fn test_to_upper_case() {
    let css = compile_src("a { c: to-upper-case(\"hello\"); }");
    assert!(css.contains("HELLO"), "to-upper-case should produce HELLO, got: {}", css);
}

#[test]
fn test_to_lower_case() {
    let css = compile_src("a { c: to-lower-case(\"HELLO\"); }");
    assert!(css.contains("hello"), "to-lower-case should produce hello, got: {}", css);
}

#[test]
fn test_string_length() {
    let css = compile_src("a { c: str-length(\"hello\"); }");
    assert!(css.contains("5"), "str-length(\"hello\") should be 5, got: {}", css);
}

#[test]
fn test_string_index() {
    let css = compile_src("a { c: str-index(\"hello\", \"ll\"); }");
    assert!(css.contains("3"), "str-index(\"hello\",\"ll\") should be 3, got: {}", css);
}

#[test]
fn test_string_insert() {
    let css = compile_src("a { c: str-insert(\"hello\", \"x\", 2); }");
    assert!(css.contains("hxello"), "str-insert should produce hxello, got: {}", css);
}

#[test]
fn test_string_slice() {
    let css = compile_src("a { c: str-slice(\"hello\", 2, 4); }");
    assert!(css.contains("ell"), "str-slice(\"hello\",2,4) should be ell, got: {}", css);
}
