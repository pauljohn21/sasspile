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
                    super::Separator::SlashLiteral => "/",
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
            // AST 级别——求值前的 fallback
            Value::Call(name, args) => {
                let inner: Vec<String> = args.iter().map(|a| a.value.to_css_string()).collect();
                format!("{name}({})", inner.join(", "))
            }
            Value::Interp(s) => s.clone(),
            Value::BinOp(b) => {
                format!("{} {} {}", b.left.to_css_string(), binop_str(&b.op), b.right.to_css_string())
            }
            Value::UnaryOp(op, v) => {
                let op_str = match op {
                    super::UnaryOp::Neg => "-",
                    super::UnaryOp::Not => "not ",
                };
                format!("{op_str}{}", v.to_css_string())
            }
            Value::Calc(s) => s.clone(),
            Value::Paren(v) => format!("({})", v.to_css_string()),
        }
    }
}

fn binop_str(op: &super::BinOpKind) -> &'static str {
    match op {
        super::BinOpKind::Add => "+",
        super::BinOpKind::Sub => "-",
        super::BinOpKind::Mul => "*",
        super::BinOpKind::Div => "/",
        super::BinOpKind::Mod => "%",
        super::BinOpKind::Eq => "==",
        super::BinOpKind::NotEq => "!=",
        super::BinOpKind::Lt => "<",
        super::BinOpKind::Gt => ">",
        super::BinOpKind::LtEq => "<=",
        super::BinOpKind::GtEq => ">=",
        super::BinOpKind::And => "and",
        super::BinOpKind::Or => "or",
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
                format!("rgba({}, {}, {}, {})", c.r as u8, c.g as u8, c.b as u8, format_alpha(c.a))
            }
        }
        ColorFormat::Hsl => {
            let (h, s, l) = rgb_to_hsl(c.r, c.g, c.b);
            if c.a >= 1.0 {
                format!("hsl({}, {}%, {}%)", format_num(h), format_num(s), format_num(l))
            } else {
                format!("hsla({}, {}%, {}%, {})", format_num(h), format_num(s), format_num(l), format_alpha(c.a))
            }
        }
        ColorFormat::Hwb => {
            let (h, w, b) = rgb_to_hwb(c.r, c.g, c.b);
            if c.a >= 1.0 {
                format!("hwb({} {}% {}%)", format_num(h), format_num(w), format_num(b))
            } else {
                format!("hwb({} {}% {}% / {})", format_num(h), format_num(w), format_num(b), format_alpha(c.a))
            }
        }
    }
}

/// 格式化 alpha 值——去掉末尾零。
fn format_alpha(a: f64) -> String {
    if a.fract() == 0.0 {
        format!("{}", a as i64)
    } else {
        format!("{a}")
    }
}

/// RGB → HSL 转换。
fn rgb_to_hsl(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let r = r / 255.0;
    let g = g / 255.0;
    let b = b / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < 1e-10 {
        return (0.0, 0.0, l * 100.0);
    }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = match max {
        x if (x - r).abs() < 1e-10 => (g - b) / d + (if g < b { 6.0 } else { 0.0 }),
        x if (x - g).abs() < 1e-10 => (b - r) / d + 2.0,
        _ => (r - g) / d + 4.0,
    };
    (h * 60.0, s * 100.0, l * 100.0)
}

/// RGB → HWB 转换。
fn rgb_to_hwb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (h, _, _) = rgb_to_hsl(r, g, b);
    let r = r / 255.0;
    let g = g / 255.0;
    let b = b / 255.0;
    let w = r.min(g).min(b) * 100.0;
    let bk = (1.0 - r.max(g).max(b)) * 100.0;
    (h, w, bk)
}

/// 常见命名颜色查找。
fn named_color(r: f64, g: f64, b: f64) -> Option<&'static str> {
    let r = r as u8;
    let g = g as u8;
    let b = b as u8;
    match (r, g, b) {
        (0, 0, 0) => Some("black"),
        (255, 255, 255) => Some("white"),
        (255, 0, 0) => Some("red"),
        (0, 128, 0) => Some("green"),
        (0, 0, 255) => Some("blue"),
        (255, 255, 0) => Some("yellow"),
        (255, 165, 0) => Some("orange"),
        (128, 0, 128) => Some("purple"),
        (0, 255, 255) => Some("aqua"),
        (255, 0, 255) => Some("fuchsia"),
        (128, 128, 128) => Some("gray"),
        (192, 192, 192) => Some("silver"),
        (0, 0, 128) => Some("navy"),
        (0, 128, 128) => Some("teal"),
        (255, 192, 203) => Some("pink"),
        (255, 20, 147) => Some("deeppink"),
        _ => None,
    }
}
