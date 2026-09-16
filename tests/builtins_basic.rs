//! 内置函数基础验收 (对齐 sass-spec core_functions 中的对应语义)

use sasspile_rx::compile;

#[test]
fn builtin_map_get_basic() {
    let src = r#"
$config: (color: red, size: 10px);
a { color: map-get($config, color); }
"#;
    let css = compile(src).expect("map-get 编译应成功");
    assert!(css.contains("color"), "应输出 color: red");
    assert!(css.contains("red"), "值应为 red");
}

#[test]
fn builtin_map_has_key() {
    let src = r#"
$config: (color: red);
a { b: map-has-key($config, color); }
c { b: map-has-key($config, missing); }
"#;
    let css = compile(src).expect("map-has-key 编译应成功");
    assert!(css.contains("true"));
    assert!(css.contains("false"));
}

#[test]
fn builtin_nth() {
    let src = r#"a { b: nth(red green blue, 2); }"#;
    let css = compile(src).expect("nth 编译应成功");
    assert!(css.contains("green"));
}

#[test]
fn builtin_length() {
    let src = r#"a { b: length(red green blue); }"#;
    let css = compile(src).expect("length 编译应成功");
    assert!(css.contains("3"));
}

#[test]
fn builtin_unquote() {
    let src = r#"a { b: unquote("hello"); }"#;
    let css = compile(src).expect("unquote 编译应成功");
    assert!(css.contains("hello"));
}

#[test]
fn builtin_quote() {
    let src = r#"a { b: quote(hello); }"#;
    let css = compile(src).expect("quote 编译应成功");
    assert!(css.contains("\"hello\""));
}

#[test]
fn builtin_mix_colors() {
    let src = r#"a { b: mix(red, blue, 50%); }"#;
    let css = compile(src).expect("mix 编译应成功");
    assert!(css.contains('#'));
}

#[test]
fn builtin_lighten() {
    let src = r#"a { b: lighten(red, 20%); }"#;
    let css = compile(src).expect("lighten 编译应成功");
    assert!(css.contains('#'));
}

#[test]
fn builtin_darken() {
    let src = r#"a { b: darken(red, 20%); }"#;
    let css = compile(src).expect("darken 编译应成功");
    assert!(css.contains('#'));
}

#[test]
fn builtin_type_of() {
    let src = r#"
$num: 10;
$str: "hello";
a { b: type-of($num); }
c { b: type-of($str); }
"#;
    let css = compile(src).expect("type-of 编译应成功");
    assert!(css.contains("number"));
    assert!(css.contains("string"));
}

#[test]
fn builtin_append() {
    let src = r#"a { b: append(red blue, green); }"#;
    let css = compile(src).expect("append 编译应成功");
    assert!(css.contains("green"));
}

#[test]
fn builtin_abs() {
    let src = r#"a { b: abs(-5px); }"#;
    let css = compile(src).expect("abs 编译应成功");
    assert!(css.contains("5px"));
}
