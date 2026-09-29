//! Tests for mixin parsing and @import/@forward chain resolution

#[test]
fn test_multiline_mixin_in_forward_import_chain() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut files = std::collections::HashMap::new();
    files.insert("_used.scss".to_string(), "@import \"imported\";".to_string());
    files.insert("_imported.scss".to_string(), "@forward \"forwarded\";".to_string());
    files.insert("_forwarded.scss".to_string(), "@mixin a() {b {c: d}}".to_string());

    let input = "@use \"used\";\n\n@include used.a;";
    let result = sasspile::compile_with_files(input, &files);
    assert!(
        result.contains("c: d"),
        "expected mixin 'a' to expand to 'c: d', got: {result}"
    );
}

#[test]
fn test_forward_variable_chain() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut files = std::collections::HashMap::new();
    files.insert("_used.scss".to_string(), "@import \"imported\";".to_string());
    files.insert("_imported.scss".to_string(), "@forward \"forwarded\";".to_string());
    files.insert("_forwarded.scss".to_string(), "$c: d;".to_string());

    let input = "@use \"used\";\n\na {b: used.$c}";
    let result = sasspile::compile_with_files(input, &files);
    assert!(
        result.contains("b: d"),
        "expected variable 'c' to resolve to 'd', got: {result}"
    );
}

#[test]
fn test_use_to_import_exact_sasspec_scenario() {
    let _ = tracing_subscriber::fmt::try_init();
    // Mimics sasspec exact scenario for use_to_import.hrx variable_use test
    let mut files = std::collections::HashMap::new();
    files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
    files.insert("upstream.scss".to_string(), "$member: value;".to_string());

    let input = "@use \"midstream\";\n\na {b: midstream.$member}";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== RESULT ===\n{result}");
    assert!(
        result.contains("b: value"),
        "expected b: value, got: {result}"
    );
}

#[test]
fn test_use_to_import_mixin_exact_sasspec_scenario() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut files = std::collections::HashMap::new();
    files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
    files.insert("upstream.scss".to_string(), "@mixin member() {a {b: c}}".to_string());

    let input = "@use \"midstream\";\n\n@include midstream.member";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== RESULT ===\n{result}");
    assert!(
        result.contains("b: c"),
        "expected b: c, got: {result}"
    );
}
