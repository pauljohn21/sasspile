//! Sass 内置函数注册表 — 响应式风格函数求值
//!
//! 设计:
//!   - 纯函数注册表: &[(&str, fn(&[String]) -> Option<String>)]
//!   - dispatch 时 try_eval_builtin 尝试替换字面量
//!   - 不持有状态 (状态在 CompileState 中)
//!   - 借用 AST 输入, 输出 Option<String>
//!
//! 模块拆分 (single-file ≤ 500):
//!   - math_trig.rs: 三角函数/高级数学
//!   - color.rs: 颜色函数
//!   - string.rs: 字符串函数

mod math_trig;
mod color;
mod string;

// ═══════════════════════════════════════════════════════════════════════════
// 公共 API
// ═══════════════════════════════════════════════════════════════════════════

type BuiltinFn = fn(&[String]) -> Option<String>;

pub const BUILTINS: &[(&str, BuiltinFn)] = &[
    // ── 颜色函数 (color 模块) ─────────────────────────────────────────
    ("rgba",       color::eval_rgba),
    ("rgb",        color::eval_rgb_call),
    ("hsl",        color::eval_hsl_call),
    ("lighten",    color::eval_lighten),
    ("darken",     color::eval_darken),
    ("mix",        color::eval_mix),
    ("adjust-hue", color::eval_adjust_hue),

    // ── 数学函数 ─────────────────────────────────────────────────────
    ("round",      eval_round),
    ("ceil",       eval_ceil),
    ("floor",      eval_floor),
    ("abs",        eval_abs),
    ("min",        eval_min),
    ("max",        eval_max),
    ("percentage", eval_percentage),
    // ── 三角函数 (math_trig 模块) ───────────────────────────────────
    ("sin",        math_trig::eval_sin),
    ("cos",        math_trig::eval_cos),
    ("tan",        math_trig::eval_tan),
    ("asin",       math_trig::eval_asin),
    ("acos",       math_trig::eval_acos),
    ("atan",       math_trig::eval_atan),
    ("sqrt",       math_trig::eval_sqrt),
    ("pow",        math_trig::eval_pow),
    ("log",        math_trig::eval_log),
    ("exp",        math_trig::eval_exp),
    ("sign",       math_trig::eval_sign),
    ("clamp",      math_trig::eval_clamp),
    ("mod",        math_trig::eval_mod),

    // ── 字符串函数 (string 模块) ─────────────────────────────────────
    ("unquote",      string::eval_unquote),
    ("quote",        string::eval_quote),
    ("str-length",   string::eval_str_length),
    ("str-index",    string::eval_str_index),
    ("str-slice",    string::eval_str_slice),
    ("to-upper-case", string::eval_to_upper_case),
    ("to-lower-case", string::eval_to_lower_case),

    // ── 列表函数 ──────────────────────────────────────────────────────
    ("length",  eval_length),
    ("nth",     eval_nth),
];

/// 尝试解析并求值内置函数调用 (如 "rgba(255, 0, 0, 0.5)")
pub fn try_eval_builtin(input: &str) -> Option<String> {
    let (open, close) = find_parens(input)?;
    let func_name = extract_func_name(&input[..open])?;
    let inner = &input[open + 1..close];

    let args = split_args(inner);

    BUILTINS.iter()
        .find(|(name, _)| *name == func_name)
        .and_then(|(_, f)| f(&args))
}

// ═══════════════════════════════════════════════════════════════════════════
// 基础数学函数实现
// ═══════════════════════════════════════════════════════════════════════════

fn split_unit(s: &str) -> Option<(f64, &str)> {
    let s = s.trim();
    // 找单位起始: 第一个非数字/非点/非负号字符; 无单位则整个字符串是数字
    match s.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-') {
        Some(split_pos) => {
            let num = s[..split_pos].parse::<f64>().ok()?;
            Some((num, &s[split_pos..]))
        }
        None => {
            let n = s.parse::<f64>().ok()?;
            Some((n, ""))
        }
    }
}

fn eval_math_with_unit(args: &[String], op: fn(f64) -> f64) -> Option<String> {
    let s = args.first()?.trim();
    if s.contains(|c: char| !c.is_ascii_digit() && c != '.' && c != '-') {
        let (num, unit) = split_unit(s)?;
        Some(format!("{}{unit}", op(num)))
    } else {
        Some(format!("{}", op(s.parse::<f64>().ok()?)))
    }
}

fn eval_round(args: &[String]) -> Option<String> {
    eval_math_with_unit(args, f64::round)
}

fn eval_ceil(args: &[String]) -> Option<String> {
    eval_math_with_unit(args, f64::ceil)
}

fn eval_floor(args: &[String]) -> Option<String> {
    eval_math_with_unit(args, f64::floor)
}

fn eval_abs(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    if s.contains(|c: char| !c.is_ascii_digit() && c != '.' && c != '-') {
        let (num, unit) = split_unit(s)?;
        Some(format!("{}{unit}", num.abs()))
    } else {
        let n = s.parse::<f64>().ok()?;
        Some(format!("{}", n.abs()))
    }
}

fn eval_min(args: &[String]) -> Option<String> {
    let first = args.first()?.trim();
    let unit = split_unit(first).map(|(_, u)| u).unwrap_or("");
    args.iter()
        .map(|s| split_unit(s).map(|(n, _)| n))
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .map(|v| format!("{v}{unit}"))
}

