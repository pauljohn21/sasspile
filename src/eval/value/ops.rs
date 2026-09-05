#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
use super::*;
use crate::error::{Result, SassError};
use crate::parse::ast::BinOpKind;

pub(crate) fn add(l: &Value, r: &Value) -> Result<Value> {
    let l = l.clone();
    let r = r.clone();
    match (l, r) {
        (Value::Number(a, u1), Value::Number(b, u2)) => {
            // 不同单位不兼容时报错（如 1px + 1em）
            match u1.is_some() && u2.is_some() && !units_compatible(u1.as_deref(), u2.as_deref()) {
                true => {
                    let u1_str = u1.as_deref().unwrap_or("");
                    let u2_str = u2.as_deref().unwrap_or("");
                    return Err(SassError::Eval(format!(
                        "{u1_str} and {u2_str} have incompatible units."
                    )));
                }
                false => {}
            }
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
            format!(
                "{a}#{:02x}{:02x}{:02x}",
                c.legacy_rgb[0].round() as u8,
                c.legacy_rgb[1].round() as u8,
                c.legacy_rgb[2].round() as u8
            ),
            qa,
        )),
        (Value::String(a, qa), Value::Null) => Ok(Value::String(a, qa)),
        (Value::Number(n, u), Value::String(b, qb)) => Ok(Value::String(
            format!("{}{}{b}", n, u.as_deref().unwrap_or("")),
            qb,
        )),
        (Value::Color(c), Value::String(b, qb)) => Ok(Value::String(
            format!(
                "#{:02x}{:02x}{:02x}{b}",
                c.legacy_rgb[0].round() as u8,
                c.legacy_rgb[1].round() as u8,
                c.legacy_rgb[2].round() as u8
            ),
            qb,
        )),
        (Value::Null, Value::String(b, qb)) => Ok(Value::String(b, qb)),
        // String + Calc / Calc + String — 拼接字符串表示
        (Value::String(a, qa), Value::Calc(c)) => Ok(Value::String(format!("{a}{c}"), qa)),
        (Value::Calc(c), Value::String(b, qb)) => Ok(Value::String(format!("{c}{b}"), qb)),
        (Value::Calc(a), Value::Calc(b)) => Ok(Value::String(format!("{a}{b}"), false)),
        // Number + Calc / Calc + Number — 作为 calc 表达式拼接
        (Value::Number(n, u), Value::Calc(c)) => {
            let n_str = format!("{n}{}", u.as_deref().unwrap_or(""));
            let c_inner = c
                .strip_prefix("calc(")
                .and_then(|s| s.strip_suffix(")"))
                .unwrap_or(c.as_str());
            Ok(Value::Calc(format!("calc({n_str} + {c_inner})")))
        }
        (Value::Calc(c), Value::Number(n, u)) => {
            let n_str = format!("{n}{}", u.as_deref().unwrap_or(""));
            let c_inner = c
                .strip_prefix("calc(")
                .and_then(|s| s.strip_suffix(")"))
                .unwrap_or(c.as_str());
            Ok(Value::Calc(format!("calc({c_inner} + {n_str})")))
        }
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
        _ => Err(SassError::Eval("Unsupported + operation".into())),
    }
}

