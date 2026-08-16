use sasspile::{tokenize, parse};

fn test_case(name: &str, src: &str) {
    println!("=== {} ===", name);
    println!("Source: {}", src.trim());
    let (tokens, lex_diags) = tokenize(src);
    let (lex_e, lex_w, _) = lex_diags.counts();
    println!("Lexer: {}e {}w", lex_e, lex_w);
    if lex_e > 0 {
        for d in lex_diags.errors().iter() {
            println!("  LEX ERR: {}", d.message);
        }
    }
    let (_stylesheet, parse_diags) = parse(src);
    let (p_e, p_w, _) = parse_diags.counts();
    println!("Parser: {}e {}w", p_e, p_w);
    if p_e > 0 {
        for d in parse_diags.errors().iter() {
            println!("  PARSE ERR: {}", d.message);
        }
    }
    println!();
}

fn main() {
    // Test math.div followed by *
    test_case("math.div + star", r#"
$x: (math.div(1, 24) * 5 * 100) * 1%;
"#);

    // Test simple $var: &; pattern
    test_case("dollar colon ampersand", r#"
$selector: &;
"#);

    // Test simple mixin with $var: &
    test_case("mixin with dollar-ampersand", r#"
@mixin test {
  $selector: &;
  @at-root {
    #{$selector}::before {
      display: table;
    }
  }
}
"#);

    // Test @each with multiline vars
    test_case("each multiline", r#"
@each $placement,
  $adjacency
    in ('top': 'left', 'bottom': 'right')
{
  .#{$placement} {
    border-#{$placement}-color: transparent;
  }
}
"#);

    // Test @each normal
    test_case("each normal", r#"
@each $type in (a, b, c) {
  .#{$type} { color: red; }
}
"#);

    // Test @for with through
    test_case("for through", r#"
@for $i from 1 through 24 {
  .col-#{$i} { width: $i * 1%; }
}
"#);

    // Test map literal
    test_case("map literal in each", r#"
@each $k, $v in ('a': 1, 'b': 2) {
  .#{$k} { val: $v; }
}
"#);
}
