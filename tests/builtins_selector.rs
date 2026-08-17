//! Selector builtins tests — tests for sass:selector functions.

use sasspile::compile;

fn compile_src(src: &str) -> String {
    compile(src).expect("compilation should succeed")
}

#[test]
fn test_selector_append() {
    let css = compile_src("a { c: selector-append(\".foo\", \".bar\"); }");
    assert!(css.contains(".foo.bar"), "selector-append should produce .foo.bar, got: {}", css);
}

#[test]
fn test_selector_nest() {
    let css = compile_src("a { c: selector-nest(\".foo\", \".bar\"); }");
    assert!(css.contains(".foo .bar"), "selector-nest should produce .foo .bar, got: {}", css);
}

#[test]
fn test_selector_is_superselector() {
    let css = compile_src("a { c: selector-is-superselector(\".foo\", \".foo.bar\"); }");
    assert!(css.contains("true"), "is-superselector should be true, got: {}", css);
}

#[test]
fn test_selector_unify() {
    let css = compile_src("a { c: selector-unify(\".foo\", \".bar\"); }");
    assert!(css.contains(".foo.bar"), "selector-unify should produce .foo.bar, got: {}", css);
}