pub(crate) fn sub(l: &Value, r: &Value) -> Result<Value> {
    let l = l.clone();
    let r = r.clone();
    match (l, r) {
        (Value::Number(a, u1), Value::Number(b, u2)) => {
            // 不同单位不兼容时报错（如 1px - 1em）
            match u1.is_some() && u2.is_some() && !units_compatible(u1.as_deref(), u2.as_deref()) {
                true => {
                    let u1_str = u1.as_deref().unwrap_or("");
                    let u2_str = u2.as_deref().unwrap_or("");
                    return Err(SassError::Eval(format!(
                        "{u1_str} and {u2_str} have incompatible units."
                    )));
                }
                false => {}
            }
            let unit = u1.or(u2);
            Ok(Value::Number(a - b, unit))
        }
        // 字符串拼接——用 - 连接
        (Value::String(a, qa), Value::String(b, _)) => Ok(Value::String(format!("{a}-{b}"), qa)),
        (Value::String(a, qa), Value::Number(n, u)) => Ok(Value::String(
            format!("{a}-{}{}", n, u.as_deref().unwrap_or("")),
            qa,
        )),
        (Value::String(a, qa), Value::Color(c)) => Ok(Value::String(
            format!(
                "{a}-#{:02x}{:02x}{:02x}",
                c.legacy_rgb[0].round() as u8,
                c.legacy_rgb[1].round() as u8,
                c.legacy_rgb[2].round() as u8
            ),
            qa,
        )),
        (Value::Number(n, u), Value::String(b, qb)) => Ok(Value::String(
            format!("{}{}-{b}", n, u.as_deref().unwrap_or("")),
            qb,
        )),
        (Value::Color(c), Value::String(b, qb)) => Ok(Value::String(
            format!(
                "#{:02x}{:02x}{:02x}-{b}",
                c.legacy_rgb[0].round() as u8,
                c.legacy_rgb[1].round() as u8,
                c.legacy_rgb[2].round() as u8
            ),
            qb,
        )),
        // Number - Calc / Calc - Number — 作为 calc 表达式
        (Value::Number(n, u), Value::Calc(c)) => {
            let n_str = format!("{n}{}", u.as_deref().unwrap_or(""));
            let c_inner = c
                .strip_prefix("calc(")
                .and_then(|s| s.strip_suffix(")"))
                .unwrap_or(c.as_str());
            Ok(Value::Calc(format!("calc({n_str} - {c_inner})")))
        }
        (Value::Calc(c), Value::Number(n, u)) => {
            let n_str = format!("{n}{}", u.as_deref().unwrap_or(""));
            let c_inner = c
                .strip_prefix("calc(")
                .and_then(|s| s.strip_suffix(")"))
                .unwrap_or(c.as_str());
            Ok(Value::Calc(format!("calc({c_inner} - {n_str})")))
        }
        _ => Err(SassError::Eval("Unsupported - operation".into())),
    }
}

pub(crate) fn mul(l: &Value, r: &Value) -> Result<Value> {
    match (l, r) {
        (Value::Number(a, u1), Value::Number(b, u2)) => {
            let unit = match u1.is_some() {
                true => u1.clone(),
                false => u2.clone(),
            };
            Ok(Value::Number(a * b, unit))
        }
        // Number * Calc — 拼 calc 表达式
        (Value::Number(n, u), Value::Calc(c)) => {
            let n_str = format_number_with_unit(*n, u.as_deref());
            let c_inner = c
                .strip_prefix("calc(")
                .and_then(|s| s.strip_suffix(')'))
                .unwrap_or(c.as_str());
            Ok(Value::Calc(format!("calc({n_str} * {c_inner})")))
        }
        (Value::Calc(c), Value::Number(n, u)) => {
            let n_str = format_number_with_unit(*n, u.as_deref());
            let c_inner = c
                .strip_prefix("calc(")
                .and_then(|s| s.strip_suffix(')'))
                .unwrap_or(c.as_str());
            Ok(Value::Calc(format!("calc({c_inner} * {n_str})")))
        }
        _ => Err(SassError::Eval(format!("Cannot multiply {l} * {r}"))),
    }
}

