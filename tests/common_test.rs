//! common 模块的测试——diff_css 和 DiffResult。
//!
//! 从 tests/common/mod.rs 移出，避免使用 #[cfg(test)] 内联测试。

mod common;

use common::{DiffLine, diff_css};

#[test]
fn test_diff_identical() {
    let result = diff_css("a { color: red; }", "a { color: red; }");
    assert_eq!(result.lines.len(), 0);
    assert_eq!(result.classify(), "identical");
}

#[test]
fn test_diff_changed_line() {
    let result = diff_css("a { color: red; }", "a { color: blue; }");
    assert_eq!(result.lines.len(), 1);
    assert!(
        matches!(&result.lines[0], DiffLine::Changed { line: 1, expected, actual }
        if expected == "a { color: red; }" && actual == "a { color: blue; }")
    );
    assert_eq!(result.classify(), "content_diff");
}

#[test]
fn test_diff_missing_output() {
    let result = diff_css("a { color: red; }\nb { color: blue; }", "a { color: red; }");
    assert_eq!(result.lines.len(), 1);
    assert!(matches!(
        &result.lines[0],
        DiffLine::ExtraExpected { line: 2, .. }
    ));
    assert_eq!(result.classify(), "missing_output");
}

#[test]
fn test_diff_extra_output() {
    let result = diff_css("a { color: red; }", "a { color: red; }\nb { color: blue; }");
    assert_eq!(result.lines.len(), 1);
    assert!(matches!(
        &result.lines[0],
        DiffLine::ExtraActual { line: 2, .. }
    ));
    assert_eq!(result.classify(), "extra_output");
}

#[test]
fn test_format_terminal() {
    let result = diff_css("a { color: red; }", "a { color: blue; }");
    let formatted = result.format_terminal();
    assert!(formatted.contains("L1"));
    assert!(formatted.contains("red"));
    assert!(formatted.contains("blue"));
}
