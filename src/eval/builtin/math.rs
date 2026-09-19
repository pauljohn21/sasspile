//! Math 内建函数。
//!
//! 包含 abs/ceil/floor/round/min/max/percentage/div/pow/sqrt/sin/cos/tan/
//! atan2/asin/acos/atan/hypot/log/random/clamp/unit/is-unitless/compatible/comparable。
//! CSS round/mod/rem 函数：css_round（1-3 `参数+策略+单位转换）、css_mod（floored）、css_rem（truncated`）。
//!
//! 支持命名参数（如 `math.abs($number: 3)`、`math.clamp($min: 0, $number: 1, $max: 2)`）。
//! 辅助函数（参数名映射、合并、验证）在 `math_helpers` 模块中。

use super::super::Evaluator;
use super::math_css::{css_mod, css_rem, unit_conversion_factor};
use super::math_helpers::{merge_math_args, validate_single_number};
use crate::error::{Result, SassError};
use crate::parse::ast::*;
use imbl::HashMap;
use std::fmt::Write;

/// Math 函数分派。返回 Ok(Some(value)) 表示已处理，Ok(None) 表示不匹配。
pub fn call(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
) -> Result<Option<Value>> {
    let raw_args = pos_args; // 保留原始 pos_args 用于策略检测
    let args = merge_math_args(pos_args, kw_args, name);
    let args = args.as_slice();

    match name {
        "abs" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, u) => Ok(Some(Value::Number(n.abs(), u.clone()))),
                Value::Calc(c) => {
                    let inner = c
                        .strip_prefix("calc(")
                        .and_then(|s| s.strip_suffix(")"))
                        .unwrap_or(c.as_str());
                    Ok(Some(Value::String(format!("abs({inner})"), false)))
                }
                _ => Err(SassError::Eval("$number is not a number.".into())),
            }
        }
        "ceil" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, u) => Ok(Some(Value::Number(n.ceil(), u.clone()))),
                Value::Calc(c) => {
                    let inner = c
                        .strip_prefix("calc(")
                        .and_then(|s| s.strip_suffix(")"))
                        .unwrap_or(c.as_str());
                    Ok(Some(Value::String(format!("ceil({inner})"), false)))
                }
                _ => Err(SassError::Eval("$number is not a number.".into())),
            }
        }
        "floor" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, u) => Ok(Some(Value::Number(n.floor(), u.clone()))),
                Value::Calc(c) => {
                    let inner = c
                        .strip_prefix("calc(")
                        .and_then(|s| s.strip_suffix(")"))
                        .unwrap_or(c.as_str());
                    Ok(Some(Value::String(format!("floor({inner})"), false)))
                }
                _ => Err(SassError::Eval("$number is not a number.".into())),
            }
        }
        "round" => {
            // CSS round(strategy, number, step?) — 检测策略取整形式
            // 注意：args 是 merge_math_args 结果（命名的 $number 会映射到 args[0]）
            // 策略取整只在原始 pos_args 上检测（命名参数形式不可能有策略字符串）
            match raw_args {
                [Value::String(s, _), ..] if is_round_strategy(s) => {
                    // round(strategy, number, step?) — 使用原始 pos_args 分派
                    match raw_args.len() {
                        2 | 3 => round_strategy(raw_args),
                        n => Err(SassError::Eval(format!(
                            "round() strategy form expects 2 or 3 arguments, got {n}."
                        ))),
                    }
                }
                _ => {
                    // 传统 1 参数 round(x) 或 round($number: x)
                    match args.len() {
                        0 => Err(SassError::Eval("Missing argument $number.".into())),
                        1 => match &args[0] {
                            Value::Number(n, u) => Ok(Some(Value::Number(n.round(), u.clone()))),
                            Value::Calc(c) => {
                                let inner = c
                                    .strip_prefix("calc(")
                                    .and_then(|s| s.strip_suffix(")"))
                                    .unwrap_or(c.as_str());
                                Ok(Some(Value::String(format!("round({inner})"), false)))
                            }
                            _ => Err(SassError::Eval("$number is not a number.".into())),
                        },
                        n => Err(SassError::Eval(format!(
                            "Only 1 argument allowed, but {n} {} passed.",
                            match n == 1 { true => "was", false => "were" }
                        ))),
                    }
                }
            }
        }
        "mod" => {
            // CSS mod(number, step) — floored modulo
            match args.len() {
                2 => css_mod(&args[0], &args[1]),
                n => Err(SassError::Eval(format!(
                    "mod() expects 2 arguments, got {n}."
                ))),
            }
        }
        "rem" => {
            // CSS rem(number, step) — truncated modulo
            match args.len() {
                2 => css_rem(&args[0], &args[1]),
                n => Err(SassError::Eval(format!(
                    "rem() expects 2 arguments, got {n}."
                ))),
            }
        }
        "min" | "max" => {
            let is_min = name == "min";
            match args.is_empty() {
                true => {
                    return Err(SassError::Eval(format!(
                        "{name} requires at least 1 argument"
                    )))
                }
                false => {}
            }
            // 提取所有数字参数
            let numbers: Vec<(f64, Option<String>)> = args
                .iter()
                .map(|arg| match arg {
                    Value::Number(n, u) => Ok((*n, u.clone())),
                    _ => Err(SassError::Eval(format!("{name} requires number arguments"))),
                })
                .collect::<Result<Vec<_>>>()?;
            // 检查所有参数的单位互相兼容；任何一对不兼容即报错
            {
                let units: Vec<Option<&str>> = numbers.iter().map(|(_, u)| u.as_deref()).collect();
                for i in 0..units.len() {
                    for j in (i + 1)..units.len() {
                        let a = units[i];
                        let b = units[j];
                        // unitless 可以混用；有单位且严格相同也 OK；不同但兼容组也 OK
                        let pair_compatible = match (a, b) {
                            (None, _) | (_, None) => true,
                            (Some(ua), Some(ub)) if ua == ub => true,
                            (Some(ua), Some(ub)) => crate::eval::value::units_compatible(Some(ua), Some(ub)),
                        };
                        if !pair_compatible {
                            let a_str = a.map(|u| format!("1{u}")).unwrap_or_else(|| "1".to_string());
                            let b_str = b.map(|u| format!("1{u}")).unwrap_or_else(|| "1".to_string());
                            return Err(SassError::Eval(format!(
                                "Incompatible units {a_str} and {b_str}."
                            )));
                        }
                    }
                }
            }
            // 找到最值参数：比较数值，兼容单位转换后比数值
            // 返回获胜者的（原始值，原始单位）
            let (winner_val, winner_unit) = numbers.iter().skip(1).fold(
                (numbers[0].0, numbers[0].1.clone()),
                |(best_val, best_unit), (val, unit)| {
                    let compatible = crate::eval::value::units_compatible(
                        best_unit.as_deref(),
                        unit.as_deref(),
                    );
                    let val_for_compare: f64 = match (&best_unit, unit.as_deref()) {
                        (Some(bu), Some(u)) if compatible && bu != u => {
                            val * crate::eval::builtin::math_css::unit_conversion_factor(u, bu)
                        }
                        _ => *val,
                    };
                    let should_replace = if is_min {
                        val_for_compare < best_val
                    } else {
                        val_for_compare > best_val
                    };
                    match should_replace {
                        true => (*val, unit.clone()),
                        false => (best_val, best_unit),
                    }
                },
            );
            Ok(Some(Value::Number(winner_val, winner_unit)))
        }
        "percentage" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, None) => Ok(Some(Value::Number(n * 100.0, Some("%".into())))),
                Value::Number(n, Some(u)) => Err(SassError::Eval(format!(
                    "$number: Expected {n} to have no units but it has {u}."
                ))),
                _ => Err(SassError::Eval("$number is not a number.".into())),
            }
        }
        "div" => {
            match args.len() {
                0 => return Err(SassError::Eval("Missing argument $number1.".into())),
                1 => return Err(SassError::Eval("Missing argument $number2.".into())),
                2 => {}
                n => return Err(SassError::Eval(format!(
                    "Only 2 arguments allowed, but {n} were passed."
                ))),
            }
            // 非数字参数返回字符串形式（如 6/b）
            let a_is_num = matches!(&args[0], Value::Number(..));
            let b_is_num = matches!(&args[1], Value::Number(..));
            if !a_is_num || !b_is_num {
                let a_str = match &args[0] {
                    Value::Number(n, u) => format!("{n}{}", u.as_deref().unwrap_or("")),
                    other => other.to_string(),
                };
                let b_str = match &args[1] {
                    Value::Number(n, u) => format!("{n}{}", u.as_deref().unwrap_or("")),
                    other => other.to_string(),
                };
                return Ok(Some(Value::String(format!("{a_str}/{b_str}"), false)));
            }
            let (a, u1) = match &args[0] {
                Value::Number(n, u) => (*n, u.clone()),
                _ => unreachable!(),
            };
            let (b, u2) = match &args[1] {
                Value::Number(n, u) => (*n, u.clone()),
                _ => unreachable!(),
            };
            // div by zero handling
            match (b == 0.0, a == 0.0) {
                (true, true) => {
                    // 0/0 = NaN
                    match u1.is_some() || u2.is_some() {
                        true => {
                            let mut calc = String::from("calc(NaN");
                            if let Some(u) = u1.as_ref() {
                                if !u.is_empty() {
                                    let _ = write!(calc, " * 1{u}");
                                }
                            }
                            if let Some(u) = u2.as_ref() {
                                if !u.is_empty() {
                                    let _ = write!(calc, " / 1{u}");
                                }
                            }
                            calc.push(')');
                            Ok(Some(Value::Calc(calc)))
                        }
                        false => Ok(Some(Value::Number(f64::NAN, None))),
                    }
                }
                (true, false) => {
                    // n/0 = infinity
                    match u1.is_some() || u2.is_some() {
                        true => {
                            let sign = match a < 0.0 { true => "-", false => "" };
                            let mut calc = format!("calc({sign}infinity");
                            if let Some(u) = u1.as_ref() {
                                if !u.is_empty() {
                                    let _ = write!(calc, " * 1{u}");
                                }
                            }
                            if let Some(u) = u2.as_ref() {
                                if !u.is_empty() {
                                    let _ = write!(calc, " / 1{u}");
                                }
                            }
                            calc.push(')');
                            Ok(Some(Value::Calc(calc)))
                        }
                        false => {
                            let val = match a < 0.0 {
                                true => f64::NEG_INFINITY,
                                false => f64::INFINITY,
                            };
                            Ok(Some(Value::Number(val, None)))
                        }
                    }
                }
                (false, _) => {
                    // 单位处理：相同单位 → unitless；不同兼容单位 → 转换
                    match (u1.as_deref(), u2.as_deref()) {
                        (Some(u1_str), Some(u2_str)) if u1_str == u2_str => {
                            // 相同单位，结果 unitless
                            Ok(Some(Value::Number(a / b, None)))
                        }
                        (Some(u1_str), Some(u2_str)) => {
                            // 不同兼容单位，转换
                            let factor = crate::eval::builtin::math_css::unit_conversion_factor(u2_str, u1_str);
                            Ok(Some(Value::Number(a / (b * factor), Some(u1_str.to_string()))))
                        }
                        _ => Ok(Some(Value::Number(a / b, u1.clone()))),
                    }
                }
            }
        }
        "pow" | "sqrt" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "atan2" | "log"
        | "hypot" | "exp" | "sign" => super::math_trig::call(name, args),
        "random" => {
            let _span = tracing::info_span!("math_random", limit = ?args.first()).entered();
            // SCSS spec: null argument ≡ no argument → returns [0, 1)
            let args: Vec<&Value> = args
                .iter()
                .filter(|v| !matches!(v, Value::Null))
                .collect();
            match args.len() > 1 {
                true => return Err(SassError::Eval(format!(
                    "Only 1 argument allowed, but {} {} passed.",
                    args.len(),
                    match args.len() == 1 { true => "was", false => "were" }
                ))),
                false => {}
            }
            match args.as_slice() {
                [] => Ok(Some(Value::Number(Evaluator::simple_random(), None))),
                [Value::Number(n, _)] => {
                    match *n <= 0.0 {
                        true => return Err(SassError::Eval(format!(
                            "$limit: {n} must be a positive integer."
                        ))),
                        false => {}
                    }
                    // Epsilon tolerance: values within 1e-9 of an integer are treated as that integer
                    let rounded = n.round();
                    match (n - rounded).abs() > 1e-9 {
                        true => return Err(SassError::Eval(format!(
                            "$limit: {n} is not an int."
                        ))),
                        false => {}
                    }
                    Ok(Some(Value::Number(
                        (Evaluator::simple_random() * rounded).floor() + 1.0,
                        None,
                    )))
                }
                [other] => Err(SassError::Eval(format!("$limit: {other} is not a number."))),
                _ => Err(SassError::Eval("$number is not a number.".into())),
            }
        }
        "clamp" => {
            match args.len() {
                0 => return Err(SassError::Eval("Missing argument $min.".into())),
                1 => return Err(SassError::Eval("Missing argument $number.".into())),
                2 => return Err(SassError::Eval("Missing argument $max.".into())),
                3 => call_clamp(&args[0], &args[1], &args[2]),
                n => Err(SassError::Eval(format!(
                    "Only 3 arguments allowed, but {n} were passed."
                ))),
            }
        }
        "unit" => {
            validate_single_number(args)?;
            match &args[0] {
                // 返回带引号的字符串，如 "px"、"px*em" 等
                Value::Number(_, Some(u)) => Ok(Some(Value::String(u.clone(), true))),
                Value::Number(_, None) => Ok(Some(Value::String(String::new(), true))),
                Value::Calc(_) => Err(SassError::Eval(
                    "$number: calc expressions can't be used to determine units.".into(),
                )),
                _ => Err(SassError::Eval("$number is not a number.".into())),
            }
        }
        "is-unitless" => {
            match args.len() {
                0 => return Err(SassError::Eval("Missing argument $number.".into())),
                1 => {}
                n => return Err(SassError::Eval(format!(
                    "Only 1 argument allowed, but {n} {} passed.",
                    match n == 1 { true => "was", false => "were" }
                ))),
            }
            match &args[0] {
                Value::Number(_, None) => Ok(Some(Value::Bool(true))),
                Value::Number(_, Some(_)) => Ok(Some(Value::Bool(false))),
                other => Err(SassError::Eval(format!(
                    "$number: {other} is not a number."
                ))),
            }
        }
        "compatible" => {
            match args.len() {
                0 => return Err(SassError::Eval("Missing argument $number1.".into())),
                1 => return Err(SassError::Eval("Missing argument $number2.".into())),
                2 => {}
                n => return Err(SassError::Eval(format!(
                    "Only 2 arguments allowed, but {n} were passed."
                ))),
            }
            let u1 = match &args[0] {
                Value::Number(_, u) => u.clone(),
                other => {
                    return Err(SassError::Eval(format!(
                        "$number1: {other} is not a number."
                    )));
                }
            };
            let u2 = match &args[1] {
                Value::Number(_, u) => u.clone(),
                other => {
                    return Err(SassError::Eval(format!(
                        "$number2: {other} is not a number."
                    )));
                }
            };
            Ok(Some(Value::Bool(crate::eval::value::units_compatible(
                u1.as_deref(),
                u2.as_deref(),
            ))))
        }
        _ => Ok(None),
    }
}

