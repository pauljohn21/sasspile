//! Validate meta function fixes: variable-exists and global-variable-exists.
//! Tests both forms (1-arg, 2-arg), dash-insensitive lookup, and error handling.
use sasspile::*;

#[test]
fn test_variable_exists_comprehensive() {
    init_tracing();
    let cases = vec![
        // 1-arg form: dash-insensitive lookup
        ("@use \"sass:meta\";\n$a_b: null;\nc {d: meta.variable-exists(a-b)}", "true"),
        ("@use \"sass:meta\";\n$a-b: null;\nc {d: meta.variable-exists(a_b)}", "true"),
        // Local variable visible from same scope
        ("@use \"sass:meta\";\na {\n  $local: null;\n  b: meta.variable-exists(local);\n}", "true"),
        // Non-existent returns false
        ("@use \"sass:meta\";\na {b: meta.variable-exists(nope)}", "false"),
    ];
    for (input, expected) in cases {
        let result = compile_expanded(input).unwrap_or_else(|e| format!("ERR: {e}"));
        assert!(
            result.contains(expected),
            "Input: {input}\nExpected: {expected}\nGot: {result}"
        );
    }
}

#[test]
fn test_global_variable_exists_comprehensive() {
    init_tracing();
    let cases = vec![
        // 1-arg: global scope variable
        ("@use \"sass:meta\";\n$g: null;\na {b: meta.global-variable-exists(g)}", "true"),
        // Local should NOT be global
        ("@use \"sass:meta\";\na {\n  $local: null;\n  b: meta.global-variable-exists(local);\n}", "false"),
        // Dash-insensitive
        ("@use \"sass:meta\";\n$a_b: null;\nc {d: meta.global-variable-exists(a-b)}", "true"),
        ("@use \"sass:meta\";\n$a-b: null;\nc {d: meta.global-variable-exists(a_b)}", "true"),
        // Non-existent
        ("@use \"sass:meta\";\na {b: meta.global-variable-exists(nope)}", "false"),
        // Named args
        ("@use \"sass:meta\";\n$g: null;\na {b: meta.global-variable-exists($name: \"g\")}", "true"),
    ];
    for (input, expected) in cases {
        let result = compile_expanded(input).unwrap_or_else(|e| format!("ERR: {e}"));
        assert!(
            result.contains(expected),
            "Input: {input}\nExpected: {expected}\nGot: {result}"
        );
    }
}

#[test]
fn test_global_variable_exists_error_cases() {
    init_tracing();
    let err_cases = vec![
        // Too few args
        ("@use \"sass:meta\";\na {b: meta.global-variable-exists()}"),
        // Too many args
        ("@use \"sass:meta\";\na {b: meta.global-variable-exists(a, b, c)}"),
        // Wrong type for name
        ("@use \"sass:meta\";\na {b: meta.global-variable-exists(12px)}"),
        // Wrong type for module
        ("@use \"sass:meta\";\na {b: meta.global-variable-exists(\"c\", 1)}"),
    ];
    for input in err_cases {
        let result = compile_expanded(input);
        assert!(result.is_err(), "Expected error for: {input}\nGot: {result:?}");
    }
}

#[test]
fn test_variable_exists_error_cases() {
    init_tracing();
    let err_cases = vec![
        ("@use \"sass:meta\";\na {b: meta.variable-exists()}"),
        ("@use \"sass:meta\";\na {b: meta.variable-exists(a, b)}"),
        ("@use \"sass:meta\";\na {b: meta.variable-exists(12px)}"),
    ];
    for input in err_cases {
        let result = compile_expanded(input);
        assert!(result.is_err(), "Expected error for: {input}\nGot: {result:?}");
    }
}
