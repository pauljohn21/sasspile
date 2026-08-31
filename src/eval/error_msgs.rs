//! 错误消息模板——消除散布在 15+ 文件中的 `format!("... is not a string.")` 模式。
//!
//! 所有内建函数统一调用此模块生成错误，保证消息一致性。

use crate::error::SassError;
use crate::parse::ast::Value;

/// 生成 `$param: value is not a string.` 错误。
pub fn err_not_a_string(param: &str, val: &Value) -> SassError {
    SassError::Eval(format!("${param}: {val} is not a string."))
}

/// 生成 `$param: value is not a number.` 错误。
pub fn err_not_a_number(param: &str, val: &Value) -> SassError {
    SassError::Eval(format!("${param}: {val} is not a number."))
}

/// 生成 `$param: value is not an int.` 错误。
pub fn err_not_an_int(param: &str, val: &Value) -> SassError {
    SassError::Eval(format!("${param}: {val} is not an int."))
}

/// 生成 `$param: value is not a color.` 错误。
pub fn err_not_a_color(param: &str, val: &Value) -> SassError {
    SassError::Eval(format!("${param}: {val} is not a color."))
}

/// 生成 `Missing argument $param.` 错误。
pub fn err_missing_arg(param: &str) -> SassError {
    SassError::Eval(format!("Missing argument ${param}."))
}

/// 生成 `Only N arguments allowed, but M were/was passed.` 错误。
pub fn err_wrong_arg_count(expected: usize, actual: usize) -> SassError {
    let verb = if actual == 1 { "was" } else { "were" };
    SassError::Eval(format!(
        "Only {expected} argument{} allowed, but {actual} {verb} passed.",
        if expected == 1 { "" } else { "s" }
    ))
}

/// 生成 `$param: Expected value to have no units.` 错误。
pub fn err_expected_no_units(param: &str, val: &Value) -> SassError {
    SassError::Eval(format!("${param}: Expected {val} to have no units."))
}

/// 生成 `$param: Expected value to be a quoted string.` 错误。
pub fn err_expected_quoted_string(param: &str, val: &Value) -> SassError {
    SassError::Eval(format!("${param}: Expected {val} to be a quoted string."))
}

/// 生成 `$param: Expected value to be an unquoted string.` 错误。
pub fn err_expected_unquoted_string(param: &str, val: &Value) -> SassError {
    SassError::Eval(format!("${param}: Expected {val} to be an unquoted string."))
}

/// 生成 `Only N arguments allowed, but M were passed.` 错误（复数专用版）。
pub fn err_wrong_arg_count_plural(expected: usize, actual: usize) -> SassError {
    SassError::Eval(format!(
        "Only {expected} arguments allowed, but {actual} were passed."
    ))
}

/// 生成 `$param: Must be 1 or greater, was N.` 错误。
pub fn err_must_be_positive(param: &str, val: i64) -> SassError {
    SassError::Eval(format!("${param}: Must be 1 or greater, was {val}."))
}

/// 生成 `$channel: Color X has no channel named Y.` 错误。
pub fn err_no_channel(color_name: &str, channel: &str) -> SassError {
    SassError::Eval(format!(
        "$channel: Color {color_name} has no channel named {channel}."
    ))
}

/// 生成 `$param: requires a number` 错误。
pub fn err_requires_a_number(param: &str) -> SassError {
    SassError::Eval(format!("${param} requires a number"))
}

/// 生成 `$param: Expected "value" to be an unquoted string.` 错误。
pub fn err_expected_unquoted_str_display(param: &str, val: &str) -> SassError {
    SassError::Eval(format!("${param}: Expected \"{val}\" to be an unquoted string."))
}

/// 生成 `$param: Expected value to be a quoted string.` 错误（Display 版本）。
pub fn err_expected_quoted_str_display(param: &str, val: &str) -> SassError {
    SassError::Eval(format!("${param}: Expected {val} to be a quoted string."))
}

/// 生成 `$space: Unknown color space: X.` 错误。
pub fn err_unknown_color_space(space: &str) -> SassError {
    SassError::Eval(format!("$space: Unknown color space: {space}."))
}

/// 生成 `$space: Unknown color space "X".` 错误（带引号变体）。
pub fn err_unknown_color_space_quoted(space: &str) -> SassError {
    SassError::Eval(format!("$space: Unknown color space \"{space}\"."))
}

/// 生成 `fn_name() requires N arguments, got M.` 错误。
pub fn err_requires_args(fn_name: &str, expected: usize, actual: usize) -> SassError {
    SassError::Eval(format!("{fn_name}() requires {expected} arguments, got {actual}"))
}

/// 生成 `There is no mixin named "name".` 错误。
pub fn err_no_mixin(name: &str) -> SassError {
    SassError::Eval(format!("There is no mixin named \"{name}\"."))
}

/// 生成 `There is no module with namespace "ns".` 错误。
pub fn err_no_module(ns: &str) -> SassError {
    SassError::Eval(format!("There is no module with namespace \"{ns}\"."))
}

/// 生成 `$param: Expected value to be exactly "X" or "Y".` 错误。
pub fn err_expected_exactly(param: &str, val: &str, options: &[&str]) -> SassError {
    let opts = options
        .iter()
        .map(|o| format!("\"{o}\""))
        .collect::<Vec<_>>()
        .join(" or ");
    SassError::Eval(format!("${param}: Expected {val} to be exactly {opts}."))
}

/// 生成 `This at-rule isn't allowed in plain CSS.` 错误。
pub fn err_plain_css_at_rule() -> SassError {
    SassError::Eval("This at-rule isn't allowed in plain CSS.".into())
}

/// 生成 `Silent comments aren't allowed in plain CSS.` 错误。
pub fn err_plain_css_silent_comment() -> SassError {
    SassError::Eval("Silent comments aren't allowed in plain CSS.".into())
}

/// 生成 `Sass variables aren't allowed in plain CSS.` 错误。
pub fn err_plain_css_sass_var() -> SassError {
    SassError::Eval("Sass variables aren't allowed in plain CSS.".into())
}