/// 内部辅助：clamp(MIN, VAL, MAX)——CSS Values Level 4 兼容实现。
///
/// 核心规则：
/// 1. 三参数单位必须兼容（同为 unitless 或同类兼容单位）
/// 2. min > max 时 CSS spec 自动 clamp 等于 min
/// 3. VAL 先转换到 MIN 单位域，clamp 后按胜出区段决定输出单位：
///    - VAL 被 MIN 夹住 → 保留 MIN 单位
///    - VAL 被 MAX 夹住 → 保留 MAX 单位
///    - VAL 在区间内 → 保留 VAL 原单位
#[tracing::instrument(skip(min, val, max), fields(result = tracing::field::Empty, reason = tracing::field::Empty))]
fn call_clamp(min: &Value, val: &Value, max: &Value) -> Result<Option<Value>> {
    let (min_n, u_min) = match min {
        Value::Number(n, u) => (*n, u.clone()),
        non_num => return Err(SassError::Eval(format!("$min: {non_num} is not a number."))),
    };
    let (val_n, u_val) = match val {
        Value::Number(n, u) => (*n, u.clone()),
        non_num => return Err(SassError::Eval(format!("$number: {non_num} is not a number."))),
    };
    let (max_n, u_max) = match max {
        Value::Number(n, u) => (*n, u.clone()),
        non_num => return Err(SassError::Eval(format!("$max: {non_num} is not a number."))),
    };

    // 类型一致性检查（unitless 与有单位不可混用）
    let min_unitless = u_min.is_none();
    let val_unitless = u_val.is_none();
    let max_unitless = u_max.is_none();
    if min_unitless != val_unitless || val_unitless != max_unitless {
        return Err(SassError::Eval("Incompatible units.".to_string()));
    }

    // 有单位时校验兼容组
    if !min_unitless {
        if !crate::eval::value::units_compatible(u_min.as_deref(), u_val.as_deref())
            || !crate::eval::value::units_compatible(u_val.as_deref(), u_max.as_deref())
        {
            return Err(SassError::Eval("Incompatible units.".to_string()));
        }
    }

    // 转换 VAL 和 MAX 到 MIN 单位域（以便在同一基准比较）
    let val_in_min_unit = match (&u_val, &u_min) {
        (Some(uv), Some(um)) if uv != um => val_n * unit_conversion_factor(uv, um),
        _ => val_n,
    };
    let max_in_min_unit = match (&u_max, &u_min) {
        (Some(ux), Some(um)) if ux != um => max_n * unit_conversion_factor(ux, um),
        _ => max_n,
    };

    // CSS clamp(MIN, VAL, MAX) = max(MIN, min(VAL, MAX))
    // MIN > MAX 时，CSS spec 规定 clamp 回退等于 MIN
    // 比较时先判断区段，按胜出者保留对应值和原始单位（避免 float == 比较）
    let (result_n, result_u, reason) = if val_in_min_unit <= min_n || min_n > max_in_min_unit {
        // VAL 低于下限（或 min > max）：胜出 = MIN
        (min_n, u_min, "clamped-to-min")
    } else if val_in_min_unit >= max_in_min_unit {
        // VAL 高于上限：胜出 = MAX
        (max_n, u_max, "clamped-to-max")
    } else {
        // VAL 在区间内：保留原值原单位
        (val_n, u_val, "within-range")
    };

    let result_str = format!("{result_n}{}", result_u.as_deref().unwrap_or(""));
    tracing::Span::current().record("result", &result_str);
    tracing::Span::current().record("reason", reason);

    Ok(Some(Value::Number(result_n, result_u)))
}

