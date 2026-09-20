//! Minimal reproduction for unquote(map.get()) evaluation

use std::io::Write;

fn run_case(label: &str, scss: &str) -> (bool, String) {
    let result = sasspile::compile(scss, sasspile::OutputStyle::Expanded);
    match result {
        Ok(css) => {
            let unresolved = css.contains("joinVarName(")
                || css.contains(" jn(")
                || css.contains("inner(")
                || css.contains("join-var(")
                || css.contains("gcv(")
                || css.contains("string.unquote(")
                || css.contains("map.get(");
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
fn test_unquote_map_get_diagnosis() {
    let mut output = Vec::new();

    let cases = [
        // KEY DIFFERENCE: map parameter comes from mixin argument
        (
            "mixin-param-map-get",
            r#"@use 'sass:map';
@use 'sass:string';
$breakpoints: (
  'sm': '(min-width: 576px)',
  'md': '(min-width: 768px)',
);
@mixin res($key, $map: $breakpoints) {
  @if map.has-key($map, $key) {
    @media only screen and #{string.unquote(map.get($map, $key))} {
      @content;
    }
  } @else {
    @warn "Undefined points: `#{$map}`";
  }
}
.test {
  @include res('sm') {
    display: none !important;
  }
}"#,
        ),
        // Simpler version: just a function parameter
        (
            "fn-param-map-get",
            r#"@use 'sass:map';
@use 'sass:string';
$breakpoints: (
  'sm': '(min-width: 576px)',
);
@function get-bp($key, $map: $breakpoints) {
  @return string.unquote(map.get($map, $key));
}
.test { content: get-bp('sm'); }"#,
        ),
        // Does the @if condition matter?
        (
            "without-if",
            r#"@use 'sass:map';
@use 'sass:string';
$breakpoints: (
  'sm': '(min-width: 576px)',
);
@mixin res($key, $map: $breakpoints) {
  @media only screen and #{string.unquote(map.get($map, $key))} {
    @content;
  }
}
.test {
  @include res('sm') {
    display: none !important;
  }
}"#,
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
    std::fs::write("/tmp/unquote_diag_results.txt", &output_str).expect("write file");

    for line in output_str.lines() {
        if line.starts_with("PASS") {
            tracing::info!("{line}");
        } else {
            tracing::error!("{line}");
        }
    }

    assert!(all_pass, "Some cases failed — see /tmp/unquote_diag_results.txt");
}
