//! Tests for nested @import (inside selector blocks)

#[test]
fn test_nested_import_mixin() {
    let _ = tracing_subscriber::fmt::try_init();
    // nested.hrx mixin test: @import inside a { } block
    let mut files = std::collections::HashMap::new();
    files.insert(
        "_midstream.scss".to_string(),
        "@forward \"upstream\";".to_string(),
    );
    files.insert(
        "_upstream.scss".to_string(),
        "@mixin b() {c: d}".to_string(),
    );

    let input = "a {\n  @import \"midstream\";\n\n  @include b;\n}";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== nested mixin result ===\n{result}");
    assert!(
        result.contains("c: d"),
        "expected 'c: d' from mixin b, got: {result}"
    );
}

#[test]
fn test_nested_import_variable() {
    let _ = tracing_subscriber::fmt::try_init();
    // nested.hrx variable_use test
    let mut files = std::collections::HashMap::new();
    files.insert(
        "_midstream.scss".to_string(),
        "@forward \"upstream\";".to_string(),
    );
    files.insert("_upstream.scss".to_string(), "$c: d;".to_string());

    let input = "a {\n  @import \"midstream\";\n\n  b: $c;\n}";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== nested variable result ===\n{result}");
    assert!(
        result.contains("b: d"),
        "expected 'b: d', got: {result}"
    );
}
