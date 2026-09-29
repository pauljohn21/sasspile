//! Test @forward with mixin

use std::collections::HashMap;
use sasspile::compile_with_files;

#[test]
fn test_forward_mixin_simple() {
    // Simple test: forward a mixin without prefix
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
    println!("=== forward mixin simple test ===");
    println!("Result:\n{}", result);
    assert!(
        result.contains("d: e"),
        "Expected 'd: e', got: {}",
        result
    );
}

#[test]
fn test_forward_mixin_with_body_on_next_line() {
    // Test mixin with body on next line
    let input = "@use \"midstream\";\n\na {@include midstream.foo}\n";

    let mut files = HashMap::new();
    files.insert(
        "midstream.scss".to_string(),
        "@forward \"upstream\";".to_string(),
    );
    files.insert(
        "upstream.scss".to_string(),
        "@mixin foo\n  c\n    d: e".to_string(),
    );

    let result = compile_with_files(input, &files);
    println!("=== forward mixin body on next line test ===");
    println!("Result:\n{}", result);
}
