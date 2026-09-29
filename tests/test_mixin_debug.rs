//! Debug test for mixin expansion

#[test]
fn test_mixin_expand_debug() {
    // Test direct mixin
    let input = "@mixin a() {b {c: d}}\n\n@include a;\n";
    let result = sasspile::compile(input);
    println!("=== direct mixin ===");
    println!("Result: [{}]", result);
    println!("Contains 'c: d': {}", result.contains("c: d"));
}

#[test]
fn test_mixin_expand_debug2() {
    // Test direct mixin without parens
    let input = "@mixin foo {c {d: e}}\n\n@include foo;\n";
    let result = sasspile::compile(input);
    println!("=== direct mixin no parens ===");
    println!("Result: [{}]", result);
    println!("Contains 'd: e': {}", result.contains("d: e"));
}

#[test]
fn test_mixin_expand_debug3() {
    // Test multi-line mixin
    let input = "@mixin foo\n  c\n    d: e\n\n@include foo;\n";
    let result = sasspile::compile(input);
    println!("=== multi-line mixin ===");
    println!("Result: [{}]", result);
    println!("Contains 'd: e': {}", result.contains("d: e"));
}
