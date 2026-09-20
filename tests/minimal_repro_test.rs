//! Minimal reproduction for rest-param function dispatch
//!
//! Run: cargo test --features otel --test minimal_repro_test -- --nocapture

use std::io::Write;

fn run_case(label: &str, scss: &str) -> (bool, String) {
    let result = sasspile::compile(scss, sasspile::OutputStyle::Expanded);
    match result {
        Ok(css) => {
            let unresolved = css.contains("joinVarName(")
                || css.contains(" jn(")
                || css.contains("inner(")
                || css.contains("join-var(")
                || css.contains("gcv(");
            if unresolved {
                (false, format!("FAIL {label}: {css}"))
            } else {
                (true, format!("PASS {label}: {css}"))
            }
        }
        Err(e) => (false, format!("ERR {label}: {e}")),
    }
}

#[test]
fn test_rest_param_dispatch() {
    let mut output = Vec::new();

    let cases = [
        (
            "rest->list",
            r#"@function inner($list) { @return nth($list, 1); }
@function outer($args...) { @return inner($args); }
.test { val: outer(a, b, c); }"#,
        ),
        (
            "joinVarName",
            r#"@function joinVarName($list) {
    $name: '--el';
    @each $item in $list { $name: $name + '-' + $item; }
    @return $name;
}
@function getCssVar($args...) { @return var(#{joinVarName($args)}); }
.test { color: getCssVar(button, text-color); }"#,
        ),
        (
            "direct-list",
            r#"@function jv($list) {
    $name: '--el';
    @each $item in $list { $name: $name + '-' + $item; }
    @return $name;
}
.test { val: jv((a, b, c)); }"#,
        ),
        (
            "EP-rest-call",
            r#"@function jn($list) {
    $name: '--el';
    @each $item in $list { $name: $name + '-' + $item; }
    @return $name;
}
@function gcv($args...) { @return var(#{jn($args)}); }
.test { color: gcv(button, text); }"#,
        ),
        (
            "explicit-tuple",
            r#"@function jn($list) {
    $name: '--el';
    @each $item in $list { $name: $name + '-' + $item; }
    @return $name;
}
@function gcv($a, $b) { @return var(#{jn(($a, $b))}); }
.test { color: gcv(button, text); }"#,
        ),
    ];

    let mut all_pass = true;
    for (label, scss) in &cases {
        let (ok, msg) = run_case(label, scss);
        if !ok {
            all_pass = false;
        }
        writeln!(output, "{msg}").expect("write");
    }

    let output_str = String::from_utf8(output).expect("utf8");
    // Write to file for analysis
    std::fs::write("/tmp/minimal_repro_results.txt", &output_str).expect("write file");

    // Also emit as a tracing error (will appear in OTel span events)
    for line in output_str.lines() {
        if line.starts_with("PASS") {
            tracing::info!("{line}");
        } else {
            tracing::error!("{line}");
        }
    }

    assert!(all_pass, "Some cases failed — see /tmp/minimal_repro_results.txt");
}
