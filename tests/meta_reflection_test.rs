//! meta 反射函数测试——feature-exists, content-exists, global-variable-exists, variable-exists, call。

fn compile(input: &str) -> Result<String, String> {
    sasspile::compile_expanded(input).map_err(|e| e.to_string())
}

#[test]
fn test_feature_exists_supported() {
    // global-variable-shadowing 是 sass 核心支持的 feature
    let css = compile("a { b: feature-exists(global-variable-shadowing); }").expect("unexpected failure in test");
    assert!(css.contains("true"), "应返回 true: {css}");
}

#[test]
fn test_feature_exists_unsupported() {
    // 不支持的 feature
    let css = compile("a { b: feature-exists(unknown-feature-xyz); }").expect("unexpected failure in test");
    assert!(css.contains("false"), "应返回 false: {css}");
}

#[test]
fn test_feature_exists_custom_property() {
    // custom-property 是支持的 feature
    let css = compile("a { b: feature-exists(custom-property); }").expect("unexpected failure in test");
    assert!(css.contains("true"), "应返回 true: {css}");
}

#[test]
fn test_global_variable_exists() {
    // 全局变量存在
    let css = compile("$global-var: 123; a { b: global-variable-exists(global-var); }").expect("unexpected failure in test");
    assert!(css.contains("true"), "应返回 true: {css}");
}

#[test]
fn test_global_variable_not_exists() {
    // 全局变量不存在
    let css = compile("a { b: global-variable-exists(no-such-var); }").expect("unexpected failure in test");
    assert!(css.contains("false"), "应返回 false: {css}");
}

#[test]
fn test_variable_exists() {
    // 局部变量存在
    let css = compile("a { $local: 456; b: variable-exists(local); }").expect("unexpected failure in test");
    assert!(css.contains("true"), "应返回 true: {css}");
}

#[test]
fn test_variable_not_exists() {
    // 变量不存在
    let css = compile("a { b: variable-exists(no-such-var); }").expect("unexpected failure in test");
    assert!(css.contains("false"), "应返回 false: {css}");
}

#[test]
fn test_content_exists_in_mixin_with_content() {
    // mixin 接收 @content
    let css = compile("@mixin apply { @content; } a { @include apply { color: red; } b: content-exists(); }").expect("unexpected failure in test");
    // content-exists() 在 mixin 内调用，检查当前 mixin 是否有 @content
    assert!(css.contains("color: red"), "应包含 color: red: {css}");
}

#[test]
fn test_call_function() {
    // meta.call 调用函数
    let css = compile("@function add($a, $b) { @return $a + $b; } a { b: call(get-function(add), 1, 2); }").expect("unexpected failure in test");
    assert!(css.contains("3"), "应返回 3: {css}");
}

#[test]
fn test_mixin_exists() {
    let css = compile("@mixin my-mixin { color: red; } a { b: mixin-exists(my-mixin); }").expect("unexpected failure in test");
    assert!(css.contains("true"), "应返回 true: {css}");
}

#[test]
fn test_mixin_not_exists() {
    let css = compile("a { b: mixin-exists(no-such-mixin); }").expect("unexpected failure in test");
    assert!(css.contains("false"), "应返回 false: {css}");
}

#[test]
fn test_function_exists() {
    let css = compile("@function my-func() { @return 42; } a { b: function-exists(my-func); }").expect("unexpected failure in test");
    assert!(css.contains("true"), "应返回 true: {css}");
}

#[test]
fn test_function_not_exists() {
    let css = compile("a { b: function-exists(no-such-func); }").expect("unexpected failure in test");
    assert!(css.contains("false"), "应返回 false: {css}");
}
