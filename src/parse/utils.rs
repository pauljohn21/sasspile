//! 解析工具函数。

use crate::error::{Result, SassError};
use crate::parse::ast::{Color, Value};

/// 解析数字字符串为 Value::Number。
pub(super) fn parse_number(s: &str) -> Result<Value> {
    // 分离数值和单位
    let (num_part, unit) = if let Some(idx) = s.find(|c: char| c.is_ascii_alphabetic() || c == '%')
    {
        (&s[..idx], Some(s[idx..].to_string()))
    } else {
        (s, None)
    };
    match num_part.parse::<f64>() {
        Ok(n) => Ok(Value::Number(n, unit)),
        Err(_) => Err(SassError::ParseError {
            expected: "数字".to_string(),
            found: s.to_string(),
        }),
    }
}

/// 解析 #hash 字符串为 Color。
pub(super) fn parse_hash_color(s: &str) -> Color {
    let bytes = s.as_bytes();
    match bytes.len() {
        3 => Color {
            r: hex2(bytes[0], bytes[0]),
            g: hex2(bytes[1], bytes[1]),
            b: hex2(bytes[2], bytes[2]),
            a: 1.0,
        },
        6 => Color {
            r: hex2(bytes[0], bytes[1]),
            g: hex2(bytes[2], bytes[3]),
            b: hex2(bytes[4], bytes[5]),
            a: 1.0,
        },
        _ => Color::default(),
    }
}

/// 两个 hex 字符转 u8。
fn hex2(high: u8, low: u8) -> u8 {
    (hex1(high) << 4) | hex1(low)
}

/// 单个 hex 字符转 u8。
fn hex1(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
}
