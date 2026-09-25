//! Math 三角函数 + 高级运算（响应式风格移植自 main math_trig.rs）
//!
//! 纯函数签名：fn(&[String]) -> Option<String>
//! - 单位解析: split_unit("45deg") → (45.0, "deg")
//! - angle→radians: deg/rad/grad/turn/无单位(弧度)
//! - 反三角函数返回值单位: deg

use super::split_unit;

// ═══════════════════════════════════════════════════════════════════════════
// 三角函数: sin / cos / tan
// ═══════════════════════════════════════════════════════════════════════════

pub fn eval_sin(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let (num, unit) = split_unit(s)?;
    let rad = angle_to_rad(num, unit);
    Some(fmt_f64(rad.sin()))
}

pub fn eval_cos(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let (num, unit) = split_unit(s)?;
    let rad = angle_to_rad(num, unit);
    Some(fmt_f64(rad.cos()))
}

pub fn eval_tan(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let (num, unit) = split_unit(s)?;
    let rad = angle_to_rad(num, unit);
    Some(fmt_f64(rad.tan()))
}

// ═══════════════════════════════════════════════════════════════════════════
// 反三角函数: asin / acos / atan — 返回 deg
// ═══════════════════════════════════════════════════════════════════════════

pub fn eval_asin(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let (num, unit) = split_unit(s)?;
    if !unit.is_empty() {
        return None; // 反三角要求 unitless
    }
    let result = num.asin().to_degrees();
    Some(format!("{}deg", fmt_f64(result)))
}

pub fn eval_acos(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let (num, unit) = split_unit(s)?;
    if !unit.is_empty() {
        return None;
    }
    let result = num.acos().to_degrees();
    Some(format!("{}deg", fmt_f64(result)))
}

pub fn eval_atan(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let (num, unit) = split_unit(s)?;
    if !unit.is_empty() {
        return None;
    }
    let result = num.atan().to_degrees();
    Some(format!("{}deg", fmt_f64(result)))
}

// ═══════════════════════════════════════════════════════════════════════════
// 指数 / 对数 / 幂
// ═══════════════════════════════════════════════════════════════════════════

pub fn eval_sqrt(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let (num, unit) = split_unit(s)?;
    if !unit.is_empty() {
        return None; // sqrt 必须 unitless
    }
    if num < 0.0 {
        return Some("calc(NaN)".to_string());
    }
    Some(fmt_f64(num.sqrt()))
}

pub fn eval_pow(args: &[String]) -> Option<String> {
    let a = args.first()?.trim();
    let b = args.get(1)?.trim();
    let (base, u1) = split_unit(a)?;
    let (exp, u2) = split_unit(b)?;
    if !u1.is_empty() || !u2.is_empty() {
        return None; // pow 参数须 unitless
    }
    let result = base.powf(exp);
    if result.is_nan() {
        Some("calc(NaN)".to_string())
    } else if result.is_infinite() {
        let sign = if result.is_sign_negative() { "-" } else { "" };
        Some(format!("calc({sign}infinity)"))
    } else {
        Some(fmt_f64(result))
    }
}

pub fn eval_log(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let (num, unit) = split_unit(s)?;
    if !unit.is_empty() {
        return None;
    }
    if num < 0.0 {
        return Some("calc(NaN)".to_string());
    }
    if num == 0.0 {
        return Some("calc(-infinity)".to_string());
    }
    if args.len() == 2 {
        let base_s = args.get(1)?.trim();
        let (base, u2) = split_unit(base_s)?;
        if !u2.is_empty() {
            return None;
        }
        return Some(fmt_f64(num.log(base)));
    }
    Some(fmt_f64(num.ln()))
}

pub fn eval_exp(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let (num, unit) = split_unit(s)?;
    if !unit.is_empty() {
        return None;
    }
    let result = num.exp();
    if result.is_infinite() || result.is_nan() {
        let s = if result.is_nan() {
            "NaN".to_string()
        } else if result.is_sign_negative() {
            "-infinity".to_string()
        } else {
            "infinity".to_string()
        };
        Some(format!("calc({s})"))
    } else {
        Some(fmt_f64(result))
    }
}

pub fn eval_sign(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let (num, unit) = split_unit(s)?;
    let result = sign_value(num);
    if unit.is_empty() {
        Some(fmt_f64(result))
    } else {
        Some(format!("{}{unit}", fmt_f64(result)))
    }
}

pub fn eval_clamp(args: &[String]) -> Option<String> {
    // clamp(min, val, max) or clamp(val) where val already bounded
    if args.len() == 1 {
        // 单参数: clamp(number) → 返回原值 (用于简化管线)
        return Some(args.first()?.trim().to_string());
    }
    if args.len() != 3 {
        return None;
    }
    let min_s = args.first()?.trim();
    let val_s = args.get(1)?.trim();
    let max_s = args.get(2)?.trim();

    let (min_n, min_u) = split_unit(min_s)?;
    let (val_n, val_u) = split_unit(val_s)?;
    let (max_n, max_u) = split_unit(max_s)?;

    // 单位必须兼容：提取第一个非空单位
    let unit = if !val_u.is_empty() {
        val_u
    } else if !min_u.is_empty() {
        min_u
    } else {
        max_u
    };

    let result = val_n.clamp(min_n, max_n);
    Some(format!("{result}{unit}"))
}

pub fn eval_mod(args: &[String]) -> Option<String> {
    let a = args.first()?.trim();
    let b = args.get(1)?.trim();
    let (a_n, a_u) = split_unit(a)?;
    let (b_n, b_u) = split_unit(b)?;
    // mod 单位从第一个参数
    let unit = if a_u.is_empty() { b_u } else { a_u };
    Some(format!("{}{unit}", fmt_f64(a_n % b_n)))
}

// ═══════════════════════════════════════════════════════════════════════════
// 辅助函数
// ═══════════════════════════════════════════════════════════════════════════

/// 角度单位→弧度 (deg/rad/grad/turn/无单位 → 弧度)
fn angle_to_rad(num: f64, unit: &str) -> f64 {
    match unit {
        "" | "rad" => num,
        "deg" => num * std::f64::consts::PI / 180.0,
        "grad" => num * std::f64::consts::PI / 200.0,
        "turn" => num * 2.0 * std::f64::consts::PI,
        _ => num, // 未知单位按弧度处理（sass-spec 不覆盖未知退化）
    }
}

/// sign 函数：CSS 规范 sign(0) = 0（不是 1）
fn sign_value(n: f64) -> f64 {
    if n == 0.0 {
        0.0
    } else {
        n.signum()
    }
}

/// f64 格式化: 去掉尾随零，整数不显示小数点
fn fmt_f64(v: f64) -> String {
    if v.is_nan() {
        "NaN".to_string()
    } else if v.is_infinite() {
        if v.is_sign_negative() {
            "-infinity".to_string()
        } else {
            "infinity".to_string()
        }
    } else if v.fract() == 0.0 && v.abs() < 1e15 {
        // 整数: 格式化为 i64
        format!("{}", v as i64)
    } else {
        // 小数: 精简格式
        let s = format!("{v}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}
