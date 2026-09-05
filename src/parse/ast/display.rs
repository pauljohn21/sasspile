#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
use super::*;
use crate::consts::{
    ALPHA_TOLERANCE, DEG_UNIT, FLOAT_NOISE_THRESHOLD, FLOAT_PRECISION_INV, PCT_ROUND_THRESHOLD,
    PCT_SCALE,
};

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

/// 清理颜色分量的浮点噪声——将极小值归零。
/// 当 |v| < `FLOAT_NOISE_THRESHOLD` 时视为 0，避免矩阵系数精度不足导致的残留。
fn clean_num(v: f64) -> f64 {
    match v.abs() < FLOAT_NOISE_THRESHOLD {
        true => 0.0,
        false => v,
    }
}

/// 清理百分比分量——接近 0 或 100 时归整。
fn clean_pct(v: f64) -> f64 {
    match v {
        v if v.abs() < FLOAT_NOISE_THRESHOLD => 0.0,
        v if (v - PCT_SCALE).abs() < PCT_ROUND_THRESHOLD => PCT_SCALE,
        _ => v,
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
            Value::Color(c) => {
                match c.output {
                    ColorOutput::RgbExplicit => {
                        match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                            true => write!(
                                f,
                                "rgb({}, {}, {})",
                                c.legacy_rgb[0].round() as u8,
                                c.legacy_rgb[1].round() as u8,
                                c.legacy_rgb[2].round() as u8
                            ),
                            false => write!(
                                f,
                                "rgba({}, {}, {}, {})",
                                c.legacy_rgb[0].round() as u8,
                                c.legacy_rgb[1].round() as u8,
                                c.legacy_rgb[2].round() as u8,
                                format_alpha(c.a)
                            ),
                        }
                    }
                    ColorOutput::RgbPercent => {
                        // channels 存储 HSL 值 (h, s, l)
                        let (h, s, l) = (c.channels[0], c.channels[1], c.channels[2]);
                        let (rp, gp, bp) = hsl_to_rgb_percent(h, s, l);
                        // 检查是否匹配命名颜色，优先输出名称
                        let alpha_ok = (c.a - 1.0).abs() < ALPHA_TOLERANCE;
                        match (alpha_ok, crate::eval::Evaluator::reverse_lookup_named_color(c)) {
                            (true, Some(name)) => write!(f, "{name}"),
                            (true, None) => write!(
                                f,
                                "rgb({}%, {}%, {}%)",
                                format_pct_val(rp),
                                format_pct_val(gp),
                                format_pct_val(bp)
                            ),
                            (false, _) => write!(
                                f,
                                "rgba({}%, {}%, {}%, {})",
                                format_pct_val(rp),
                                format_pct_val(gp),
                                format_pct_val(bp),
                                format_alpha(c.a)
                            ),
                        }
                    }
                    ColorOutput::Auto => match c.space {
                        ColorSpace::Hsl => {
                            let (h, s, l) = (c.channels[0], c.channels[1], c.channels[2]);
                            let hue_str = format_hue(h);
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => write!(
                                    f,
                                    "hsl({}, {}%, {}%)",
                                    hue_str,
                                    format_pct(s),
                                    format_pct(l)
                                ),
                                false => write!(
                                    f,
                                    "hsla({}, {}%, {}%, {})",
                                    hue_str,
                                    format_pct(s),
                                    format_pct(l),
                                    format_alpha(c.a)
                                ),
                            }
                        }
                        ColorSpace::Hwb => {
                            let (h, w, bk) = (c.channels[0], c.channels[1], c.channels[2]);
                            // 全通道 NaN 时保留 hwb(none none none) 格式
                            match h.is_nan() && w.is_nan() && bk.is_nan() {
                                true => {
                                    match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                        true => write!(f, "hwb(none none none)"),
                                        false => write!(f, "hwb(none none none / {})", format_alpha(c.a)),
                                    }
                                }
                                false => {
                                    // SCSS 规范：HWB 是 legacy 格式，Auto 输出规范化为 HSL
                                    let h = if h.is_nan() { 0.0 } else { h };
                                    let w = if w.is_nan() { 0.0 } else { w };
                                    let bk = if bk.is_nan() { 0.0 } else { bk };
                                    // HWB → HSL 转换（内联实现，避免跨模块引用）
                                    let (hsl_h, hsl_s, hsl_l) = hwb_to_hsl_inline(h, w, bk);
                                    let hue_str = format_hue(hsl_h);
                                    match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                        true => write!(
                                            f,
                                            "hsl({}, {}%, {}%)",
                                            hue_str,
                                            format_pct(hsl_s),
                                            format_pct(hsl_l)
                                        ),
                                        false => write!(
                                            f,
                                            "hsla({}, {}%, {}%, {})",
                                            hue_str,
                                            format_pct(hsl_s),
                                            format_pct(hsl_l),
                                            format_alpha(c.a)
                                        ),
                                    }
                                }
                            }
                        }
                        ColorSpace::Lab => {
                            let (l, a, b) = (c.channels[0], c.channels[1], c.channels[2]);
                            let l_clean = clean_pct(l);
                            let a_clean = clean_num(a);
                            let b_clean = clean_num(b);
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => write!(
                                    f,
                                    "lab({}% {} {})",
                                    format_num(l_clean),
                                    format_num(a_clean),
                                    format_num(b_clean)
                                ),
                                false => write!(
                                    f,
                                    "lab({}% {} {} / {})",
                                    format_num(l_clean),
                                    format_num(a_clean),
                                    format_num(b_clean),
                                    format_alpha(c.a)
                                ),
                            }
                        }
                        ColorSpace::Lch => {
                            let (l, ch, h) = (c.channels[0], c.channels[1], c.channels[2]);
                            let l_clean = clean_pct(l);
                            let ch_clean = clean_num(ch);
                            let h_str = match ch_clean == 0.0 {
                                true => "none".to_string(),
                                false => format!("{}{}", format_hue(h), DEG_UNIT),
                            };
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => write!(
                                    f,
                                    "lch({}% {} {})",
                                    format_num(l_clean),
                                    format_num(ch_clean),
                                    h_str
                                ),
                                false => write!(
                                    f,
                                    "lch({}% {} {} / {})",
                                    format_num(l_clean),
                                    format_num(ch_clean),
                                    h_str,
                                    format_alpha(c.a)
                                ),
                            }
                        }
                        ColorSpace::Oklab => {
                            let (l, a, b) = (c.channels[0], c.channels[1], c.channels[2]);
                            let l_pct = clean_pct(l * PCT_SCALE);
                            let a_clean = clean_num(a);
                            let b_clean = clean_num(b);
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => write!(
                                    f,
                                    "oklab({}% {} {})",
                                    format_num(l_pct),
                                    format_num(a_clean),
                                    format_num(b_clean)
                                ),
                                false => write!(
                                    f,
                                    "oklab({}% {} {} / {})",
                                    format_num(l_pct),
                                    format_num(a_clean),
                                    format_num(b_clean),
                                    format_alpha(c.a)
                                ),
                            }
                        }
                        ColorSpace::Oklch => {
                            let (l, ch, h) = (c.channels[0], c.channels[1], c.channels[2]);
                            let l_pct = clean_pct(l * PCT_SCALE);
                            let ch_clean = clean_num(ch);
                            let h_str = match ch_clean == 0.0 {
                                true => "none".to_string(),
                                false => format!("{}{}", format_hue(h), DEG_UNIT),
                            };
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => write!(
                                    f,
                                    "oklch({}% {} {})",
                                    format_num(l_pct),
                                    format_num(ch_clean),
                                    h_str
                                ),
                                false => write!(
                                    f,
                                    "oklch({}% {} {} / {})",
                                    format_num(l_pct),
                                    format_num(ch_clean),
                                    h_str,
                                    format_alpha(c.a)
                                ),
                            }
                        }
                        ColorSpace::DisplayP3 => {
                            let (r, g, b) = (c.channels[0], c.channels[1], c.channels[2]);
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => write!(
                                    f,
                                    "color(display-p3 {} {} {})",
                                    format_num(r),
                                    format_num(g),
                                    format_num(b)
                                ),
                                false => write!(
                                    f,
                                    "color(display-p3 {} {} {} / {})",
                                    format_num(r),
                                    format_num(g),
                                    format_num(b),
                                    format_alpha(c.a)
                                ),
                            }
                        }
                        ColorSpace::DisplayP3Linear => {
                            let (r, g, b) = (c.channels[0], c.channels[1], c.channels[2]);
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => write!(
                                    f,
                                    "color(display-p3-linear {} {} {})",
                                    format_num(r),
                                    format_num(g),
                                    format_num(b)
                                ),
                                false => write!(
                                    f,
                                    "color(display-p3-linear {} {} {} / {})",
                                    format_num(r),
                                    format_num(g),
                                    format_num(b),
                                    format_alpha(c.a)
                                ),
                            }
                        }
                        ColorSpace::Srgb => {
                            let (r, g, b) = (c.channels[0], c.channels[1], c.channels[2]);
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => write!(
                                    f,
                                    "color(srgb {} {} {})",
                                    format_num(r),
                                    format_num(g),
                                    format_num(b)
                                ),
                                false => write!(
                                    f,
                                    "color(srgb {} {} {} / {})",
                                    format_num(r),
                                    format_num(g),
                                    format_num(b),
                                    format_alpha(c.a)
                                ),
                            }
                        }
                        ColorSpace::SrgbLinear => {
                            let (r, g, b) = (c.channels[0], c.channels[1], c.channels[2]);
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => write!(
                                    f,
                                    "color(srgb-linear {} {} {})",
                                    format_num(r),
                                    format_num(g),
                                    format_num(b)
                                ),
                                false => write!(
                                    f,
                                    "color(srgb-linear {} {} {} / {})",
                                    format_num(r),
                                    format_num(g),
                                    format_num(b),
                                    format_alpha(c.a)
                                ),
                            }
                        }
                        ColorSpace::A98Rgb => {
                            let (r, g, b) = (c.channels[0], c.channels[1], c.channels[2]);
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => write!(
                                    f,
                                    "color(a98-rgb {} {} {})",
                                    format_num(r),
                                    format_num(g),
                                    format_num(b)
                                ),
                                false => write!(
                                    f,
                                    "color(a98-rgb {} {} {} / {})",
                                    format_num(r),
                                    format_num(g),
                                    format_num(b),
                                    format_alpha(c.a)
                                ),
                            }
                        }
                        ColorSpace::ProphotoRgb => {
                            let (r, g, b) = (c.channels[0], c.channels[1], c.channels[2]);
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => write!(
                                    f,
                                    "color(prophoto-rgb {} {} {})",
                                    format_num(r),
                                    format_num(g),
                                    format_num(b)
                                ),
                                false => write!(
                                    f,
                                    "color(prophoto-rgb {} {} {} / {})",
                                    format_num(r),
                                    format_num(g),
                                    format_num(b),
                                    format_alpha(c.a)
                                ),
                            }
                        }
                        ColorSpace::Rec2020 => {
                            let (r, g, b) = (c.channels[0], c.channels[1], c.channels[2]);
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => write!(
                                    f,
                                    "color(rec2020 {} {} {})",
                                    format_num(r),
                                    format_num(g),
                                    format_num(b)
                                ),
                                false => write!(
                                    f,
                                    "color(rec2020 {} {} {} / {})",
                                    format_num(r),
                                    format_num(g),
                                    format_num(b),
                                    format_alpha(c.a)
                                ),
                            }
                        }
                        ColorSpace::XyzD65 => {
                            let (x, y, z) = (c.channels[0], c.channels[1], c.channels[2]);
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => write!(
                                    f,
                                    "color(xyz {} {} {})",
                                    format_num(x),
                                    format_num(y),
                                    format_num(z)
                                ),
                                false => write!(
                                    f,
                                    "color(xyz {} {} {} / {})",
                                    format_num(x),
                                    format_num(y),
                                    format_num(z),
                                    format_alpha(c.a)
                                ),
                            }
                        }
                        ColorSpace::XyzD50 => {
                            let (x, y, z) = (c.channels[0], c.channels[1], c.channels[2]);
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => write!(
                                    f,
                                    "color(xyz-d50 {} {} {})",
                                    format_num(x),
                                    format_num(y),
                                    format_num(z)
                                ),
                                false => write!(
                                    f,
                                    "color(xyz-d50 {} {} {} / {})",
                                    format_num(x),
                                    format_num(y),
                                    format_num(z),
                                    format_alpha(c.a)
                                ),
                            }
                        }
                        ColorSpace::Rgb => {
                            // Auto + Rgb = hex / 命名色 / rgba
                            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                                true => match crate::eval::Evaluator::reverse_lookup_named_color(c) {
                                    Some(name) => write!(f, "{name}"),
                                    None => write!(
                                        f,
                                        "#{:02x}{:02x}{:02x}",
                                        c.legacy_rgb[0].round() as u8,
                                        c.legacy_rgb[1].round() as u8,
                                        c.legacy_rgb[2].round() as u8
                                    ),
                                },
                                false => write!(
                                    f,
                                    "rgba({}, {}, {}, {})",
                                    c.legacy_rgb[0].round() as u8,
                                    c.legacy_rgb[1].round() as u8,
                                    c.legacy_rgb[2].round() as u8,
                                    format_alpha(c.a)
                                ),
                            }
                        }
                    },
                }
            }
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
