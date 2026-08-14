use super::*;
use crate::error::{Result, SassError};
use crate::parse::ast::BinOpKind;

pub(crate) fn add(l: &Value, r: &Value) -> Result<Value> {
    let l = l.clone();
    let r = r.clone();
    match (l, r) {
        (Value::Number(a, u1), Value::Number(b, u2)) => {
            let unit = u1.or(u2);
            Ok(Value::Number(a + b, unit))
        }
        // 字符串拼接——结果引号跟随左侧
        (Value::String(a, qa), Value::String(b, _)) => Ok(Value::String(format!("{a}{b}"), qa)),
        (Value::String(a, qa), Value::Number(n, u)) => Ok(Value::String(
            format!("{a}{}{}", n, u.as_deref().unwrap_or("")),
            qa,
        )),
        (Value::String(a, qa), Value::Color(c)) => Ok(Value::String(
            format!("{a}#{:02x}{:02x}{:02x}", c.r, c.g, c.b),
            qa,
        )),
        (Value::String(a, qa), Value::Null) => Ok(Value::String(a, qa)),
        (Value::Number(n, u), Value::String(b, qb)) => Ok(Value::String(
            format!("{}{}{b}", n, u.as_deref().unwrap_or("")),
            qb,
        )),
        (Value::Color(c), Value::String(b, qb)) => Ok(Value::String(
            format!("#{:02x}{:02x}{:02x}{b}", c.r, c.g, c.b),
            qb,
        )),
        (Value::Null, Value::String(b, qb)) => Ok(Value::String(b, qb)),
        // String + Calc / Calc + String — 拼接字符串表示
        (Value::String(a, qa), Value::Calc(c)) => Ok(Value::String(format!("{a}{c}"), qa)),
        (Value::Calc(c), Value::String(b, qb)) => Ok(Value::String(format!("{c}{b}"), qb)),
        (Value::Calc(a), Value::Calc(b)) => Ok(Value::Raw(format!("{a}{b}"))),
        // String + Bool / Bool + String
        (Value::String(a, qa), Value::Bool(b)) => Ok(Value::String(format!("{a}{b}"), qa)),
        (Value::Bool(a), Value::String(b, qb)) => Ok(Value::String(format!("{a}{b}"), qb)),
        // 列表拼接
        (Value::List(mut items, sep, _), Value::List(items2, _, _)) => {
            items.extend(items2);
            Ok(Value::List(items, sep, false))
        }
        (Value::List(mut items, sep, _), other) => {
            items.push(other);
            Ok(Value::List(items, sep, false))
        }
        (other, Value::List(items, sep, false)) => {
            let mut new_items = vec![other];
            new_items.extend(items);
            Ok(Value::List(new_items, sep, false))
        }
        _ => Err(SassError::Eval("不支持的 + 运算".into())),
    }
}

pub(crate) fn sub(l: &Value, r: &Value) -> Result<Value> {
    let l = l.clone();
    let r = r.clone();
    match (l, r) {
        (Value::Number(a, u1), Value::Number(b, u2)) => {
            let unit = u1.or(u2);
            Ok(Value::Number(a - b, unit))
        }
        // 字符串拼接——用 - 连接
        (Value::String(a, qa), Value::String(b, _)) => {
            Ok(Value::String(format!("{a}-{b}"), qa))
        }
        (Value::String(a, qa), Value::Number(n, u)) => Ok(Value::String(
            format!("{a}-{}{}", n, u.as_deref().unwrap_or("")),
            qa,
        )),
        (Value::String(a, qa), Value::Color(c)) => Ok(Value::String(
            format!("{a}-#{:02x}{:02x}{:02x}", c.r, c.g, c.b),
            qa,
        )),
        (Value::Number(n, u), Value::String(b, qb)) => Ok(Value::String(
            format!("{}{}-{b}", n, u.as_deref().unwrap_or("")),
            qb,
        )),
        (Value::Color(c), Value::String(b, qb)) => Ok(Value::String(
            format!("#{:02x}{:02x}{:02x}-{b}", c.r, c.g, c.b),
            qb,
        )),
        _ => Err(SassError::Eval("不支持的 - 运算".into())),
    }
}

