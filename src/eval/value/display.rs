//! Value Display 实现 + ColorFormat。

use std::fmt;

use crate::lex::token::QuoteStyle;
use crate::eval::value::{Value, Color, Separator};

/// 颜色格式——影响序列化输出。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    Auto,
    Rgb,
    Hsl,
    Hwb,
}

impl super::Value {
    /// 序列化为 CSS 字符串。
    pub fn to_css_string(&self) -> String {
        match self {
            Value::Number(n, Some(unit)) => {
                format_num(*n) + unit
            }
            Value::Number(n, None) => format_num(*n),
            Value::String(s, QuoteStyle::Double) => format!("\"{s}\""),
            Value::String(s, QuoteStyle::Single) => format!("'{s}'"),
            Value::String(s, QuoteStyle::None) => s.clone(),
            Value::Ident(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            Value::Color(c) => color_to_css(c),
            Value::List(items, sep, brackets) => {
                let sep_str = match sep {
                    super::Separator::Comma => ", ",
                    super::Separator::Space | super::Separator::Undecided => " ",
                    super::Separator::Slash => " / ",
                };
                let inner: Vec<String> = items.iter().map(|v| v.to_css_string()).collect();
                let joined = inner.join(sep_str);
                if *brackets {
                    format!("[{joined}]")
                } else {
                    joined
                }
            }
            Value::Map(pairs) => {
                let inner: Vec<String> = pairs.iter()
                    .map(|(k, v)| format!("{}: {}", k.to_css_string(), v.to_css_string()))
                    .collect();
                format!("({})", inner.join(", "))
            }
            Value::Function(f) => format!("get-function(\"{}\")", f.name),
            Value::Variable(name) => format!("${name}"),
            Value::ArgList(items) => {
                let inner: Vec<String> = items.iter().map(|v| v.to_css_string()).collect();
                inner.join(", ")
            }
        }
    }
}

fn format_num(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

fn color_to_css(c: &super::Color) -> String {
    match c.format {
        ColorFormat::Auto | ColorFormat::Rgb => {
            if c.a >= 1.0 {
                format!("#{:02x}{:02x}{:02x}", c.r as u8, c.g as u8, c.b as u8)
            } else {
                format!("rgba({}, {}, {}, {})", c.r as u8, c.g as u8, c.b as u8, c.a)
            }
        }
        ColorFormat::Hsl => format!("hsl({}, {}%, {}%)", 0, 0, 0), // placeholder
        ColorFormat::Hwb => format!("hwb(0 0% 0%)"), // placeholder
    }
}