/// CSS round(strategy, number, step?) 策略取整。
/// strategy: "up" | "down" | "nearest" | "to-zero"
/// 带 step: result = step * round(strategy, number / step)
fn round_strategy(args: &[Value]) -> Result<Option<Value>> {
    // 解析 strategy（第一参数必须为策略字符串）
    let strategy = match &args[0] {
        Value::String(s, _) => s.trim().to_string(),
        other => {
            return Err(SassError::Eval(format!(
                "$strategy: {other} is not a valid rounding strategy."
            )))
        }
    };

    // 解析 number（第二参数必须为数字）
    let (number, num_unit) = match &args[1] {
        Value::Number(n, u) => (*n, u.clone()),
        other => {
            return Err(SassError::Eval(format!(
                "$number: {other} is not a number."
            )))
        }
    };

    // 可选 step（第三参数）— 校验单位兼容性
    let step = match args.len() {
        3 => match &args[2] {
            Value::Number(s, u) => {
                // 单位兼容性校验
                match (num_unit.as_deref(), u.as_deref()) {
                    (Some(nu), Some(su)) if nu != su => {
                        return Err(SassError::Eval(format!(
                            "$step: {s}{su} and $number: {number}{nu} have incompatible units."
                        )));
                    }
                    _ => {}
                }
                // step != 0 校验
                if *s == 0.0 {
                    return Err(SassError::Eval(
                        "$step: cannot be zero.".into(),
                    ));
                }
                Some((*s, u.clone()))
            }
            other => {
                return Err(SassError::Eval(format!(
                    "$step: {other} is not a number."
                )))
            }
        },
        _ => None,
    };

    // 处理 infinity/NaN 特殊值
    if number.is_nan() {
        return Ok(Some(Value::Number(f64::NAN, num_unit)));
    }
    if number.is_infinite() {
        return Ok(Some(Value::Number(number, num_unit)));
    }

    // 策略分派
    let result = match step {
        Some((step_val, _step_unit)) => {
            let scaled = number / step_val;
            let rounded = apply_single_strategy(&strategy, scaled)?;
            rounded * step_val
        }
        None => apply_single_strategy(&strategy, number)?,
    };

    Ok(Some(Value::Number(result, num_unit)))
}

/// 判断字符串是否为合法的 round 策略名。
fn is_round_strategy(s: &str) -> bool {
    let t = s.trim();
    t == "up" || t == "down" || t == "nearest" || t == "to-zero"
}

/// 对单个数字应用取整策略。
fn apply_single_strategy(strategy: &str, n: f64) -> Result<f64> {
    match strategy {
        "up" => Ok(n.ceil()),
        "down" => Ok(n.floor()),
        "nearest" => Ok(n.round()),
        "to-zero" => Ok(n.trunc()),
        _ => Err(SassError::Eval(format!(
            "$strategy: {strategy} is not a valid rounding strategy. Expected up, down, nearest, or to-zero."
        ))),
    }
}