pub(crate) fn mul(l: &Value, r: &Value) -> Result<Value> {
    match (l, r) {
        (Value::Number(a, u1), Value::Number(b, u2)) => {
            // 当两边都有单位时，合并为复合单位（如 px*em）
            let unit = match (u1, u2) {
                (Some(u1), Some(u2)) => Some(format!("{u1}*{u2}")),
                (Some(u), None) | (None, Some(u)) => Some(u.clone()),
                (None, None) => None,
            };
            Ok(Value::Number(a * b, unit))
        }
        _ => Err(SassError::Eval(format!("无法 {l} * {r}"))),
    }
}

pub(crate) fn div(l: &Value, r: &Value) -> Result<Value> {
    match (l, r) {
        (Value::Number(a, u1), Value::Number(b, u2)) => {
            // 构建结果单位：分子单位/分母单位
            let result_unit = match (u1, u2) {
                (Some(n), Some(d)) => Some(format!("{n}/{d}")),
                (Some(n), None) => Some(n.clone()),
                (None, Some(d)) => Some(format!("/{d}")),
                (None, None) => None,
            };
            if *b == 0.0 {
                // SCSS: 1/0 = Infinity, -1/0 = -Infinity, 0/0 = NaN
                if *a == 0.0 {
                    return Ok(Value::Number(f64::NAN, result_unit));
                }
                return Ok(Value::Number(a / b, result_unit)); // f64 除零产生 Infinity
            }
            Ok(Value::Number(a / b, result_unit))
        }
        // 非数字 / —— 作为斜杠分隔列表保留（如 font: 16px/24px）
        // 简化简单 calc(N) → N
        _ => {
            let l_str = value_to_raw(l);
            let r_str = value_to_raw(r);
            Ok(Value::Raw(format!("{l_str}/{r_str}")))
        }
    }
}

pub(crate) fn modulo(l: &Value, r: &Value) -> Result<Value> {
    match (l, r) {
        (Value::Number(a, u), Value::Number(b, _)) => {
            if *b == 0.0 {
                // Sass: 1 % 0 = NaN（不报错）
                let unit = u.as_deref().unwrap_or("");
                if unit.is_empty() {
                    return Ok(Value::Number(f64::NAN, None));
                }
                return Ok(Value::Raw(format!("calc(NaN * 1{unit})")));
            }
            // 处理 infinity 情况：1px % Infinity = 1px, -1px % -Infinity = -1px
            // 符号不同时：-1px % Infinity = NaN, 1px % -Infinity = NaN
            if b.is_infinite() {
                if a == &0.0 || a.signum() == b.signum() {
                    return Ok(Value::Number(*a, u.clone()));
                } else {
                    let unit = u.as_deref().unwrap_or("");
                    return Ok(Value::Raw(format!("calc(NaN * 1{unit})")));
                }
            }
            // Dart Sass 使用向下取整除法语义：余数符号与除数相同
            // result = a - floor(a/b) * b
            let raw = a - (a / b).floor() * b;
            // 精度截断——消除浮点误差（如 0.8999999999999995 → 0.9）
            let result = (raw * 1e10).round() / 1e10;
            Ok(Value::Number(result, u.clone()))
        }
        // 处理右侧是 calc(infinity * 1px) 等情况
        (Value::Number(a, u), Value::Calc(s)) => {
            if s.contains("infinity") {
                let is_negative = s.starts_with("calc(-") || s.contains("-infinity");
                let a_sign = a.signum();
                if *a == 0.0 || (is_negative && a_sign < 0.0) || (!is_negative && a_sign > 0.0) {
                    Ok(Value::Number(*a, u.clone()))
                } else {
                    let unit = u.as_deref().unwrap_or("");
                    Ok(Value::Raw(format!("calc(NaN * 1{unit})")))
                }
            } else {
                // 非 infinity 的 calc —— 作为空格分隔列表保留
                Ok(Value::List(
                    vec![l.clone(), r.clone()],
                    Separator::Space,
                    false,
                ))
            }
        }
        // Null RHS — % 不是运算符，作为字符串保留
        (l, Value::Null) => Ok(Value::List(
            vec![l.clone(), Value::String("%".to_string(), false)],
            Separator::Space,
            false,
        )),
        // 非数字 % —— 作为空格分隔列表保留
        _ => Ok(Value::List(
            vec![l.clone(), r.clone()],
            Separator::Space,
            false,
        )),
    }
}

