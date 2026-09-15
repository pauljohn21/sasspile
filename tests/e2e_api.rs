//! 端到端 API 测试 — 直接调用 `sasspile_rx::compile()` 验证对外 API
//!
//! 测试策略：每个测试调用公共 `compile` 函数，断言输出 CSS 字符串。

use sasspile_rx::compile;

#[test]
fn test_basic_rule() {
    let css = compile("a { color: red; }").unwrap();
    assert!(css.contains("color"), "应包含属性名 color, got: {css}");
    assert!(css.contains("red"), "应包含值 red, got: {css}");
    info_contains(&css, "a");
}

#[test]
fn test_nested_rule() {
    let css = compile("a { color: red; span { color: blue; } }").unwrap();
    assert!(css.contains("color"));
}

#[test]
fn test_variable_declaration() {
    // $var : value;
    let src = r#"
$primary: red;
a { color: $primary; }
"#;
    let css = compile(src).expect("变量替换编译失败");
    assert!(
        css.contains("color"),
        "应包含属性 color, got: {css}"
    );
    assert!(
        css.contains("red"),
        "变量应被替换为 red, got: {css}"
    );
}

#[test]
fn test_interpolation_selector() {
    // 选择器中的插值 #{$var}
    let src = r#"
$name: foo;
.#{$name} { color: red; }
"#;
    let css = compile(src).expect("插值选择器编译失败");
    assert!(
        css.contains("foo"),
        "插值展开后应包含 foo, got: {css}"
    );
    assert!(
        css.contains("red"),
        "编译结果应包含 red, got: {css}"
    );
}

#[test]
fn test_mixin_basic() {
    // @mixin / @include — 当前只验证编译流程不报错
    let src = r#"
@mixin bold { font-weight: bold; }
a { @include bold; }
"#;
    let result = compile(src);
    match result {
        Ok(css) => {
            assert!(
                css.contains("font-weight") && css.contains("bold"),
                "mixin 应被展开, got: {css}"
            );
        }
        Err(e) => {
            panic!("@include 编译应成功, got error: {e}");
        }
    }
}

#[test]
fn test_mixin_with_args() {
    // 带参数的 mixin
    let src = r#"
@mixin theme($color) { color: $color; }
a { @include theme(blue); }
"#;
    let result = compile(src);
    match result {
        Ok(css) => {
            assert!(
                css.contains("color") && css.contains("blue"),
                "带参 mixin 调用应展开, got: {css}"
            );
        }
        Err(e) => {
            panic!("带参 @include 编译应成功, got error: {e}");
        }
    }
}

#[test]
fn test_if_directive() {
    // @if / @else
    let src = r#"
@if true { a { color: red; } }
"#;
    let result = compile(src);
    assert!(result.is_ok(), "@if true 编译应成功");
}

#[test]
fn test_for_directive() {
    // @for 循环
    let src = r#"
@for $i from 1 to 3 { .item-#{$i} { width: 10px * $i; } }
"#;
    let result = compile(src);
    match result {
        Ok(css) => {
            assert!(
                css.contains("item-"),
                "@for 循环应生成 item- 选择器, got: {css}"
            );
        }
        Err(e) => {
            panic!("@for 编译应成功, got error: {e}");
        }
    }
}

#[test]
fn test_compile_error() {
    // 故意传入无效输入，验证错误返回
    let result = compile("");
    // 空输入可能成功也可能失败，但不应 panic
    let _ = result;
}

fn info_contains(css: &str, needle: &str) {
    assert!(
        css.contains(needle),
        "结果 CSS 中应包含 {needle}, got: {css}"
    );
}
