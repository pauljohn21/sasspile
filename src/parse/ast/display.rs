use super::*;

/// 格式化浮点数——截断到 10 位小数（与 Dart Sass 一致）。
fn format_num(n: f64) -> String {
    let n = (n * 1e10).round() / 1e10;
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// 清理颜色分量的浮点噪声——将极小值归零。
/// 当 |v| < 1e-6 时视为 0，避免矩阵系数精度不足导致的残留。
fn clean_num(v: f64) -> f64 {
    if v.abs() < 1e-6 { 0.0 } else { v }
}

/// 清理百分比分量——接近 0 或 100 时归整。
fn clean_pct(v: f64) -> f64 {
    if (v - 0.0).abs() < 1e-6 { 0.0 }
    else if (v - 100.0).abs() < 1e-4 { 100.0 }
    else { v }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n, None) => {
                if n.is_infinite() {
                    return write!(f, "calc(infinity)");
                }
                if n.is_nan() {
                    return write!(f, "calc(NaN)");
                }
                write!(f, "{}", format_num(*n))
            }
            Value::Number(n, Some(unit)) => {
                if n.is_infinite() {
                    return write!(f, "calc(infinity * 1{unit})");
                }
                if n.is_nan() {
                    return write!(f, "calc(NaN * 1{unit})");
                }
                write!(f, "{}{unit}", format_num(*n))
            }
            Value::String(s, true) => {
                let (quote, escaped) = Self::escape_quoted_string(s);
                write!(f, "{quote}{escaped}{quote}")
            }
            Value::String(s, false) => {
                // 未加引号的标识符也需要转义控制字符和加引号字符
                write!(f, "{}", Self::escape_css_ident(s))
            }
            Value::Color(c) => {
                match &c.format {
                    ColorFormat::Hsl(h, s, l) => {
                        let hue_str = format_hue(*h);
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "hsl({}, {}%, {}%)", hue_str, format_pct(*s), format_pct(*l))
                        } else {
                            write!(f, "hsla({}, {}%, {}%, {})", hue_str, format_pct(*s), format_pct(*l), format_alpha(c.a))
                        }
                    }
                    ColorFormat::Hwb(h, w, b) => {
                        let hue_str = format_hue(*h);
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "hwb({} {}% {}%)", hue_str, format_pct(*w), format_pct(*b))
                        } else {
                            write!(f, "hwb({} {}% {}% / {})", hue_str, format_pct(*w), format_pct(*b), format_alpha(c.a))
                        }
                    }
                    ColorFormat::Rgb => {
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "rgb({}, {}, {})", c.r, c.g, c.b)
                        } else {
                            write!(f, "rgba({}, {}, {}, {})", c.r, c.g, c.b, format_alpha(c.a))
                        }
                    }
                    ColorFormat::RgbPercent(h, s, l) => {
                        // 从 HSL 值精确计算 RGB 百分比
                        let (rp, gp, bp) = hsl_to_rgb_percent(*h, *s, *l);
                        // 检查是否匹配命名颜色，优先输出名称
                        if (c.a - 1.0).abs() < f64::EPSILON
                            && let Some(name) = crate::eval::Evaluator::reverse_lookup_named_color(c) {
                            write!(f, "{name}")
                        } else if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "rgb({}%, {}%, {}%)", format_pct_val(rp), format_pct_val(gp), format_pct_val(bp))
                        } else {
                            write!(f, "rgba({}%, {}%, {}%, {})", format_pct_val(rp), format_pct_val(gp), format_pct_val(bp), format_alpha(c.a))
                        }
                    }
                    ColorFormat::Lab(l, a, b) => {
                        let l_clean = clean_pct(*l);
                        let a_clean = clean_num(*a);
                        let b_clean = clean_num(*b);
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "lab({}% {} {})", format_num(l_clean), format_num(a_clean), format_num(b_clean))
                        } else {
                            write!(f, "lab({}% {} {} / {})", format_num(l_clean), format_num(a_clean), format_num(b_clean), format_alpha(c.a))
                        }
                    }
                    ColorFormat::Lch(l, ch, h) => {
                        let l_clean = clean_pct(*l);
                        let ch_clean = clean_num(*ch);
                        let h_str = if ch_clean == 0.0 { "none".to_string() } else { format!("{}deg", format_hue(*h)) };
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "lch({}% {} {})", format_num(l_clean), format_num(ch_clean), h_str)
                        } else {
                            write!(f, "lch({}% {} {} / {})", format_num(l_clean), format_num(ch_clean), h_str, format_alpha(c.a))
                        }
                    }
                    ColorFormat::Oklab(l, a, b) => {
                        let l_pct = clean_pct(*l * 100.0);
                        let a_clean = clean_num(*a);
                        let b_clean = clean_num(*b);
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "oklab({}% {} {})", format_num(l_pct), format_num(a_clean), format_num(b_clean))
                        } else {
                            write!(f, "oklab({}% {} {} / {})", format_num(l_pct), format_num(a_clean), format_num(b_clean), format_alpha(c.a))
                        }
                    }
                    ColorFormat::Oklch(l, ch, h) => {
                        let l_pct = clean_pct(*l * 100.0);
                        let ch_clean = clean_num(*ch);
                        let h_str = if ch_clean == 0.0 { "none".to_string() } else { format!("{}deg", format_hue(*h)) };
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "oklch({}% {} {})", format_num(l_pct), format_num(ch_clean), h_str)
                        } else {
                            write!(f, "oklch({}% {} {} / {})", format_num(l_pct), format_num(ch_clean), h_str, format_alpha(c.a))
                        }
                    }
                    ColorFormat::DisplayP3(r, g, b) => {
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "color(display-p3 {} {} {})", format_num(*r), format_num(*g), format_num(*b))
                        } else {
                            write!(f, "color(display-p3 {} {} {} / {})", format_num(*r), format_num(*g), format_num(*b), format_alpha(c.a))
                        }
                    }
                    ColorFormat::DisplayP3Linear(r, g, b) => {
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "color(display-p3-linear {} {} {})", format_num(*r), format_num(*g), format_num(*b))
                        } else {
                            write!(f, "color(display-p3-linear {} {} {} / {})", format_num(*r), format_num(*g), format_num(*b), format_alpha(c.a))
                        }
                    }
                    ColorFormat::Srgb(r, g, b) => {
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "color(srgb {} {} {})", format_num(*r), format_num(*g), format_num(*b))
                        } else {
                            write!(f, "color(srgb {} {} {} / {})", format_num(*r), format_num(*g), format_num(*b), format_alpha(c.a))
                        }
                    }
                    ColorFormat::SrgbLinear(r, g, b) => {
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "color(srgb-linear {} {} {})", format_num(*r), format_num(*g), format_num(*b))
                        } else {
                            write!(f, "color(srgb-linear {} {} {} / {})", format_num(*r), format_num(*g), format_num(*b), format_alpha(c.a))
                        }
                    }
                    ColorFormat::A98Rgb(r, g, b) => {
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "color(a98-rgb {} {} {})", format_num(*r), format_num(*g), format_num(*b))
                        } else {
                            write!(f, "color(a98-rgb {} {} {} / {})", format_num(*r), format_num(*g), format_num(*b), format_alpha(c.a))
                        }
                    }
                    ColorFormat::ProphotoRgb(r, g, b) => {
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "color(prophoto-rgb {} {} {})", format_num(*r), format_num(*g), format_num(*b))
                        } else {
                            write!(f, "color(prophoto-rgb {} {} {} / {})", format_num(*r), format_num(*g), format_num(*b), format_alpha(c.a))
                        }
                    }
                    ColorFormat::Rec2020(r, g, b) => {
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "color(rec2020 {} {} {})", format_num(*r), format_num(*g), format_num(*b))
                        } else {
                            write!(f, "color(rec2020 {} {} {} / {})", format_num(*r), format_num(*g), format_num(*b), format_alpha(c.a))
                        }
                    }
                    ColorFormat::XyzD65(x, y, z) => {
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "color(xyz {} {} {})", format_num(*x), format_num(*y), format_num(*z))
                        } else {
                            write!(f, "color(xyz {} {} {} / {})", format_num(*x), format_num(*y), format_num(*z), format_alpha(c.a))
                        }
                    }
                    ColorFormat::XyzD50(x, y, z) => {
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "color(xyz-d50 {} {} {})", format_num(*x), format_num(*y), format_num(*z))
                        } else {
                            write!(f, "color(xyz-d50 {} {} {} / {})", format_num(*x), format_num(*y), format_num(*z), format_alpha(c.a))
                        }
                    }
                    ColorFormat::Auto => {
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            // 检查是否为命名颜色，优先输出名称（如 red 而非 #ff0000）
                            if let Some(name) = crate::eval::Evaluator::reverse_lookup_named_color(c) {
                                write!(f, "{name}")
                            } else {
                                write!(f, "#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
                            }
                        } else {
                            write!(f, "rgba({}, {}, {}, {})", c.r, c.g, c.b, format_alpha(c.a))
                        }
                    }
                }
            }
            Value::List(elements, sep, bracketed) => {
                if elements.is_empty() {
                    if *bracketed {
                        return write!(f, "[]");
                    }
                    return Ok(());
                }
                let sep_str = match sep {
                    Separator::Comma => ", ",
                    Separator::Space => " ",
                    Separator::Slash => " / ",
                    Separator::Undecided => " ",
                };
                if *bracketed {
                    f.write_str("[")?;
                }
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 {
                        f.write_str(sep_str)?;
                    }
                    e.fmt(f)?;
                }
                if *bracketed {
                    f.write_str("]")?;
                }
                Ok(())
            }
            Value::Map(pairs) => {
                f.write_str("(")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
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
                if name == "if" && args.iter().any(|a| a.condition.is_some() || a.name.as_deref() == Some("else")) {
                    write!(f, "{name}(")?;
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 {
                            f.write_str("; ")?;
                        }
                        if let Some(cond) = &a.condition {
                            cond.fmt(f)?;
                            f.write_str(": ")?;
                            a.value.fmt(f)?;
                        } else if let Some(n) = &a.name {
                            write!(f, "{n}: ")?;
                            a.value.fmt(f)?;
                        } else {
                            a.value.fmt(f)?;
                        }
                    }
                    f.write_str(")")
                } else {
                    write!(f, "{name}(")?;
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 {
                            f.write_str(", ")?;
                        }
                        a.value.fmt(f)?;
                    }
                    f.write_str(")")
                }
            }
            Value::Interp(s) => write!(f, "#{{{s}}}"),
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
                UnaryOp::Neg => write!(f, "-{v}"),
                UnaryOp::Not => write!(f, "not {v}"),
            },
            Value::Calc(s) => write!(f, "{s}"),
            Value::Paren(v) => write!(f, "({v})"),
            Value::Spread(v) => write!(f, "{v}..."),
        }
    }
}

