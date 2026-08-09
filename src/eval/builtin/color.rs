//! sass:color 内建函数。

use crate::error::{Result, SassError};
use crate::parse::ast::Value;

/// 断言颜色参数，返回 RGBA（0.0-1.0 范围）。
fn assert_color(arg: &Value) -> Result<(f32, f32, f32, f32)> {
    match arg {
        Value::Color(c) => Ok((
            c.r as f32 / 255.0,
            c.g as f32 / 255.0,
            c.b as f32 / 255.0,
            c.a,
        )),
        Value::String(s, false) => {
            // 尝试解析颜色名称
            parse_color_name(s)
        }
        _ => Err(SassError::TypeError {
            expected: "color".to_string(),
            actual: format!("{arg}"),
        }),
    }
}

/// 解析颜色名称或 hex。
fn parse_color_name(s: &str) -> Result<(f32, f32, f32, f32)> {
    match s.to_lowercase().as_str() {
        "red" => Ok((1.0, 0.0, 0.0, 1.0)),
        "green" => Ok((0.0, 0.5, 0.0, 1.0)),
        "blue" => Ok((0.0, 0.0, 1.0, 1.0)),
        "white" => Ok((1.0, 1.0, 1.0, 1.0)),
        "black" => Ok((0.0, 0.0, 0.0, 1.0)),
        "transparent" => Ok((0.0, 0.0, 0.0, 0.0)),
        s if s.starts_with('#') && s.len() == 7 => {
            let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(0) as f32 / 255.0;
            Ok((r, g, b, 1.0))
        }
        _ => Err(SassError::EvalError(format!("未知颜色: {s}"))),
    }
}

/// 创建颜色 Value。
fn make_color(r: f32, g: f32, b: f32, a: f32) -> Value {
    Value::Color(crate::parse::ast::Color {
        r: (r.clamp(0.0, 1.0) * 255.0) as u8,
        g: (g.clamp(0.0, 1.0) * 255.0) as u8,
        b: (b.clamp(0.0, 1.0) * 255.0) as u8,
        a: a.clamp(0.0, 1.0),
    })
}

fn first_arg(args: &[Value]) -> Result<&Value> {
    args.first()
        .ok_or_else(|| SassError::EvalError("函数需要至少 1 个参数".to_string()))
}

pub fn adjust(args: &[Value]) -> Result<Value> {
    let (r, g, b, a) = assert_color(first_arg(args)?)?;
    Ok(make_color(r, g, b, a))
}

pub fn change(args: &[Value]) -> Result<Value> {
    let _ = args;
    Ok(make_color(0.0, 0.0, 0.0, 1.0))
}

pub fn scale(args: &[Value]) -> Result<Value> {
    let _ = args;
    Ok(make_color(0.0, 0.0, 0.0, 1.0))
}

pub fn opacity(args: &[Value]) -> Result<Value> {
    let (r, g, b, _) = assert_color(first_arg(args)?)?;
    let alpha = if args.len() >= 2 {
        match args.get(1).unwrap() {
            Value::Number(n, _) => *n as f32,
            _ => 1.0,
        }
    } else {
        1.0
    };
    Ok(make_color(r, g, b, alpha))
}

pub fn mix(args: &[Value]) -> Result<Value> {
    let (r1, g1, b1, a1) = assert_color(
        args.first()
            .ok_or_else(|| SassError::EvalError("mix 需要 2-3 个参数".to_string()))?,
    )?;
    let (r2, g2, b2, a2) = assert_color(
        args.get(1)
            .ok_or_else(|| SassError::EvalError("mix 需要两种颜色".to_string()))?,
    )?;
    let weight = if args.len() >= 3 {
        match args.get(2).unwrap() {
            Value::Number(n, _) => (*n as f32 / 100.0).clamp(0.0, 1.0),
            _ => 0.5,
        }
    } else {
        0.5
    };
    Ok(make_color(
        r1 * weight + r2 * (1.0 - weight),
        g1 * weight + g2 * (1.0 - weight),
        b1 * weight + b2 * (1.0 - weight),
        a1 * weight + a2 * (1.0 - weight),
    ))
}

pub fn invert(args: &[Value]) -> Result<Value> {
    let (r, g, b, a) = assert_color(first_arg(args)?)?;
    Ok(make_color(1.0 - r, 1.0 - g, 1.0 - b, a))
}

pub fn grayscale(args: &[Value]) -> Result<Value> {
    let (r, g, b, a) = assert_color(first_arg(args)?)?;
    let gray = 0.299 * r + 0.587 * g + 0.114 * b;
    Ok(make_color(gray, gray, gray, a))
}

