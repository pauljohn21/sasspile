use super::*;

/// 格式化单位为 CSS 字符串（不带前缀）。
/// - `"px"` → `"1px"`
/// - `"px*em"` → `"1px * 1em"`
/// - `"px/em"` → `"1px / 1em"`
/// - `"/px"` → `"/ 1px"`
/// - `"px*em/s"` → `"1px * 1em / 1s"`
fn format_unit(unit: &str) -> String {
    let mut tokens: Vec<String> = Vec::new();
    let (numerator, denominator) = if let Some(rest) = unit.strip_prefix('/') {
        (String::new(), rest)
    } else {
        let split: Vec<&str> = unit.splitn(2, '/').collect();
        (split[0].to_string(), split.get(1).copied().unwrap_or(""))
    };
    // 分子（* 分隔）
    let num_units: Vec<&str> = numerator.split('*').filter(|u| !u.is_empty()).collect();
    for (i, u) in num_units.iter().enumerate() {
        if i > 0 {
            tokens.push("*".to_string());
        }
        tokens.push(format!("1{u}"));
    }
    // 分母（* 分隔）
    let den_units: Vec<&str> = denominator.split('*').filter(|u| !u.is_empty()).collect();
    for u in den_units.iter() {
        tokens.push("/".to_string());
        tokens.push(format!("1{u}"));
    }
    if tokens.is_empty() {
        String::new()
    } else {
        tokens.join(" ")
    }
}

/// 安全格式化 f64 整数值——避免 i64 溢出。
/// 当值超出 i64 范围时使用原始 f64 的整数形式。
fn fmt_int(val: f64) -> String {
    if val >= i64::MAX as f64 {
        // 超出 i64 范围，直接截断小数位输出
        format!("{:.0}", val)
    } else if val <= i64::MIN as f64 {
        format!("{:.0}", val)
    } else {
        format!("{}", val as i64)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n, None) => {
                if n.is_infinite() {
                    return if n.is_sign_negative() {
                        write!(f, "calc(-infinity)")
                    } else {
                        write!(f, "calc(infinity)")
                    };
                }
                if n.is_nan() {
                    return write!(f, "calc(NaN)");
                }
                // Dart Sass 精度规则：
                // 1. 绝对值 < 1e-10 显示为 0
                // 2. 接近整数（距离 < 1e-10）四舍五入为整数
                if n.abs() < 1e-10 {
                    return write!(f, "0");
                }
                let rounded = n.round();
                if (n - rounded).abs() < 1e-10 {
                    return write!(f, "{}", fmt_int(rounded));
                }
                if n.fract() == 0.0 {
                    write!(f, "{}", fmt_int(*n))
                } else {
                    write!(f, "{n}")
                }
            }
            Value::Number(n, Some(unit)) => {
                let formatted = format_unit(unit);
                let prefix = if unit.starts_with('/') { "" } else { "* " };
                if n.is_infinite() {
                    return if n.is_sign_negative() {
                        write!(f, "calc(-infinity {prefix}{formatted})")
                    } else {
                        write!(f, "calc(infinity {prefix}{formatted})")
                    };
                }
                if n.is_nan() {
                    return write!(f, "calc(NaN {prefix}{formatted})");
                }
                // Dart Sass 精度规则
                if n.abs() < 1e-10 {
                    return write!(f, "0");
                }
                let rounded = n.round();
                if (n - rounded).abs() < 1e-10 {
                    // 复合单位用空格分隔：1px * 1em 或 1px / 1em
                    if unit.contains('*') || unit.contains('/') {
                        return write!(f, "{} {}", fmt_int(rounded), formatted);
                    }
                    return write!(f, "{}{unit}", fmt_int(rounded));
                }
                if n.fract() == 0.0 {
                    // 复合单位用空格分隔：1px * 1em 或 1px / 1em
                    if unit.contains('*') || unit.contains('/') {
                        return write!(f, "{} {}", fmt_int(*n), formatted);
                    }
                    write!(f, "{}{unit}", fmt_int(*n))
                } else {
                    write!(f, "{n}{unit}")
                }
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
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "hsl({}, {}%, {}%)", format_hue(*h), format_pct(*s), format_pct(*l))
                        } else {
                            write!(f, "hsla({}, {}%, {}%, {})", format_hue(*h), format_pct(*s), format_pct(*l), format_alpha(c.a))
                        }
                    }
                    ColorFormat::Hwb(h, w, b) => {
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "hwb({} {}% {}%)", format_hue(*h), format_pct(*w), format_pct(*b))
                        } else {
                            write!(f, "hwb({} {}% {}% / {})", format_hue(*h), format_pct(*w), format_pct(*b), format_alpha(c.a))
                        }
                    }
                    ColorFormat::Rgb => {
                        if (c.a - 1.0).abs() < f64::EPSILON {
                            write!(f, "rgb({}, {}, {})", c.r, c.g, c.b)
                        } else {
                            write!(f, "rgba({}, {}, {}, {})", c.r, c.g, c.b, format_alpha(c.a))
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
                    Separator::Slash => "/",
                    Separator::SlashDiv => "/",
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
            Value::Raw(s) => write!(f, "{s}"),
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
    /// 反斜杠 → `\\`，控制字符 → `\XXXX`，NULL → `\0 `，
    /// 前导数字 → `\XX `（CSS 标识符不能以数字开头）。
    pub(crate) fn escape_css_ident(s: &str) -> String {
        let chars: Vec<char> = s.chars().collect();
        let mut result = String::new();

        for (i, &c) in chars.iter().enumerate() {
            if c.is_ascii_digit() && (i == 0 || (i == 1 && chars[0] == '-')) {
                // 前导数字（包括 -后跟数字）需要十六进制转义
                let hex = format!("{:x}", c as u32);
                result.push('\\');
                result.push_str(&hex);
                // CSS 规范：转义产生的数字后必须跟空格终止（当下一个字符存在且不是空白时）
                let next = chars.get(i + 1).copied();
                if next.is_some_and(|nc| !nc.is_whitespace()) {
                    result.push(' ');
                }
            } else if c == '$' {
                // $ 在 CSS 标识符中需要保留转义（Dart Sass 行为）
                result.push_str("\\$");
            } else {
                // 其余字符使用标准转义逻辑（反斜杠、控制字符、NULL、私有区字符）
                let fragment = Self::escape_css_chars(&c.to_string(), |_| false);
                result.push_str(&fragment);
            }
        }
        result
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
