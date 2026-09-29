//! Tests for @use "url" as * (global namespace)

#[test]
fn test_use_as_star_variable() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut files = std::collections::HashMap::new();
    files.insert("other.scss".to_string(), "$member: value;".to_string());

    let input = "@use \"other\" as *;\n\na {b: $member}";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== variable result ===\n{result}");
    assert_eq!(result.trim(), "a {\n  b: value;\n}", "got: {result}");
}

#[test]
fn test_use_as_star_mixin() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut files = std::collections::HashMap::new();
    files.insert("other.scss".to_string(), "@mixin member() {a {b: c}}".to_string());

    let input = "@use \"other\" as *;\n\n@include member";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== mixin result ===\n{result}");
    assert_eq!(result.trim(), "a {\n  b: c;\n}", "got: {result}");
}

#[test]
fn test_use_as_star_function() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut files = std::collections::HashMap::new();
    files.insert("other.scss".to_string(), "@function member() {@return value}".to_string());

    let input = "@use \"other\" as *;\n\na {b: member()}";
    let result = sasspile::compile_with_files(input, &files);
    eprintln!("=== function result ===\n{result}");
    assert_eq!(result.trim(), "a {\n  b: value;\n}", "got: {result}");
}
