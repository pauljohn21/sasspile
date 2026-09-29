//! Test @use + @import chain (variable_use scenario)

use std::collections::HashMap;
use sasspile::compile_with_files;

#[test]
fn test_use_with_import_chain_variable() {
    // Scenario: @use "midstream" where midstream.scss imports upstream.scss
    let input = "@use \"midstream\";\n\na {b: midstream.$member}\n";

    let mut files = HashMap::new();
    files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
    files.insert("upstream.scss".to_string(), "$member: value;".to_string());

    let result = compile_with_files(input, &files);
    println!("=== variable_use test ===");
    println!("Result:\n{}", result);
    assert!(
        result.contains("b: value"),
        "Expected 'b: value', got: {}",
        result
    );
}

#[test]
fn test_use_with_import_chain_mixin() {
    // Scenario: @use "midstream" where midstream.scss imports upstream with mixin
    let input = "@use \"midstream\";\n\n@include midstream.member;\n";

    let mut files = HashMap::new();
    files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
    files.insert(
        "upstream.scss".to_string(),
        "@mixin member() {a {b: c}}".to_string(),
    );

    let result = compile_with_files(input, &files);
    println!("=== mixin test ===");
    println!("Result:\n{}", result);
    assert!(
        result.contains("b: c"),
        "Expected 'b: c', got: {}",
        result
    );
}

#[test]
fn test_use_with_import_chain_function() {
    // Scenario: @use "midstream" where midstream.scss imports upstream with function
    let input = "@use \"midstream\";\n\na {b: midstream.member()}\n";

    let mut files = HashMap::new();
    files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
    files.insert(
        "upstream.scss".to_string(),
        "@function member() {@return value}".to_string(),
    );

    let result = compile_with_files(input, &files);
    println!("=== function test ===");
    println!("Result:\n{}", result);
    assert!(
        result.contains("b: value"),
        "Expected 'b: value', got: {}",
        result
    );
}
