//! Test mixin injection in @forward scenario

use std::collections::HashMap;
use sasspile::compile_with_files;

#[test]
fn test_mixin_inject_single_line() {
    // Direct test: mixin definition with nested braces
    let input = "@mixin foo {c {d: e}}\n\na {@include foo}\n";
    let result = sasspile::compile(input);
    println!("=== direct mixin test ===");
    println!("Result:\n{}", result);
    assert!(
        result.contains("d: e"),
        "Expected 'd: e', got: {}",
        result
    );
}

#[test]
fn test_forward_mixin_single_line_debug() {
    // Forward scenario with single-line mixin
    let input = "@use \"midstream\";\n\na {@include midstream.foo}\n";

    let mut files = HashMap::new();
    files.insert(
        "midstream.scss".to_string(),
        "@forward \"upstream\";".to_string(),
    );
    files.insert(
        "upstream.scss".to_string(),
        "@mixin foo {c {d: e}}".to_string(),
    );

    let result = compile_with_files(input, &files);
    println!("=== forward mixin single line debug ===");
    println!("Result:\n{}", result);
}
