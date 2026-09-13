//! 纯 calc 表达式检测与解析辅助函数。
//!
//! `is_pure_calc_expr` 检查 calc() 内容是否仅包含纯数字、常量、基础算术运算。
//! `parse_simple_number` 尝试将字符串解析为纯数字（含单位）。

use super::*;

/// 检查内容是否仅包含纯数字、单位字面量、常量（pi/e）和基础算术运算（+-*/）。
/// 任何变量引用（$）、插值痕迹（#{}）、非数字函数调用都会使此返回 false。
/// 用于决定 eval_value 中是否可对 `calc(...)` 内容做数值简化——
/// 仅纯表达式可简化，含插值/变量时须保留 Calc 类型供 meta.calc-args 等内省。
pub(crate) fn is_pure_calc_expr(s: &str) -> bool {
    let s = s.trim();
    match s.is_empty() {
        true => return false,
        false => {}
    }
    // 纯数字+单位 — 允许简化。calc(1px) → Number(1px) 是合法的数值简化。
    // 注意：这使得 meta.calc-args(calc(1px)) 接收 Number 而非 Calc，
    // 需要 calc-args/calc-name 内做一个 从 Number → 单元素列表 的转换。
    if parse_simple_number(s).is_some() {
        return true;
    }
    // 嵌套 calc/min/max/clamp 调用 —— 递归检查参数
    for prefix in &["calc(", "min(", "max(", "clamp("] {
        if let Some(rest) = s.strip_prefix(prefix) {
            let Some(inner) = rest.strip_suffix(")") else {
                return false;
            };
            let mut depth = 0i32;
            let mut arg_start = 0;
            for (i, c) in inner.char_indices() {
                match c {
                    '(' | '[' => depth += 1,
                    ')' | ']' => depth -= 1,
                    ',' if depth == 0 => {
                        if !is_pure_calc_expr(&inner[arg_start..i]) {
                            return false;
                        }
                        arg_start = i + 1;
                    }
                    _ => {}
                }
            }
            return is_pure_calc_expr(&inner[arg_start..]);
        }
    }
    // 括号包裹的纯表达式
    if let Some(inner) = s.strip_prefix('(').and_then(|r| r.strip_suffix(')')) {
        return is_pure_calc_expr(inner);
    }
    // 算术运算（+-*/）：分割运算符，递归验证两边
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ' ' if depth == 0 => {
                let rest = &s[i..];
                if rest.starts_with(" + ")
                    || rest.starts_with(" - ")
                    || rest.starts_with(" * ")
                    || rest.starts_with(" / ")
                {
                    return is_pure_calc_expr(&s[..i]) && is_pure_calc_expr(&s[i + 3..]);
                }
            }
            _ => {}
        }
    }
    // 存在变量/插值/其他非数字字符 → 不纯
    !s.contains('$') && !s.contains('#')
}

/// 尝试将字符串解析为纯数字（含单位）。
pub(crate) fn parse_simple_number(s: &str) -> Option<Value> {
    let s = s.trim();
    match s {
        "pi" => return Some(Value::Number(std::f64::consts::PI, None)),
        "e" => return Some(Value::Number(std::f64::consts::E, None)),
        _ => {}
    }
    let s = s.strip_prefix('+').unwrap_or(s);
    let split = s.find(|c: char| {
        !c.is_ascii_digit() && c != '.' && c != '-' && c != 'e' && c != 'E' && c != '+'
    });
    match split {
        None => s.parse::<f64>().ok().map(|n| Value::Number(n, None)),
        Some(idx) if idx > 0 => {
            let (num_str, unit) = s.split_at(idx);
            let n = num_str.parse::<f64>().ok()?;
            let unit = unit.trim();
            match unit.is_empty() {
                true => return Some(Value::Number(n, None)),
                false => {}
            }
            match !unit.chars().all(|c| c.is_ascii_alphabetic()) {
                true => return None,
                false => {}
            }
            Some(Value::Number(n, Some(unit.to_string())))
        }
        _ => None,
    }
}
