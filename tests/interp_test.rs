//! 插值求值专项测试——覆盖 `eval_interp_segments` 修复后的各种场景。
//!
//! 参照 sass-spec 中的插值语义：
//! - 裸插值 `#{$var}` → 变量值（distributed_vars.hrx）
//! - 字符串内插 `"#{$var}"` → 带引号的插值结果
//! - 前缀+插值+后缀 `prefix-#{$a}-suffix` → 拼接
//! - 表达式插值 `#{1+2}px` → 求值后拼接
//! - 多段插值 `#{$a}#{$b}` → 拼接
//! - 字符串拼接 `foo#{"hey"}bar` → 拼接
//! - null 插值 `#{null}` → 空字符串

use sasspile::{compile_expanded, init_tracing_otel};

/// 编译 SCSS 字符串为展开格式 CSS。
fn compile(src: &str) -> String {
    let _ = init_tracing_otel();
    compile_expanded(src).expect("编译应成功")
}

// —— sass-spec: distributed_vars.hrx 核心场景 ——

#[test]
fn bare_variable_interp() {
    let css = compile("$a: hello; .x { content: #{$a}; }");
    assert!(
        css.contains("content: hello"),
        "裸插值 #{{$a}} 应求值为变量值: {css}"
    );
}

#[test]
fn string_interp() {
    let css = compile("$a: world; .x { content: \"#{$a}\"; }");
    assert!(
        css.contains("content: \"world\""),
        "字符串内插值应保留引号: {css}"
    );
}

// —— sass-spec: interpolated-strings.hrx ——

#[test]
fn prefix_suffix_interp() {
    // $y: why → heywhyho
    let css = compile("$y: why; .x { content: hey#{$y}ho; }");
    assert!(
        css.contains("content: heywhyho"),
        "前缀+插值+后缀应拼接: {css}"
    );
}

#[test]
fn string_concat_interp() {
    // foo#{"hey"}bar → fooheybar
    let css = compile(".x { content: foo#{\"hey\"}bar; }");
    assert!(
        css.contains("content: fooheybar"),
        "字符串拼接应正确: {css}"
    );
}

#[test]
fn quoted_string_in_interp() {
    // "hey #{$x} ho" → "hey ecks ho"
    let css = compile("$x: ecks; .x { content: \"hey #{$x} ho\"; }");
    assert!(
        css.contains("content: \"hey ecks ho\""),
        "引号字符串内插值应正确: {css}"
    );
}

// —— sass-spec: hyphen-interpolated.hrx ——

#[test]
fn hyphen_prefix_expr_interp() {
    // -hux-#{2+3} → -hux-5
    let css = compile(".x { content: -hux-#{2 + 3}; }");
    assert!(
        css.contains("content: -hux-5"),
        "前缀+表达式插值应拼接: {css}"
    );
}

// —— sass-spec: weird_added_space.hrx ——

#[test]
fn moz_prefix_variable_interp() {
    // -moz-#{$value} → -moz-bip
    let css = compile("$value: bip; .x { content: -moz-#{$value}; }");
    assert!(
        css.contains("content: -moz-bip"),
        "前缀+变量插值应拼接: {css}"
    );
}

// —— sass-spec: zero-compression.hrx ——

#[test]
fn zero_minus_variable_interp() {
    // 0 -#{$orig} → 0 -0.12em（空格分隔列表）
    let css = compile("$orig: 0.12em; .x { content: 0 -#{$orig}; }");
    assert!(
        css.contains("0 -0.12em") || css.contains("0-0.12em"),
        "0 -变量插值应拼接: {css}"
    );
}

// —— sass-spec: null.hrx ——

#[test]
fn null_interp() {
    // #{null} → 空字符串（content 声明值为空，输出可能省略或输出空内容）
    let css = compile(".x { content: #{null}; }");
    assert!(
        !css.contains("content: null"),
        "null 插值不应输出 'content: null': {css}"
    );
}

// —— sass-spec: 多段插值拼接 ——

#[test]
fn multiple_interp_segments() {
    let css = compile("$a: foo; $b: bar; .x { content: #{$a}#{$b}; }");
    assert!(css.contains("content: foobar"), "多段插值拼接应正确: {css}");
}

// —— 表达式插值+后缀 ——

#[test]
fn expression_with_suffix_interp() {
    // #{1 + 2}px → 3px
    let css = compile(".x { width: #{1 + 2}px; }");
    assert!(css.contains("width: 3px"), "表达式插值带后缀应求值: {css}");
}

#[test]
fn number_with_unit_interp() {
    // $n: 42 → #{$n}px → 42px
    let css = compile("$n: 42; .x { width: #{$n}px; }");
    assert!(
        css.contains("width: 42px"),
        "数字变量带单位插值应求值: {css}"
    );
}

#[test]
fn pure_expression_interp() {
    // #{1 + 2} → 3
    let css = compile(".x { width: #{1 + 2}; }");
    assert!(css.contains("width: 3"), "纯表达式插值应求值: {css}");
}

// —— sass-spec: basic_prop_name_interpolation.hrx ——

#[test]
fn property_name_expr_interp() {
    // bar#{1 + 2} → bar3
    let css = compile("foo { bar#{1 + 2}: blip; }");
    assert!(css.contains("bar3: blip"), "属性名表达式插值应求值: {css}");
}

// —— sass-spec: quotes-in-interpolated-strings.hrx ——

#[test]
fn interp_value_quoted_vs_unquoted() {
    // #{$bar}: #{$bar} → bar: bar （去引号）
    // #{$bar}: $bar → bar: "bar" （保留引号）
    let css = compile("$bar: \"bar\"; .x { content: #{$bar}; }");
    assert!(
        css.contains("content: bar"),
        "插值中的引号字符串应去引号: {css}"
    );
}