/// 将 Value 转换为原始 CSS 字符串，简化简单 calc(N) 为 N
fn value_to_raw(v: &Value) -> String {
    match v {
        Value::Calc(s) => {
            // 尝试简化 calc(N) → N
            if let Some(inner) = s.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")) {
                let inner = inner.trim();
                if let Ok(n) = inner.parse::<f64>() {
                    if n.fract() == 0.0 {
                        return format!("{}", n as i64);
                    } else {
                        return format!("{n}");
                    }
                }
            }
            s.clone()
        }
        _ => v.to_string(),
    }
}

pub(crate) fn compare(op: &BinOpKind, l: &Value, r: &Value) -> Result<Value> {
    match (l, r) {
        (Value::Number(a, _), Value::Number(b, _)) => {
            let result = match op {
                BinOpKind::Lt => a < b,
                BinOpKind::Gt => a > b,
                BinOpKind::LtEq => a <= b,
                BinOpKind::GtEq => a >= b,
                _ => false,
            };
            Ok(Value::Bool(result))
        }
        _ => Err(SassError::Eval(format!("无法比较 {l} 和 {r}"))),
    }
}

/// 检查两个单位是否兼容（属于同一物理量类别）。
pub(crate) fn units_compatible(u1: Option<&str>, u2: Option<&str>) -> bool {
    if u1 == u2 {
        return true;
    }
    if u1.is_none() || u2.is_none() {
        return true;
    }
    // 单位兼容组——同组的单位互相兼容
    const GROUPS: &[&[&str]] = &[
        &["px", "in", "cm", "mm", "pt", "pc", "q"], // 长度
        &["deg", "grad", "rad", "turn"],            // 角度
        &["s", "ms"],                               // 时间
        &["hz", "khz"],                             // 频率
        &["dpi", "dpcm", "dppx"],                   // 分辨率
    ];
    for group in GROUPS {
        let has1 = group.contains(&u1.unwrap());
        let has2 = group.contains(&u2.unwrap());
        if has1 && has2 {
            return true;
        }
    }
    false
}

pub(crate) fn values_eq(l: &Value, r: &Value) -> bool {
    match (l, r) {
        (Value::Number(a, _), Value::Number(b, _)) => {
            // IEEE 754：NaN != NaN
            if a.is_nan() || b.is_nan() {
                return false;
            }
            if a.is_infinite() && b.is_infinite() && a.signum() == b.signum() {
                return true;
            }
            (a - b).abs() < f64::EPSILON
        }
        (Value::String(a, _), Value::String(b, _)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Color(a), Value::Color(b)) => a == b,
        (Value::Null, Value::Null) => true,
        (Value::List(a, sa, ba), Value::List(b, sb, bb)) => {
            // 列表相等：长度、分隔符、括号状态、每个元素都相同
            sa == sb && ba == bb && a.len() == b.len()
                && a.iter().zip(b.iter()).all(|(x, y)| values_eq(x, y))
        }
        (Value::Map(a), Value::Map(b)) => {
            a.len() == b.len()
                && a.iter().all(|(k, v)| {
                    b.iter()
                        .any(|(k2, v2)| values_eq(k, k2) && values_eq(v, v2))
                })
        }
        _ => false,
    }
}
