//! Exact replication of sasspec use_to_import.hrx — including exact file keys and input

#[test]
fn test_exact_sasspec_variable_use() {
    let _ = tracing_subscriber::fmt::try_init();
    // From sasspec parsing of use_to_import.hrx variable_use test:
    // cur_name = "variable_use"
    // After prefix stripping: midstream.scss, upstream.scss
    let mut files = std::collections::HashMap::new();
    files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
    files.insert("upstream.scss".to_string(), "$member: value;".to_string());

    // input.scss content for variable_use test
    let input = "@use \"midstream\";\n\na {b: midstream.$member}";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== variable_use ===\n{result}");
    eprintln!("=== bytes: {:?} ===", result.bytes().collect::<Vec<_>>());
    assert_eq!(result.trim(), "a {\n  b: value;\n}");
}

#[test]
fn test_exact_sasspec_mixin() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut files = std::collections::HashMap::new();
    files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
    files.insert("upstream.scss".to_string(), "@mixin member() {a {b: c}}".to_string());

    let input = "@use \"midstream\";\n\n@include midstream.member";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== mixin ===\n{result}");
    eprintln!("=== bytes: {:?} ===", result.bytes().collect::<Vec<_>>());
    assert_eq!(result.trim(), "a {\n  b: c;\n}");
}

#[test]
fn test_exact_sasspec_function() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut files = std::collections::HashMap::new();
    files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
    files.insert("upstream.scss".to_string(), "@function member() {@return value}".to_string());

    let input = "@use \"midstream\";\n\na {b: midstream.member()}";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== function ===\n{result}");
    assert_eq!(result.trim(), "a {\n  b: value;\n}");
}

#[test]
fn test_exact_sasspec_variable_assignment() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut files = std::collections::HashMap::new();
    files.insert("midstream.scss".to_string(), "@import \"upstream\";".to_string());
    files.insert(
        "upstream.scss".to_string(),
        "$member: value;\n\n@function get-member() {@return $member}".to_string(),
    );

    let input = "@use \"midstream\";\n\nmidstream.$member: new value;\n\na {b: midstream.get-member()}";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== variable_assignment ===\n{result}");
    assert_eq!(result.trim(), "a {\n  b: new value;\n}");
}
