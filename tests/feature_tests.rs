//! 功能测试——关键字参数、@forward as prefix-* 等特性的集成测试。
//!
//! 这些测试使用公共 API（compile_expanded / compile_file），
//! 在独立编译单元中运行，与源码隔离。

use sasspile::{OutputStyle, compile_expanded, compile_file};
use std::path::PathBuf;

// —— 颜色函数关键字参数 ——

#[test]
fn test_adjust_color_kwarg() {
    let css = compile_expanded("a { b: adjust-color(red, $red: -50); }").unwrap();
    assert!(css.contains("#cd0000"), "should contain #cd0000: {css}");
}

#[test]
fn test_color_module_adjust() {
    let css =
        compile_expanded("@use \"sass:color\"; a { b: color.adjust(red, $red: -50); }").unwrap();
    assert!(css.contains("#cd0000"), "should contain #cd0000: {css}");
}

#[test]
fn test_change_color_kwarg() {
    let css = compile_expanded("a { b: change-color(red, $red: 100); }").unwrap();
    assert!(css.contains("#64"), "should contain changed red: {css}");
}

#[test]
fn test_scale_color_kwarg() {
    let css = compile_expanded("a { b: scale-color(red, $lightness: 50%); }").unwrap();
    assert!(
        !css.contains("scale-color"),
        "should not contain raw function name: {css}"
    );
}

// —— @forward as prefix-* ——

#[test]
fn test_forward_as_prefix_variable() {
    let dir = std::env::temp_dir().join("sasspile_fwd_as_var");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_upstream.scss"), "$c: e;\n").unwrap();
    std::fs::write(
        dir.join("_midstream.scss"),
        "@forward \"upstream\" as d-*;\n",
    )
    .unwrap();
    let main = dir.join("main.scss");
    std::fs::write(&main, "@use \"midstream\";\na {b: midstream.$d-c}\n").unwrap();
    let css = compile_file(&main, OutputStyle::Expanded).unwrap();
    assert!(css.contains("b: e"), "should contain b: e: {css}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_forward_as_prefix_function() {
    let dir = std::env::temp_dir().join("sasspile_fwd_as_fn");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_upstream.scss"), "@function c() {@return e}\n").unwrap();
    std::fs::write(
        dir.join("_midstream.scss"),
        "@forward \"upstream\" as d-*;\n",
    )
    .unwrap();
    let main = dir.join("main.scss");
    std::fs::write(&main, "@use \"midstream\";\na {b: midstream.d-c()}\n").unwrap();
    let css = compile_file(&main, OutputStyle::Expanded).unwrap();
    assert!(css.contains("b: e"), "should contain b: e: {css}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_forward_as_prefix_underscore() {
    let dir = std::env::temp_dir().join("sasspile_fwd_as_us");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_upstream.scss"), "$c: e;\n").unwrap();
    std::fs::write(
        dir.join("_midstream.scss"),
        "@forward \"upstream\" as d_*;\n",
    )
    .unwrap();
    let main = dir.join("main.scss");
    std::fs::write(&main, "@use \"midstream\";\na {b: midstream.$d_c}\n").unwrap();
    let css = compile_file(&main, OutputStyle::Expanded).unwrap();
    assert!(css.contains("b: e"), "should contain b: e: {css}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_forward_as_prefix_mixin() {
    let dir = std::env::temp_dir().join("sasspile_fwd_as_mx");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_upstream.scss"), "@mixin a() {c {d: e}}\n").unwrap();
    std::fs::write(
        dir.join("_midstream.scss"),
        "@forward \"upstream\" as b-*;\n",
    )
    .unwrap();
    let main = dir.join("main.scss");
    std::fs::write(&main, "@use \"midstream\";\n@include midstream.b-a;\n").unwrap();
    let css = compile_file(&main, OutputStyle::Expanded).unwrap();
    assert!(css.contains("d: e"), "should contain d: e: {css}");
    std::fs::remove_dir_all(&dir).ok();
}

// —— load path ——

#[test]
fn test_compile_load_path() {
    let spec_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");
    let utils_path = spec_root.join("core_functions/list/_utils.scss");
    if !utils_path.exists() {
        return;
    }
    let tmp = std::env::temp_dir().join("sasspile_test_loadpath");
    std::fs::create_dir_all(&tmp).unwrap();
    let input = tmp.join("input.scss");
    std::fs::write(
        &input,
        "@use \"core_functions/list/utils\";\na {b: utils.real-separator(())}\n",
    )
    .unwrap();
    let result =
        sasspile::compile_file_with_load_paths(&input, OutputStyle::Expanded, vec![spec_root]);
    std::fs::remove_dir_all(&tmp).ok();
    match result {
        Ok(css) => assert!(css.contains("undecided"), "should contain undecided: {css}"),
        Err(e) => panic!("load path test failed: {e}"),
    }
}
