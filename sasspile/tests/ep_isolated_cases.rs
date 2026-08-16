//! EP isolated failure diagnosis — testing specific syntax patterns.

use sasspile::{tokenize, parse};
use tracing::info;

fn test_case(name: &str, src: &str) -> Result<(), String> {
    let (tokens, lex_diags) = tokenize(src);
    let (lex_e, lex_w, _) = lex_diags.counts();
    if lex_e > 0 {
        let detail: Vec<String> = lex_diags.errors().iter().map(|d| d.message.clone()).collect();
        return Err(format!("lexer: {lex_e} errors — {}", detail.join("; ")));
    }

    let (_stylesheet, parse_diags) = parse(src);
    let (p_e, p_w, _) = parse_diags.counts();
    if p_e > 0 {
        let detail: Vec<String> = parse_diags.errors().iter().map(|d| d.message.clone()).collect();
        return Err(format!("parser: {p_e} errors, {p_w} warns — {}", detail.join("; ")));
    }
    info!(name, "OK");
    Ok(())
}

#[test]
fn test_math_div_and_star() {
    let cases = &[
        ("math.div + star simple", r#"$x: math.div(1, 24) * 1%;"#, ),
        ("math.div basic", r#"$x: math.div(1, 24);"#, ),
        ("paren math.div star", r#"$x: (math.div(1, 24) * 5 * 100) * 1%;"#, ),
        ("paren simple", r#"$x: (1 + 2) * 3%;"#, ),
    ];
    let mut failed = Vec::new();
    for (name, src) in cases {
        if let Err(e) = test_case(name, src) {
            failed.push((*name, e));
        }
    }
    if !failed.is_empty() {
        let detail: Vec<String> = failed.iter().map(|(n, e)| format!("{n}: {e}")).collect();
        panic!("math.div tests:\n{}", detail.join("\n"));
    }
}

#[test]
fn test_variable_ampersand() {
    let cases = &[
        ("simple dollar-amp", r#"$selector: &;"#, ),
        ("in mixin", r#"
@mixin test {
  $selector: &;
  @at-root {
    #{$selector}::before {
      display: table;
    }
  }
}
"#, ),
    ];
    let mut failed = Vec::new();
    for (name, src) in cases {
        if let Err(e) = test_case(name, src) {
            failed.push((*name, e));
        }
    }
    if !failed.is_empty() {
        let detail: Vec<String> = failed.iter().map(|(n, e)| format!("{n}: {e}")).collect();
        panic!("ampersand tests:\n{}", detail.join("\n"));
    }
}

#[test]
fn test_if_else_patterns() {
    let cases = &[
        ("simple if-else", r#"
$x: 1;
@if $x == 0 {
  .a { color: red; }
} @else {
  .b { color: blue; }
}
"#),
        ("if-else-if chain", r#"
$x: 1;
@if $x == 0 {
  .a { color: red; }
} @else if $x == 1 {
  .b { color: green; }
} @else {
  .c { color: blue; }
}
"#),
        ("nested if-else in rule", r#"
@mixin foo {
  @include bar {
    @for $i from 1 through 3 {
      .x {
        @if $i == 0 {
          display: none;
        } @else {
          display: block;
        }
      }
    }
  }
}
"#),
        ("if as last stmt in rule", r#"
.x {
  @if $i == 0 {
    display: none;
  } @else {
    display: block;
  }
}
"#),
        ("if only (no else)", r#"
.x {
  @if $i == 0 {
    display: none;
  }
}
"#),
    ];
    let mut failed = Vec::new();
    for (name, src) in cases {
        if let Err(e) = test_case(name, src) {
            failed.push((*name, e));
        }
    }
    if !failed.is_empty() {
        let detail: Vec<String> = failed.iter().map(|(n, e)| format!("{n}: {e}")).collect();
        panic!("if-else tests:\n{}", detail.join("\n"));
    }
}

#[test]
fn test_each_for_patterns() {
    let cases = &[
        ("each normal", r#"
@each $type in (a, b, c) {
  .#{$type} { color: red; }
}
"#),
        ("each multiline", r#"
@each $placement,
  $adjacency
    in ('top': 'left', 'bottom': 'right')
{
  .#{$placement} {
    border-#{$placement}-color: transparent;
  }
}
"#),
        ("each map", r#"
@each $k, $v in ('a': 1, 'b': 2) {
  .#{$k} { val: $v; }
}
"#),
        ("for through", r#"
@for $i from 1 through 24 {
  .col-#{$i} { width: $i * 1%; }
}
"#),
        ("for to", r#"
@for $i from 1 to 24 {
  .col-#{$i} { width: $i * 1%; }
}
"#),
    ];
    let mut failed = Vec::new();
    for (name, src) in cases {
        if let Err(e) = test_case(name, src) {
            failed.push((*name, e));
        }
    }
    if !failed.is_empty() {
        let detail: Vec<String> = failed.iter().map(|(n, e)| format!("{n}: {e}")).collect();
        panic!("each/for tests:\n{}", detail.join("\n"));
    }
}
