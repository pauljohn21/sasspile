//! 变量定义 + 替换测试 (Phase A)
//!
//! 验证 rxrust 管线的变量能力:
//!   - 变量定义 $name: value;
//!   - $var 直接引用替换
//!   - #{$var} 插值语法
//!   - 变量在属性值 / 选择器 / @include mixin 中展开

use sasspile::compile;

#[test]
fn var_def_and_basic_substitution() {
    let input = "$color: red;\n.btn { color: $color; }";
    let output = compile(input);
    assert!(
        !output.contains("$"),
        "输出不应保留 $var 引用, got: {output}"
    );
    assert!(
        output.contains("red"),
        "变量应展开为值, got: {output}"
    );
}

#[test]
fn var_interpolation_syntax() {
    let input = "$size: lg;\n.icon-#{$size} { width: 1em; }";
    let output = compile(input);
    // 注: 字面 #{} 的写法在 format! 里需要 {{}} 转义,但 assert! 第二个参数不含 format 参数,无此限制
    assert!(
        output.contains(".icon-lg"),
        "hashtag interpolation expansion, got: {output}"
    );
}

#[test]
fn var_in_selector() {
    let input = "$ns: btn;\n.#{$ns}--primary { color: blue; }";
    let output = compile(input);
    assert!(
        output.contains(".btn--primary"),
        "selector hashtag interpolation, got: {output}"
    );
}

#[test]
fn var_plus_mixin_combo() {
    // note: 当前管线要求 @include 独占一行(逐行处理),不能在嵌套块内联
    let input = "$pad: 16px;\n@mixin pad($x: 8px) { padding: $x; }\n@include pad($pad);\n";
    let output = compile(input);
    assert!(
        output.contains("padding: 16px"),
        "variable passed via @include, got: {output}"
    );
}

#[test]
fn var_multi_use() {
    let input = "$c: red;\n$w: 2px;\n.box { color: $c; border: $w solid $c; }";
    let output = compile(input);
    assert!(
        output.contains("color: red"),
        "multi-var substitution (property), got: {output}"
    );
    assert!(
        output.contains("2px"),
        "multi-var substitution (width), got: {output}"
    );
}