pub(crate) fn div(l: &Value, r: &Value) -> Result<Value> {
    match (l, r) {
        (Value::Number(a, u1), Value::Number(b, u2)) => {
            match *b == 0.0 {
                true => {
                    // SCSS: 1/0 = Infinity, -1/0 = -Infinity, 0/0 = NaN
                    match *a == 0.0 {
                        true => return Ok(Value::Number(f64::NAN, u1.clone())),
                        false => {}
                    }
                    // 除零产生 infinity——构建 calc(infinity) 表达式
                    let neg = *a < 0.0;
                    let calc = format_infinity_with_units(
                        neg,
                        u1.as_ref().map(std::string::String::as_str).into_iter().collect::<Vec<_>>().as_slice(),
                        u2.as_ref().map(std::string::String::as_str).into_iter().collect::<Vec<_>>().as_slice(),
                    );
                    Ok(Value::Calc(calc))
                }
                false => Ok(Value::Number(a / b, u1.clone())),
            }
        }
        // Calc / Number — 拼接 calc 表达式（含除零处理）
        (Value::Calc(c), Value::Number(b, u2)) => {
            match *b == 0.0 {
                true => {
                    // 从 calc 表达式中提取单位信息
                    let (numerators, denominators) = extract_units_from_calc(c);
                    let num_refs: Vec<&str> = numerators.iter().map(|s| s.as_str()).collect();
                    let den_refs: Vec<&str> = denominators.iter().map(|s| s.as_str()).collect();
                    // 检测 calc 内部是否有负号（如 calc(-1px * 1em)）
                    let calc_inner = c
                        .strip_prefix("calc(")
                        .and_then(|s| s.strip_suffix(')'))
                        .unwrap_or(c.as_str());
                    let neg = calc_inner.trim_start().starts_with('-');
                    let calc = format_infinity_with_units(neg, &num_refs, &den_refs);
                    // 合并除数的单位到分母
                    let calc = match u2.as_ref() {
                        Some(u) if !u.is_empty() => {
                            let calc_inner = calc.strip_prefix("calc(").and_then(|s| s.strip_suffix(')')).unwrap_or(&calc);
                            format!("calc({calc_inner} / 1{u})")
                        }
                        _ => calc,
                    };
                    Ok(Value::Calc(calc))
                }
                false => {
                    let b_str = format_number_with_unit(*b, u2.as_deref());
                    let c_inner = c
                        .strip_prefix("calc(")
                        .and_then(|s| s.strip_suffix(')'))
                        .unwrap_or(c.as_str());
                    Ok(Value::Calc(format!("calc({c_inner} / {b_str})")))
                }
            }
        }
        // Number / Calc — 拼接 calc 表达式
        (Value::Number(n, u), Value::Calc(c)) => {
            let n_str = format_number_with_unit(*n, u.as_deref());
            let c_inner = c
                .strip_prefix("calc(")
                .and_then(|s| s.strip_suffix(')'))
                .unwrap_or(c.as_str());
            Ok(Value::Calc(format!("calc({n_str} / {c_inner})")))
        }
        // 非数字 / —— 作为斜杠分隔列表保留（如 font: 16px/24px）
        _ => Ok(Value::String(format!("{l}/{r}"), false)),
    }
}

