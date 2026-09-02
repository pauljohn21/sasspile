//! CSS round/mod/rem 函数 + 单位转换。
//!
//! - `css_round(strategy, number, step)`: CSS `round()` 四种舍入策略
//! - `css_mod(number, step)`: floored modulo
//! - `css_rem(number, step)`: truncated modulo
//! - `unit_conversion_factor`: 兼容单位间转换因子

use crate::error::{Result, SassError};
use crate::parse::ast::*;

/// CSS round(strategy, number, step) 函数。
///
/// 根据 strategy 将 number 舍入到 step 的倍数：
/// - nearest: 最接近的倍数（默认）
/// - up: 向上舍入
/// - down: 向下舍入
/// - to-zero: 向零舍入
#[allow(clippy::pedantic)]
pub(crate) fn css_round(strategy: &str, number: &Value, step: &Value) -> Result<Option<Value>> {
    let (n, n_unit) = match number {
        Value::Number(n, u) => (*n, u.clone()),
        _ => {
            return Err(SassError::Eval(format!(
                "$number: {number} is not a number."
            )));
        }
    };
    let (s, s_unit) = match step {
        Value::Number(s, u) => (*s, u.clone()),
        _ => return Err(SassError::Eval(format!("$step: {step} is not a number."))),
    };
    if s == 0.0 {
        return Err(SassError::Eval("Round step cannot be zero.".into()));
    }
    let compatible = crate::eval::value::units_compatible(n_unit.as_deref(), s_unit.as_deref());
    if !compatible {
        let n_str = match &n_unit {
            Some(u) => format!("{n}{u}"),
            None => n.to_string(),
        };
        let s_str = match &s_unit {
            Some(u) => format!("{s}{u}"),
            None => s.to_string(),
        };
        return Ok(Some(Value::String(
            format!("round({strategy}, {n_str}, {s_str})"),
            false,
        )));
    }
    let (s_converted, out_unit) = match (&n_unit, &s_unit) {
        (None, None) => (s, None),
        (Some(u), None) => (s, Some(u.clone())),
        (None, Some(u)) => (s, Some(u.clone())),
        (Some(nu), Some(su)) if nu == su => (s, Some(nu.clone())),
        (Some(nu), Some(su)) => (s * unit_conversion_factor(su, nu), Some(nu.clone())),
    };
    let ratio = n / s_converted;
    let rounded = match strategy {
        "nearest" => ratio.round(),
        "up" => ratio.ceil(),
        "down" => ratio.floor(),
        "to-zero" => ratio.trunc(),
        _ => return Err(SassError::Eval(format!("Unknown strategy: {strategy}"))),
    };
    let result = rounded * s_converted;
    Ok(Some(Value::Number(result, out_unit)))
}

/// 获取从 `from_unit` 到 `to_unit` 的转换因子。
pub(crate) fn unit_conversion_factor(from: &str, to: &str) -> f64 {
    if from == to {
        return 1.0;
    }
    // 长度单位到 px 的转换因子
    const LENGTH_TO_PX: &[(&str, f64)] = &[
        ("px", 1.0),
        ("in", 96.0),
        ("cm", 96.0 / 2.54),
        ("mm", 96.0 / 25.4),
        ("pt", 96.0 / 72.0),
        ("pc", 96.0 / 6.0),
        ("q", 96.0 / 254.0),
    ];
    // 角度单位到 deg 的转换因子
    const ANGLE_TO_DEG: &[(&str, f64)] = &[
        ("deg", 1.0),
        ("grad", 0.9),
        ("rad", 180.0 / std::f64::consts::PI),
        ("turn", 360.0),
    ];
    // 时间单位到 s 的转换因子
    const TIME_TO_S: &[(&str, f64)] = &[("s", 1.0), ("ms", 0.001)];
    // 频率单位到 Hz 的转换因子
    const FREQ_TO_HZ: &[(&str, f64)] = &[("hz", 1.0), ("khz", 1000.0)];
    // 分辨率单位到 dpi 的转换因子
    const RES_TO_DPI: &[(&str, f64)] = &[("dpi", 1.0), ("dpcm", 2.54), ("dppx", 96.0)];
    for table in [
        LENGTH_TO_PX,
        ANGLE_TO_DEG,
        TIME_TO_S,
        FREQ_TO_HZ,
        RES_TO_DPI,
    ] {
        let from_f = table.iter().find(|(u, _)| *u == from).map(|(_, f)| *f);
        let to_f = table.iter().find(|(u, _)| *u == to).map(|(_, f)| *f);
        if let (Some(f), Some(t)) = (from_f, to_f) {
            return f / t;
        }
    }
    1.0 // 不兼容——不转换
}

