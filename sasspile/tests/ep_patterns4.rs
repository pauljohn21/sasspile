//! 剩余 EP 失败模式诊断。

use sasspile::{tokenize, parse};

fn test_parse_debug(name: &str, src: &str) {
    let (_tokens, lex_diags) = tokenize(src);
    let lex_e = lex_diags.errors().len();
    let (_stylesheet, parse_diags) = parse(src);
    let parse_e = parse_diags.errors().len();
    if lex_e == 0 && parse_e == 0 {
        tracing::info!(pattern = %name, "OK");
    } else {
        let lex_msg: Vec<String> = lex_diags.errors().iter().map(|d| d.message.clone()).collect();
        let parse_msg: Vec<String> = parse_diags.errors().iter().map(|d| d.message.clone()).collect();
        tracing::info!(pattern = %name, lex_e, parse_e, lex_msg = ?lex_msg, parse_msg = ?parse_msg, "FAIL");
    }
}

fn init() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .try_init();
}

#[test]
fn test_remaining_patterns() {
    init();

    // 1. @at-root with interpolation + pseudo-element (utils.scss pattern)
    test_parse_debug("at_root_before", "@mixin foo {\n  $sel: &;\n  @at-root {\n    #{$sel}::before {\n      color: red;\n    }\n  }\n}");

    // 2. Interpolation::after
    test_parse_debug("interp_after", "@mixin foo {\n  $sel: &;\n  @at-root {\n    #{$sel}::after {\n      color: red;\n    }\n  }\n}");

    // 3. Simple interpolation in selector
    test_parse_debug("interp_selector", ".#{$ns}-col-#{$i} { color: red; }");

    // 4. Attribute selector with interpolation
    test_parse_debug("attr_interp", "[class*='#{$ns}-col-'] { color: red; }");

    // 5. Comma-separated interpolation selectors
    test_parse_debug("interp_comma_sel", "@at-root {\n  #{$sel}::before, #{$sel}::after {\n    display: table;\n  }\n}");

    // 6. content: '' (empty string)
    test_parse_debug("empty_string_content", ".foo { content: ''; }");

    // 7. Multi-condition @if
    test_parse_debug("multi_if", "@mixin foo {\n  @if $i == 0 {\n    display: none;\n  } @else {\n    display: block;\n  }\n}");

    // 8. math.div
    test_parse_debug("math_div", ".foo { width: math.div(1, 24) * 100%; }");

    // 9. $selector: &;
    test_parse_debug("dollar_amp", "@mixin foo {\n  $sel: &;\n}");

    // 10. max-width with complex expression
    test_parse_debug("complex_expr", ".foo { max-width: (math.div(1, 24) * $i * 100) * 1%; }");

    // 11. Property with interpolation
    test_parse_debug("prop_interp", ".foo { #{$prop}: red; }");

    // 12. Comma selector with interpolation
    test_parse_debug("comma_interp", "h1, h2 { color: red; }");

    // 13. interpolation::before as part of compound
    test_parse_debug("interp_pseudo_only", "#{$sel}::before { color: red; }");
}