pub(crate) fn modulo(l: &Value, r: &Value) -> Result<Value> {
    match (l, r) {
        (Value::Number(a, u), Value::Number(b, _)) => {
            match *b == 0.0 {
                true => return Err(SassError::DivideByZero),
                false => {}
            }
            // Sass 使用 floored modulo: a - b * floor(a / b)
            // 结果符号跟随除数 b，而非被除数 a
            let result = a - (*b * (a / b).floor());
            Ok(Value::Number(result, u.clone()))
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
        _ => Err(SassError::Eval(format!("Cannot compare {l} and {r}"))),
    }
}

/// 检查两个单位是否兼容（属于同一物理量类别）。
/// 单位兼容组——同组的单位互相兼容。
const UNIT_COMPAT_GROUPS: &[&[&str]] = &[
    &["px", "in", "cm", "mm", "pt", "pc", "q"], // 长度
    &["deg", "grad", "rad", "turn"],            // 角度
    &["s", "ms"],                               // 时间
    &["hz", "khz"],                             // 频率
    &["dpi", "dpcm", "dppx"],                   // 分辨率
];

pub(crate) fn units_compatible(u1: Option<&str>, u2: Option<&str>) -> bool {
    match u1 == u2 {
        true => return true,
        false => {}
    }
    match u1.is_none() || u2.is_none() {
        true => return true,
        false => {}
    }
    let g1 = u1.expect("non-none unit after none check");
    let g2 = u2.expect("non-none unit after none check");
    UNIT_COMPAT_GROUPS
        .iter()
        .any(|group| group.contains(&g1) && group.contains(&g2))
}

/// 格式化数字+单位为字符串（如 `1px`, `2.5`, `0`）。
fn format_number_with_unit(n: f64, unit: Option<&str>) -> String {
    let n_str = match n.fract() == 0.0 && n.abs() < 1e15 {
        true => format!("{n:.0}"),
        false => format!("{n}"),
    };
    match unit {
        Some(u) if !u.is_empty() => format!("{n_str}{u}"),
        _ => n_str,
    }
}

/// 格式化 infinity + 单位为 calc 表达式。
///
/// `calc(infinity * 1px * 1em)`, `calc(infinity / 1px)`, `calc(-infinity * 1px * 1em)`
fn format_infinity_with_units(
    neg: bool,
    numerators: &[&str],
    denominators: &[&str],
) -> String {
    let sign = if neg { "-" } else { "" };
    let numer_parts: Vec<String> = std::iter::once(format!("{sign}infinity"))
        .chain(
            numerators
                .iter()
                .filter(|u| !u.is_empty())
                .map(|u| format!("1{u}")),
        )
        .collect();
    let denom_parts: Vec<String> = denominators
        .iter()
        .filter(|u| !u.is_empty())
        .map(|u| format!("1{u}"))
        .collect();
    let numer_str = numer_parts.join(" * ");
    match denom_parts.is_empty() {
        true => format!("calc({numer_str})"),
        false => format!("calc({numer_str} / {})", denom_parts.join(" / ")),
    }
}

/// 从 calc 字符串中提取分子和分母单位。
///
/// 如 `calc(1px * 1em)` → (`["px", "em"]`, `[]`)
/// 如 `calc(1px / 1s)` → (`["px"]`, `["s"]`)
fn extract_units_from_calc(c: &str) -> (Vec<String>, Vec<String>) {
    let inner = c
        .strip_prefix("calc(")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(c);
    let (numerators, denominators) = inner
        .split_whitespace()
        .try_fold((Vec::new(), Vec::new()), |(mut nums, mut dens), part| {
            match part {
                "*" => Some((nums, dens)),
                "/" => {
                    // 切换到分母模式——用插入哨兵标记
                    nums.push(String::new());
                    Some((nums, dens))
                }
                _ => {
                    let unit = part
                        .chars()
                        .position(|c| c.is_alphabetic() || c == '%')
                        .map(|start| &part[start..])
                        .filter(|u| !u.is_empty());
                    match unit {
                        Some(u) => {
                            // 检查是否已切换到分母（nums 中有哨兵）
                            if nums.last().map(|s| s.is_empty()).unwrap_or(false) {
                                nums.pop();
                                dens.push(u.to_string());
                            } else {
                                nums.push(u.to_string());
                            }
                        }
                        None => {}
                    }
                    Some((nums, dens))
                }
            }
        })
        .unwrap_or_default();
    (numerators, denominators)
}

pub(crate) fn values_eq(l: &Value, r: &Value) -> bool {
    match (l, r) {
        (Value::Number(a, _), Value::Number(b, _)) => {
            match a.is_nan() && b.is_nan() {
                true => return true,
                false => {}
            }
            match a.is_infinite() && b.is_infinite() && a.signum() == b.signum() {
                true => return true,
                false => {}
            }
            (a - b).abs() < f64::EPSILON
        }
        (Value::String(a, _), Value::String(b, _)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Color(a), Value::Color(b)) => a == b,
        (Value::Null, Value::Null) => true,
        (Value::List(a, _, _), Value::List(b, _, _)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_eq(x, y))
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