impl Value {
    /// 转义引用字符串中的特殊字符为 CSS 转义序列。
    ///
    /// 返回 (quote_char, escaped_content)。
    /// - 如果字符串包含 `"` 但不包含 `'`，用单引号包裹，避免转义
    /// - 否则用双引号包裹，转义 `"`
    /// - `\` → `\\`
    /// - 对加引号的字符串进行转义并选择引号字符。
    /// - NULL (U+0000) → `\0 ` (with trailing space if needed)
    /// - 控制字符和私有区字符 → `\XXXX` (lowercase hex)
    /// - 其他非 ASCII 字符保持原样（会触发 @charset 前缀）
    pub(crate) fn escape_quoted_string(s: &str) -> (char, String) {
        let has_double = s.contains('"');
        let has_single = s.contains("'");
        // 如果包含双引号但不含单引号，用单引号包裹
        let quote = if has_double && !has_single { '\'' } else { '"' };

        let escaped = Self::escape_css_chars(s, |c| {
            (c == '"' && quote == '"') || (c == '\'' && quote == '\'')
        });
        (quote, escaped)
    }

    /// 对未加引号的 CSS 标识符进行转义。
    /// 反斜杠 → `\\`，控制字符 → `\XXXX`，NULL → `\0 `。
    pub(crate) fn escape_css_ident(s: &str) -> String {
        Self::escape_css_chars(s, |_| false)
    }

    /// 核心转义逻辑——遍历字符并转义特殊字符。
    /// `is_quote` 判断当前字符是否为需要转义的引号。
    fn escape_css_chars(s: &str, is_quote: impl Fn(char) -> bool) -> String {
        let chars: Vec<char> = s.chars().collect();
        let mut result = String::new();
        for (i, &c) in chars.iter().enumerate() {
            match c {
                '\\' => result.push_str("\\\\"),
                c if is_quote(c) => {
                    result.push('\\');
                    result.push(c);
                }
                '\0' => result.push_str("\\0 "),
                c if c.is_control() || ('\u{E000}'..='\u{F8FF}').contains(&c) => {
                    let hex = format!("{:x}", c as u32);
                    result.push('\\');
                    result.push_str(&hex);
                    // 仅在下一个字符是十六进制数字或空白时添加空格终止转义
                    let next = chars.get(i + 1).copied();
                    if next.is_some_and(|nc| nc.is_ascii_hexdigit() || nc.is_whitespace()) {
                        result.push(' ');
                    }
                }
                _ => result.push(c),
            }
        }
        result
    }
}
