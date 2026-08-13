//! @use ... with ($key: value) 配置传递测试。
//!
//! 验证 @use 'module' with ($var: value) 能正确覆盖模块中的 !default 变量。

use sasspile::{compile_file, OutputStyle};

#[test]
fn test_use_with_single_config() {
    let dir = std::env::temp_dir().join("sasspile_use_with_single");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("_other.scss"),
        "$a: original !default;\nb {c: $a}\n",
    )
    .unwrap();
    let main = dir.join("main.scss");
    std::fs::write(
        &main,
        "@use \"other\" with ($a: configured);\n",
    )
    .unwrap();
    let css = compile_file(&main, OutputStyle::Expanded).unwrap();
    assert!(
        css.contains("c: configured"),
        "应该包含 c: configured, 得到: {css}"
    );
    assert!(
        !css.contains("original"),
        "不应该包含 original, 得到: {css}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_use_with_multiple_configs() {
    let dir = std::env::temp_dir().join("sasspile_use_with_multi");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("_other.scss"),
        "$a: original a !default;\n$b: original b !default;\n$c: original c !default;\nd {\n  a: $a;\n  b: $b;\n  c: $c;\n}\n",
    )
    .unwrap();
    let main = dir.join("main.scss");
    std::fs::write(
        &main,
        "@use \"other\" with (\n  $a: configured a,\n  $b: configured b,\n  $c: configured c\n);\n",
    )
    .unwrap();
    let css = compile_file(&main, OutputStyle::Expanded).unwrap();
    assert!(css.contains("a: configured a"), "应该包含 a: configured a, 得到: {css}");
    assert!(css.contains("b: configured b"), "应该包含 b: configured b, 得到: {css}");
    assert!(css.contains("c: configured c"), "应该包含 c: configured c, 得到: {css}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_use_with_no_config_keeps_default() {
    let dir = std::env::temp_dir().join("sasspile_use_with_no_config");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("_other.scss"),
        "$a: original !default;\nb {c: $a}\n",
    )
    .unwrap();
    let main = dir.join("main.scss");
    std::fs::write(&main, "@use \"other\";\n").unwrap();
    let css = compile_file(&main, OutputStyle::Expanded).unwrap();
    assert!(
        css.contains("c: original"),
        "没有 with 配置时应该保留默认值, 得到: {css}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_use_with_partial_config() {
    let dir = std::env::temp_dir().join("sasspile_use_with_partial");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("_other.scss"),
        "$a: original a !default;\n$b: original b !default;\nc {\n  a: $a;\n  b: $b;\n}\n",
    )
    .unwrap();
    let main = dir.join("main.scss");
    std::fs::write(
        &main,
        "@use \"other\" with ($a: configured a);\n",
    )
    .unwrap();
    let css = compile_file(&main, OutputStyle::Expanded).unwrap();
    assert!(
        css.contains("a: configured a"),
        "应该覆盖 $a, 得到: {css}"
    );
    assert!(
        css.contains("b: original b"),
        "未配置的 $b 应该保留默认值, 得到: {css}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_use_with_null_config() {
    let dir = std::env::temp_dir().join("sasspile_use_with_null");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("_other.scss"),
        "$a: original !default;\nb {c: $a}\n",
    )
    .unwrap();
    let main = dir.join("main.scss");
    std::fs::write(
        &main,
        "@use \"other\" with ($a: null);\n",
    )
    .unwrap();
    let css = compile_file(&main, OutputStyle::Expanded).unwrap();
    // null 配置应该将变量设为 null（在 Sass 中 null 值声明不输出）
    assert!(
        !css.contains("original"),
        "null 配置不应该保留默认值, 得到: {css}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_use_with_config_trailing_comma() {
    let dir = std::env::temp_dir().join("sasspile_use_with_trailing");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("_other.scss"),
        "$a: original !default;\nb {c: $a}\n",
    )
    .unwrap();
    let main = dir.join("main.scss");
    std::fs::write(
        &main,
        "@use \"other\" with ($a: configured,);\n",
    )
    .unwrap();
    let css = compile_file(&main, OutputStyle::Expanded).unwrap();
    assert!(
        css.contains("c: configured"),
        "尾随逗号应该正常工作, 得到: {css}"
    );
    std::fs::remove_dir_all(&dir).ok();
}
