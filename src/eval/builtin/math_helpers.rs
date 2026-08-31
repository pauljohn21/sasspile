//! Math 函数辅助工具——参数名映射、参数合并、参数验证。

use crate::error::{Result, SassError};
use crate::parse::ast::*;
use std::collections::HashMap;

/// 返回每个 math 函数的参数名列表（按位置顺序）。
/// 用于将命名参数（kw_args）按参数名映射到位置参数。
pub(crate) fn math_param_names(name: &str) -> &'static [&'static str] {
    match name {
        "abs" | "ceil" | "floor" | "round" | "sqrt" | "sin" | "cos" | "tan"
        | "asin" | "acos" | "atan" | "unit" | "is-unitless"
        | "percentage" => &["number"],
        "div" => &["number1", "number2"],
        "pow" => &["base", "exponent"],
        "atan2" => &["y", "x"],
        "log" => &["number", "base"],
        "clamp" => &["min", "number", "max"],
        "compatible" | "comparable" => &["number1", "number2"],
        "random" => &["limit"],
        // variadic：直接返回 pos_args
        "hypot" | "min" | "max" => &[],
        _ => &[],
    }
}

/// 将位置参数和命名参数合并为统一的位置参数列表。
/// 按 `param_names` 顺序填充：先取 pos_args 对应位置，不足的从 kw_args 按参数名查找。
pub(crate) fn merge_math_args(
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    name: &str,
) -> Vec<Value> {
    let param_names = math_param_names(name);
    if param_names.is_empty() {
        return pos_args.to_vec();
    }
    let mut result = Vec::with_capacity(param_names.len().max(pos_args.len()));
    for (i, pname) in param_names.iter().enumerate() {
        if i < pos_args.len() {
            result.push(pos_args[i].clone());
        } else if let Some(v) = kw_args.get(*pname) {
            result.push(v.clone());
        } else if let Some(v) = kw_args.get(&format!("${pname}")) {
            result.push(v.clone());
        }
    }
    // 追加多余的 pos_args（如 rest 参数场景）
    if pos_args.len() > param_names.len() {
        result.extend_from_slice(&pos_args[param_names.len()..]);
    }
    result
}

/// 验证单参数 math 函数的参数数量和类型。
/// 检查：空参数 → Missing argument $number；多参数 → Only 1 argument allowed；非数字 → $number is not a number。
pub(crate) fn validate_single_number(args: &[Value]) -> Result<()> {
    if args.is_empty() {
        return Err(SassError::Eval("Missing argument $number.".into()));
    }
    if args.len() > 1 {
        return Err(SassError::Eval(format!(
            "Only 1 argument allowed, but {} {} passed.",
            args.len(),
            if args.len() == 1 { "was" } else { "were" }
        )));
    }
    match &args[0] {
        Value::Number(..) | Value::Calc(..) => Ok(()),
        other => Err(SassError::Eval(format!(
            "$number: {} is not a number.", other
        ))),
    }
}
