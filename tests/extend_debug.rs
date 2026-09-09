//! Debug specific failing cases

use sasspile::{Reactor, OutputStyle};

fn compile(input: &str) -> Result<String, String> {
    Reactor::new(input.to_string())
        .lex()
        .map_err(|e| format!("{e}"))?
        .parse()
        .map_err(|e| format!("{e}"))?
        .evaluate()
        .map_err(|e| format!("{e}"))?
        .serialize(OutputStyle::Expanded)
        .finish()
        .map_err(|e| format!("{e}"))
}

#[test]
fn extend_list_one_matches() {
    let result = compile(r#"@use "sass:selector"; a {b: selector.extend(".c", ".c, .d", ".e")}"#);
    match result {
        Ok(s) => {
            let clean: String = s.chars().filter(|c| !c.is_whitespace()).collect();
            assert_eq!(clean, "a{b:.c,.e;}");
        }
        Err(e) => panic!("{}", e),
    }
}

#[test]
fn extend_list_different_matches() {
    let result = compile(r#"@use "sass:selector"; a {b: selector.extend(".c.d, .c .e, .d .f", ".c, .d", ".g")}"#);
    println!("Result: {:?}", result);
    match result {
        Ok(s) => {
            let clean: String = s.chars().filter(|c| !c.is_whitespace()).collect();
            println!("Clean: {}", clean);
        }
        Err(e) => panic!("{}", e),
    }
}

#[test]
fn debug_namespace_cases() {
    sasspile::init_tracing();

    let cases: Vec<(&str, &str, &str)> = vec![
        ("namespace/empty/and_empty (expect: |c, d)",
         r#"@use "sass:selector"; a { b: selector.extend("|c", "|c", "d"); }"#,
         "|c, d"),
        ("namespace/empty/and_explicit (expect: |c)",
         r#"@use "sass:selector"; a { b: selector.extend("|c", "d|c", "e"); }"#,
         "|c"),
        ("namespace/empty/and_implicit (expect: |c)",
         r#"@use "sass:selector"; a { b: selector.extend("|c", "c", "d"); }"#,
         "|c"),
        ("namespace/universal/and_empty (expect: NO-OP, |c)",
         r#"@use "sass:selector"; a { b: selector.extend("*|c", "|c", "d"); }"#,
         "*|c"),
        ("namespace/universal/and_universal (expect: *|c, d)",
         r#"@use "sass:selector"; a { b: selector.extend("*|c", "*|c", "d"); }"#,
         "*|c, d"),
        ("namespace/anyns_wildcard/and_universal (expect: NO-OP, *|*)",
         r#"@use "sass:selector"; a { b: selector.extend("*|*", "*|*", "c"); }"#,
         "*|*"),
        ("simple/universal/equal (expect: NO-OP, *)",
         r#"@use "sass:selector"; a { b: selector.extend("*", "*", "c"); }"#,
         "*"),
    ];

    for (desc, input, expected) in cases {
        match compile(input) {
            Ok(output) => {
                let clean = output.lines()
                    .filter(|l| !l.is_empty() && !l.starts_with("/*"))
                    .collect::<Vec<_>>().join("\n");
                tracing::info!("[{}]\n  output:   [{}]\n  expected: [{}]", desc, clean, expected);
            }
            Err(e) => tracing::error!(target: "extend_debug", "[{}] ERROR: {}", desc, e),
        }
    }
}
