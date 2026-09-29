//! Test @forward "url" as prefix-* (转发前缀)

#[test]
fn test_simple_optional_extend() {
    let input = ".bar { color: red; }";
    let result = sasspile::compile(input);
    eprintln!("SIMPLE RULE TEST: {:?}", result);
    assert!(result.contains(".bar"), "expected .bar in output: {}", result);
}

#[test]
fn test_optional_extend_only() {
    let input = ".bar { @extend %undefined !optional; color: red; }";
    let result = sasspile::compile(input);
    eprintln!("OPTIONAL EXTEND TEST: {:?}", result);
    assert!(result.contains(".bar"), "expected .bar in output: {}", result);
}

use std::collections::HashMap;
use sasspile::compile_with_files;

#[test]
fn test_forward_prefix_variable() {
    // Scenario: @forward "upstream" as d-* should prefix members with "d-"
    let input = "@use \"midstream\";\n\na {b: midstream.$d-c}\n";

    let mut files = HashMap::new();
    files.insert(
        "midstream.scss".to_string(),
        "@forward \"upstream\" as d-*;".to_string(),
    );
    files.insert("upstream.scss".to_string(), "$c: e;".to_string());

    let result = compile_with_files(input, &files);
    println!("=== forward prefix variable test ===");
    println!("Result:\n{}", result);
    assert!(
        result.contains("b: e"),
        "Expected 'b: e', got: {}",
        result
    );
}

#[test]
fn test_forward_prefix_function() {
    // Scenario: @forward "upstream" as d-* should prefix function names with "d-"
    let input = "@use \"midstream\";\n\na {b: midstream.d-c()}\n";

    let mut files = HashMap::new();
    files.insert(
        "midstream.scss".to_string(),
        "@forward \"upstream\" as d-*;".to_string(),
    );
    files.insert(
        "upstream.scss".to_string(),
        "@function c() {@return e}".to_string(),
    );

    let result = compile_with_files(input, &files);
    println!("=== forward prefix function test ===");
    println!("Result:\n{}", result);
    assert!(
        result.contains("b: e"),
        "Expected 'b: e', got: {}",
        result
    );
}

#[test]
fn test_forward_prefix_mixin() {
    // Scenario: @forward "upstream" as b-* should prefix mixin names with "b-"
    let input = "@use \"midstream\";\n\na {@include midstream.b-a}\n";

    let mut files = HashMap::new();
    files.insert(
        "midstream.scss".to_string(),
        "@forward \"upstream\" as b-*;".to_string(),
    );
    files.insert(
        "upstream.scss".to_string(),
        "@mixin a() {c {d: e}}".to_string(),
    );

    let result = compile_with_files(input, &files);
    println!("=== forward prefix mixin test ===");
    println!("Result:\n{}", result);
    assert!(
        result.contains("d: e"),
        "Expected 'd: e', got: {}",
        result
    );
}
