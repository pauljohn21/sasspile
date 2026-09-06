#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
use super::*;
use crate::consts::FLOAT_PRECISION_INV;

/// 格式化浮点数——截断到 10 位小数（与 SCSS 规范一致）。
/// NaN 输出为 `none`（CSS Color 4 missing 通道）。
fn format_num(n: f64) -> String {
    match n.is_nan() {
        true => return "none".to_string(),
        false => {}
    }
    let n = (n * FLOAT_PRECISION_INV).round() / FLOAT_PRECISION_INV;
    match n.fract() == 0.0 {
        true => format!("{}", n as i64),
        false => format!("{n}"),
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n, None) => match (*n < 0.0, n.is_infinite(), n.is_nan()) {
                (_, true, _) => {
                    let sign = match *n < 0.0 { true => "-", false => "" };
                    write!(f, "calc({sign}infinity)")
                }
                (_, _, true) => write!(f, "calc(NaN)"),
                _ => write!(f, "{}", format_num(*n)),
            },
            Value::Number(n, Some(unit)) => match (*n < 0.0, n.is_infinite(), n.is_nan()) {
                (_, true, _) => {
                    let sign = match *n < 0.0 { true => "-", false => "" };
                    write!(f, "calc({sign}infinity * 1{unit})")
                }
                (_, _, true) => write!(f, "calc(NaN * 1{unit})"),
                _ => write!(f, "{}{unit}", format_num(*n)),
            },
            Value::String(s, true) => {
                let (quote, escaped) = Self::escape_quoted_string(s);
                write!(f, "{quote}{escaped}{quote}")
            }
            Value::String(s, false) => {
                // 未加引号的字符串——只转义控制字符和引号
                write!(f, "{}", Self::escape_css_chars(s, |_| false))
            }
            Value::Color(c) => display_color::fmt_color(c, c.output, f),
            Value::List(elements, sep, bracketed) => {
                match elements.is_empty() {
                    true => {
                        return match *bracketed {
                            true => write!(f, "[]"),
                            false => Ok(()),
                        };
                    }
                    false => {}
                }
                let sep_str = match sep {
                    Separator::Comma => ", ",
                    Separator::Space => " ",
                    Separator::Slash => " / ",
                    Separator::SlashLiteral => "/",
                    Separator::Undecided => " ",
                };
                match *bracketed {
                    true => f.write_str("[")?,
                    false => {}
                }
                for (i, e) in elements.iter().enumerate() {
                    match i > 0 {
                        true => f.write_str(sep_str)?,
                        false => {}
                    }
                    e.fmt(f)?;
                }
                match *bracketed {
                    true => f.write_str("]")?,
                    false => {}
                }
                Ok(())
            }
            Value::Map(pairs) => {
                f.write_str("(")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    match i > 0 {
                        true => f.write_str(", ")?,
                        false => {}
                    }
                    k.fmt(f)?;
                    f.write_str(": ")?;
                    v.fmt(f)?;
                }
                f.write_str(")")
            }
            Value::Variable(name) => write!(f, "${name}"),
            Value::Bool(true) => f.write_str("true"),
            Value::Bool(false) => f.write_str("false"),
            Value::Null => f.write_str("null"),
            Value::Call(name, args) => {
                // if() 冒号语法：condition: value; else: other
                match name == "if"
                    && args
                        .iter()
                        .any(|a| a.condition.is_some() || a.name.as_deref() == Some("else"))
                {
                    true => {
                        write!(f, "{name}(")?;
                        for (i, a) in args.iter().enumerate() {
                            match i > 0 {
                                true => f.write_str("; ")?,
                                false => {}
                            }
                            match (&a.condition, &a.name) {
                                (Some(cond), _) => {
                                    cond.fmt(f)?;
                                    f.write_str(": ")?;
                                    a.value.fmt(f)?;
                                }
                                (None, Some(n)) => {
                                    write!(f, "{n}: ")?;
                                    a.value.fmt(f)?;
                                }
                                (None, None) => {
                                    a.value.fmt(f)?;
                                }
                            }
                        }
                        f.write_str(")")
                    }
                    false => {
                        write!(f, "{name}(")?;
                        for (i, a) in args.iter().enumerate() {
                            match i > 0 {
                                true => f.write_str(", ")?,
                                false => {}
                            }
                            a.value.fmt(f)?;
                        }
                        f.write_str(")")
                    }
                }
            }
            Value::Interp(segments) => {
                for seg in segments {
                    match seg {
                        InterpSegment::Expr(e) => write!(f, "#{{{e}}}")?,
                        InterpSegment::Text(t) => f.write_str(t)?,
                    }
                }
                Ok(())
            }
            Value::BinOp(b) => {
                let op_str = match b.op {
                    BinOpKind::Add => " + ",
                    BinOpKind::Sub => " - ",
                    BinOpKind::Mul => " * ",
                    BinOpKind::Div => " / ",
                    BinOpKind::Mod => " % ",
                    BinOpKind::Eq => " == ",
                    BinOpKind::NotEq => " != ",
                    BinOpKind::Lt => " < ",
                    BinOpKind::Gt => " > ",
                    BinOpKind::LtEq => " <= ",
                    BinOpKind::GtEq => " >= ",
                    BinOpKind::And => " and ",
                    BinOpKind::Or => " or ",
                };
                write!(f, "{}{}{}", b.left, op_str, b.right)
            }
            Value::UnaryOp(op, v) => match op {
                UnaryOp::Pos => write!(f, "+{v}"),
                UnaryOp::Neg => write!(f, "-{v}"),
                UnaryOp::Not => write!(f, "not {v}"),
            },
            Value::Calc(s) => write!(f, "{s}"),
            Value::Paren(v) => write!(f, "({v})"),
            Value::Spread(v) => write!(f, "{v}..."),
            Value::MixinRef(data) => {
                write!(f, "get-mixin(\"{}\")", data.name)
            }
        }
    }
}