/// CSS mod(number, step) — floored modulo。
/// 结果符号跟随 step 的符号。
#[allow(clippy::pedantic)]
pub(crate) fn css_mod(number: &Value, step: &Value) -> Result<Option<Value>> {
    let (n, n_unit) = match number {
        Value::Number(n, u) => (*n, u.clone()),
        _ => {
            return Err(SassError::Eval(format!(
                "$number: {number} is not a number."
            )));
        }
    };
    let (s, s_unit) = match step {
        Value::Number(s, u) => (*s, u.clone()),
        _ => return Err(SassError::Eval(format!("$step: {step} is not a number."))),
    };
    if s == 0.0 {
        return Err(SassError::Eval("mod() step cannot be zero.".into()));
    }
    let compatible = crate::eval::value::units_compatible(n_unit.as_deref(), s_unit.as_deref());
    if !compatible {
        let n_str = match &n_unit {
            Some(u) => format!("{n}{u}"),
            None => n.to_string(),
        };
        let s_str = match &s_unit {
            Some(u) => format!("{s}{u}"),
            None => s.to_string(),
        };
        return Ok(Some(Value::String(format!("mod({n_str}, {s_str})"), false)));
    }
    let (s_converted, out_unit) = match (&n_unit, &s_unit) {
        (None, None) => (s, None),
        (Some(u), None) => (s, Some(u.clone())),
        (None, Some(u)) => (s, Some(u.clone())),
        (Some(nu), Some(su)) if nu == su => (s, Some(nu.clone())),
        (Some(nu), Some(su)) => (s * unit_conversion_factor(su, nu), Some(nu.clone())),
    };
    // floored modulo: n - s * floor(n / s)
    let result = n - s_converted * (n / s_converted).floor();
    Ok(Some(Value::Number(result, out_unit)))
}

/// CSS rem(number, step) — truncated modulo。
/// 结果符号跟随 number 的符号。
#[allow(clippy::pedantic)]
pub(crate) fn css_rem(number: &Value, step: &Value) -> Result<Option<Value>> {
    let (n, n_unit) = match number {
        Value::Number(n, u) => (*n, u.clone()),
        _ => {
            return Err(SassError::Eval(format!(
                "$number: {number} is not a number."
            )));
        }
    };
    let (s, s_unit) = match step {
        Value::Number(s, u) => (*s, u.clone()),
        _ => return Err(SassError::Eval(format!("$step: {step} is not a number."))),
    };
    if s == 0.0 {
        return Err(SassError::Eval("rem() step cannot be zero.".into()));
    }
    let compatible = crate::eval::value::units_compatible(n_unit.as_deref(), s_unit.as_deref());
    if !compatible {
        let n_str = match &n_unit {
            Some(u) => format!("{n}{u}"),
            None => n.to_string(),
        };
        let s_str = match &s_unit {
            Some(u) => format!("{s}{u}"),
            None => s.to_string(),
        };
        return Ok(Some(Value::String(format!("rem({n_str}, {s_str})"), false)));
    }
    let (s_converted, out_unit) = match (&n_unit, &s_unit) {
        (None, None) => (s, None),
        (Some(u), None) => (s, Some(u.clone())),
        (None, Some(u)) => (s, Some(u.clone())),
        (Some(nu), Some(su)) if nu == su => (s, Some(nu.clone())),
        (Some(nu), Some(su)) => (s * unit_conversion_factor(su, nu), Some(nu.clone())),
    };
    // truncated modulo: n - s * trunc(n / s)
    let result = n - s_converted * (n / s_converted).trunc();
    Ok(Some(Value::Number(result, out_unit)))
}
