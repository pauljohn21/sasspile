//! Exact replication of sasspec use_to_import.hrx tests

#[test]
fn test_use_to_import_variable() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut files = std::collections::HashMap::new();
    files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
    files.insert("upstream.scss".to_string(), "$member: value;".to_string());

    let input = "@use \"midstream\";\n\na {b: midstream.$member}";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== variable_use result ===\n{result}");
    assert_eq!(result.trim(), "a {\n  b: value;\n}", "got: {result}");
}

#[test]
fn test_use_to_import_mixin() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut files = std::collections::HashMap::new();
    files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
    files.insert("upstream.scss".to_string(), "@mixin member() {a {b: c}}".to_string());

    let input = "@use \"midstream\";\n\n@include midstream.member";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== mixin result ===\n{result}");
    assert_eq!(result.trim(), "a {\n  b: c;\n}", "got: {result}");
}

#[test]
fn test_use_to_import_function() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut files = std::collections::HashMap::new();
    files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
    files.insert("upstream.scss".to_string(), "@function member() {@return value}".to_string());

    let input = "@use \"midstream\";\n\na {b: midstream.member()}";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== function result ===\n{result}");
    assert_eq!(result.trim(), "a {\n  b: value;\n}", "got: {result}");
}

#[test]
fn test_use_to_import_variable_assignment() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut files = std::collections::HashMap::new();
    files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
    files.insert("upstream.scss".to_string(), "$member: value;\n\n@function get-member() {@return $member}".to_string());

    let input = "@use \"midstream\";\n\nmidstream.$member: new value;\n\na {b: midstream.get-member()}";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== variable_assignment result ===\n{result}");
    assert_eq!(result.trim(), "a {\n  b: new value;\n}", "got: {result}");
}

/// Run all 4 use_to_import tests in sequence (simulates some parallelism)
#[test]
fn test_use_to_import_all_sequential() {
    let _ = tracing_subscriber::fmt::try_init();

    // Test 1: variable_use
    {
        let mut files = std::collections::HashMap::new();
        files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
        files.insert("upstream.scss".to_string(), "$member: value;".to_string());
        let input = "@use \"midstream\";\n\na {b: midstream.$member}";
        let result = sasspile::compile_with_files(input, &files);
        assert_eq!(result.trim(), "a {\n  b: value;\n}", "variable_use got: {result}");
    }

    // Test 2: mixin
    {
        let mut files = std::collections::HashMap::new();
        files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
        files.insert("upstream.scss".to_string(), "@mixin member() {a {b: c}}".to_string());
        let input = "@use \"midstream\";\n\n@include midstream.member";
        let result = sasspile::compile_with_files(input, &files);
        assert_eq!(result.trim(), "a {\n  b: c;\n}", "mixin got: {result}");
    }

    // Test 3: function
    {
        let mut files = std::collections::HashMap::new();
        files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
        files.insert("upstream.scss".to_string(), "@function member() {@return value}".to_string());
        let input = "@use \"midstream\";\n\na {b: midstream.member()}";
        let result = sasspile::compile_with_files(input, &files);
        assert_eq!(result.trim(), "a {\n  b: value;\n}", "function got: {result}");
    }

    // Test 4: variable_assignment
    {
        let mut files = std::collections::HashMap::new();
        files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
        files.insert("upstream.scss".to_string(), "$member: value;\n\n@function get-member() {@return $member}".to_string());
        let input = "@use \"midstream\";\n\nmidstream.$member: new value;\n\na {b: midstream.get-member()}";
        let result = sasspile::compile_with_files(input, &files);
        assert_eq!(result.trim(), "a {\n  b: new value;\n}", "variable_assignment got: {result}");
    }
}