fn eval_max(args: &[String]) -> Option<String> {
    let first = args.first()?.trim();
    let unit = split_unit(first).map(|(_, u)| u).unwrap_or("");
    args.iter()
        .map(|s| split_unit(s).map(|(n, _)| n))
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .map(|v| format!("{v}{unit}"))
}

fn eval_percentage(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let val = if s.contains(|c: char| !c.is_ascii_digit() && c != '.' && c != '-') {
        split_unit(s)?.0
    } else {
        s.parse::<f64>().ok()?
    };
    Some(format!("{}%", (val * 100.0).round() as i64))
}

// ═══════════════════════════════════════════════════════════════════════════
// 列表函数实现
// ═══════════════════════════════════════════════════════════════════════════

fn eval_length(args: &[String]) -> Option<String> {
    if args.is_empty() {
        Some("0".to_string())
    } else if args.len() == 1 {
        let s = args.first()?.trim();
        if s.starts_with('(') || s.contains(',') {
            let items: Vec<&str> = s.trim_start_matches('(').trim_end_matches(')')
                .split(',').map(str::trim).filter(|x| !x.is_empty()).collect();
            Some(items.len().to_string())
        } else {
            Some("1".to_string())
        }
    } else {
        Some(args.len().to_string())
    }
}

fn eval_nth(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let n = args.get(1)?.trim().parse::<usize>().ok()?;
    let items: Vec<&str> = s.trim_start_matches('(').trim_end_matches(')')
        .split(',').map(str::trim).filter(|x| !x.is_empty()).collect();
    items.get(n - 1).map(|v| v.to_string())
}

// ═══════════════════════════════════════════════════════════════════════════
// 颜色辅助函数（各子模块共用）
// ═══════════════════════════════════════════════════════════════════════════

fn parse_color(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some((r, g, b))
            }
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some((r, g, b))
            }
            _ => None,
        }
    } else {
        match s.to_lowercase().as_str() {
            "red" => Some((255, 0, 0)),
            "green" => Some((0, 128, 0)),
            "blue" => Some((0, 0, 255)),
            "white" => Some((255, 255, 255)),
            "black" => Some((0, 0, 0)),
            "yellow" => Some((255, 255, 0)),
            "cyan" => Some((0, 255, 255)),
            "magenta" => Some((255, 0, 255)),
            "gray" | "grey" => Some((128, 128, 128)),
            "transparent" => Some((0, 0, 0)),
            _ => None,
        }
    }
}

fn strip_pct(s: &str) -> Option<f64> {
    s.trim().trim_end_matches('%').parse::<f64>().ok()
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r1 = r as f64 / 255.0;
    let g1 = g as f64 / 255.0;
    let b1 = b as f64 / 255.0;

    let max = r1.max(g1).max(b1);
    let min = r1.min(g1).min(b1);
    let delta = max - min;

    let l = (max + min) / 2.0;

    if delta == 0.0 {
        return (0.0, 0.0, l);
    }

    let s = if l > 0.5 { delta / (2.0 - max - min) } else { delta / (max + min) };

    let h = if max == r1 {
        ((g1 - b1) / delta) % 6.0
    } else if max == g1 {
        (b1 - r1) / delta + 2.0
    } else {
        (r1 - g1) / delta + 4.0
    };

    let mut h_deg = h * 60.0;
    if h_deg < 0.0 { h_deg += 360.0; }

    (h_deg, s, l)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let (r1, g1, b1) = match h_prime {
        hp if hp >= 0.0 && hp < 1.0 => (c, x, 0.0),
        hp if hp >= 1.0 && hp < 2.0 => (x, c, 0.0),
        hp if hp >= 2.0 && hp < 3.0 => (0.0, c, x),
        hp if hp >= 3.0 && hp < 4.0 => (0.0, x, c),
        hp if hp >= 4.0 && hp < 5.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    let r = ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (r, g, b)
}

// ═══════════════════════════════════════════════════════════════════════════
// 辅助函数 (纯函数, 零副作用)
// ═══════════════════════════════════════════════════════════════════════════

fn find_parens(input: &str) -> Option<(usize, usize)> {
    let open = input.rfind('(')?;
    let close = input[open..].find(')').map(|p| open + p)?;
    Some((open, close))
}

fn extract_func_name(prefix: &str) -> Option<&str> {
    let end = prefix.rfind(|c: char| c.is_alphanumeric() || c == '-')?;
    Some(&prefix[..=end])
}

fn split_args(inner: &str) -> Vec<String> {
    if inner.trim().is_empty() {
        return vec![];
    }
    inner.split(',').map(|s| s.trim().to_string()).collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// 公开入口: 扫描字符串中所有函数调用并替换为求值结果
// ═══════════════════════════════════════════════════════════════════════════

pub fn eval_all_calls(input: &str) -> String {
    let mut result = input.to_string();

    while let Some((start, end)) = find_last_call(&result) {
        let call = &result[start..=end];
        if let Some(replacement) = try_eval_builtin(call) {
            result.replace_range(start..=end, &replacement);
        } else {
            break;
        }
    }

    result
}

fn find_last_call(input: &str) -> Option<(usize, usize)> {
    let close = input.rfind(')')?;
    let open = input[..close].rfind('(')?;
    let name_end = input[..open].rfind(|c: char| c.is_alphanumeric() || c == '-')?;
    let name_start = input[..name_end].rfind(|c: char| !c.is_alphanumeric() && c != '-' && c != '$')
        .map(|p| p + 1)
        .unwrap_or(0);
    Some((name_start, close))
}
