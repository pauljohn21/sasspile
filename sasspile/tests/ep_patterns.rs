//! EP 失败模式最小复现测试。

use sasspile::{tokenize, parse};

fn test_parse(name: &str, src: &str) {
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
fn test_ep_patterns() {
    init();

    // 1. BEM @include with string arg (table-v2 pattern)
    test_parse("bem_include_string", r#"
@include b('table-v2') {
  color: red;
}
"#);

    // 2. @include e() element mixin
    test_parse("include_e", r#"
@include e('root') {
  position: relative;
}
"#);

    // 3. Multi-line box-shadow (input.scss pattern)
    test_parse("multiline_box_shadow", r#"
.foo {
  box-shadow:
    1px 0 0 0 red inset,
    0 1px 0 0 red inset,
    0 -1px 0 0 red inset;
}
"#);

    // 4. calc with function call (reset.scss pattern)
    test_parse("calc_function", r#"
.foo {
  font-size: calc(getCssVar('font-size', 'base') + 6px);
}
"#);

    // 5. function() - $var pattern (input.scss line 57)
    test_parse("function_minus_var", r#"
.foo {
  padding: 5px map.get($input, 'default') - $border-width;
}
"#);

    // 6. var(#{...}, ...) interpolation in var()
    test_parse("var_with_interp", r#"
.foo {
  color: var(#{getName()}, red);
}
"#);

    // 7. @at-root with interpolation (utils.scss pattern)
    test_parse("at_root_interp", r#"
@mixin utils-clearfix {
  $selector: &;
  @at-root {
    #{$selector}::before,
    #{$selector}::after {
      display: table;
    }
  }
}
"#);

    // 8. Multi-comma selector (reset.scss pattern)
    test_parse("multi_comma_selector", r#"
h1, h2, h3, h4, h5, h6 {
  color: red;
}
"#);

    // 9. @include when() pattern
    test_parse("include_when", r#"
@include when('align-center') {
  justify-content: center;
}
"#);

    // 10. Complex nested BEM
    test_parse("nested_bem", r#"
@include b(table-v2) {
  @include e(root) {
    position: relative;
    &:hover {
      @include e(main) {
        opacity: 1;
      }
    }
  }
}
"#);

    // 11. mixin def with @content
    test_parse("mixin_with_content", r#"
@mixin b($block) {
  $B: $ns + '-' + $block;
  .#{$B} {
    @content;
  }
}
"#);

    // 12. map.get with string key
    test_parse("map_get_string_key", r#"
.foo {
  padding: map.get($input-padding-horizontal, 'default');
}
"#);

    // 13. Multi-value padding with operation
    test_parse("multi_value_op", r#"
.foo {
  padding: 5px map.get($map, 'key') + 14px 5px map.get($map, 'key') - 1px;
}
"#);

    // 14. Single-quoted string in include
    test_parse("single_quote_include", r#"
@use 'sass:map';
@use 'mixins/mixins' as *;
"#);

    // 15. @include with content block and using
    test_parse("include_using", r#"
@mixin foo {
  @content;
}
@include foo using ($arg) {
  color: $arg;
}
"#);
}