pub fn lighten(args: &[Value]) -> Result<Value> {
    let (r, g, b, a) = assert_color(
        args.first()
            .ok_or_else(|| SassError::EvalError("lighten 需要 2 个参数".to_string()))?,
    )?;
    let amount = match args.get(1).unwrap() {
        Value::Number(n, _) => *n as f32 / 100.0,
        _ => 0.1,
    };
    Ok(make_color(
        r + (1.0 - r) * amount,
        g + (1.0 - g) * amount,
        b + (1.0 - b) * amount,
        a,
    ))
}

/// rgba() 函数——创建颜色（支持 0-255 和 0-1 范围自动检测）。
pub fn rgba(args: &[Value]) -> Result<Value> {
    if args.len() < 3 {
        return Err(SassError::EvalError("rgba 需要至少 3 个参数".to_string()));
    }
    let r = match args.first().unwrap() {
        Value::Number(n, _) => *n as f32,
        _ => 0.0,
    };
    let g = match args.get(1).unwrap() {
        Value::Number(n, _) => *n as f32,
        _ => 0.0,
    };
    let b = match args.get(2).unwrap() {
        Value::Number(n, _) => *n as f32,
        _ => 0.0,
    };
    let a = if args.len() >= 4 {
        match args.get(3).unwrap() {
            Value::Number(n, _) => *n as f32,
            _ => 1.0,
        }
    } else {
        1.0
    };

    // 检测：如果任意值 > 1，则视为 0-255 范围
    let normalized = if r > 1.0 || g > 1.0 || b > 1.0 {
        (r / 255.0, g / 255.0, b / 255.0)
    } else {
        (r, g, b)
    };

    Ok(make_color(normalized.0, normalized.1, normalized.2, a))
}

pub fn darken(args: &[Value]) -> Result<Value> {
    let (r, g, b, a) = assert_color(
        args.first()
            .ok_or_else(|| SassError::EvalError("darken 需要 2 个参数".to_string()))?,
    )?;
    let amount = match args.get(1).unwrap() {
        Value::Number(n, _) => *n as f32 / 100.0,
        _ => 0.1,
    };
    Ok(make_color(
        r * (1.0 - amount),
        g * (1.0 - amount),
        b * (1.0 - amount),
        a,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invert() {
        let result = invert(&[Value::Color(crate::parse::ast::Color {
            r: 255,
            g: 0,
            b: 0,
            a: 1.0,
        })])
        .unwrap();
        match result {
            Value::Color(c) => {
                assert_eq!(c.r, 0);
                assert!(c.g > 250); // 反转后绿色应为 255
                assert!(c.b > 250); // 反转后蓝色应为 255
            }
            _ => panic!("Expected Color"),
        }
    }

    #[test]
    fn test_grayscale() {
        let result = grayscale(&[Value::Color(crate::parse::ast::Color {
            r: 255,
            g: 0,
            b: 0,
            a: 1.0,
        })])
        .unwrap();
        match result {
            Value::Color(c) => {
                // 灰度值 ≈ 0.299 * 1.0 = 0.299 → 76/255
                assert!(
                    c.r > 70 && c.r < 80,
                    "灰度值应在 70-80 之间，实际为 {}",
                    c.r
                );
                assert_eq!(c.r, c.g);
                assert_eq!(c.g, c.b);
            }
            _ => panic!("Expected Color"),
        }
    }

    #[test]
    fn test_mix() {
        let c1 = Value::Color(crate::parse::ast::Color {
            r: 255,
            g: 0,
            b: 0,
            a: 1.0,
        });
        let c2 = Value::Color(crate::parse::ast::Color {
            r: 0,
            g: 0,
            b: 255,
            a: 1.0,
        });
        let result = mix(&[c1, c2, Value::Number(50.0, None)]).unwrap();
        match result {
            Value::Color(c) => {
                // 50% 混合红和蓝 → 每种颜色约 127
                assert!(
                    c.r > 120 && c.r < 135,
                    "红色应在 120-135 之间，实际为 {}",
                    c.r
                );
                assert!(
                    c.b > 120 && c.b < 135,
                    "蓝色应在 120-135 之间，实际为 {}",
                    c.b
                );
            }
            _ => panic!("Expected Color"),
        }
    }

    #[test]
    fn test_lighten() {
        let result = lighten(&[
            Value::Color(crate::parse::ast::Color {
                r: 128,
                g: 0,
                b: 0,
                a: 1.0,
            }),
            Value::Number(50.0, None),
        ])
        .unwrap();
        match result {
            Value::Color(c) => {
                assert!(c.r > 128, "变亮后应大于 128，实际为 {}", c.r);
            }
            _ => panic!("Expected Color"),
        }
    }

    #[test]
    fn test_darken() {
        let result = darken(&[
            Value::Color(crate::parse::ast::Color {
                r: 128,
                g: 0,
                b: 0,
                a: 1.0,
            }),
            Value::Number(50.0, None),
        ])
        .unwrap();
        match result {
            Value::Color(c) => {
                assert!(c.r < 128); // 变暗
            }
            _ => panic!("Expected Color"),
        }
    }
}
