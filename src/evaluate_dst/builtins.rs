//! Built-in function implementations (Map / List / String / Color).

use crate::shared::context::CompilerContext;

// ═══════════════════════════════════════════════════════════════════════════════
// Map builtins
// ═══════════════════════════════════════════════════════════════════════════════

fn arg_to_map(s: &str) -> Vec<(String, String)> {
    let trimmed = s.trim();
    let inner = trimmed
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(trimmed);
    let mut map = Vec::new();
    for pair in inner.split(',') {
        if let Some((k, v)) = pair.split_once(':') {
            map.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    map
}

pub(crate) fn builtin_map_get(args: &[String], ctx: &CompilerContext) -> String {
    if args.len() < 2 {
        return String::new();
    }
    let vars = ctx.global_variables.borrow();
    let map_text = super::resolve_vars_only(&vars, &args[0]);
    let key = super::resolve_vars_only(&vars, &args[1]);
    drop(vars);
    let map = arg_to_map(&map_text);
    for (k, v) in &map {
        if k.trim() == key.trim() {
            return v.clone();
        }
    }
    String::new()
}

pub(crate) fn builtin_map_has_key(args: &[String], ctx: &CompilerContext) -> String {
    if args.len() < 2 {
        return "false".to_string();
    }
    let vars = ctx.global_variables.borrow();
    let map_text = super::resolve_vars_only(&vars, &args[0]);
    let key = super::resolve_vars_only(&vars, &args[1]);
    drop(vars);
    let map = arg_to_map(&map_text);
    for (k, _) in &map {
        if k.trim() == key.trim() {
            return "true".to_string();
        }
    }
    "false".to_string()
}

pub(crate) fn builtin_map_keys(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return "()".to_string();
    }
    let map = arg_to_map(&args[0]);
    let keys: Vec<&str> = map.iter().map(|(k, _)| k.as_str()).collect();
    format!("({})", keys.join(", "))
}

pub(crate) fn builtin_map_values(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return "()".to_string();
    }
    let map = arg_to_map(&args[0]);
    let vals: Vec<&str> = map.iter().map(|(_, v)| v.as_str()).collect();
    format!("({})", vals.join(", "))
}

pub(crate) fn builtin_map_merge(args: &[String], _ctx: &CompilerContext) -> String {
    if args.len() < 2 {
        return String::new();
    }
    let mut merged = arg_to_map(&args[0]);
    for a in &args[1..] {
        merged.extend(arg_to_map(a));
    }
    let pairs: Vec<String> = merged
        .iter()
        .map(|(k, v)| format!("{k}: {v}"))
        .collect();
    format!("({})", pairs.join(", "))
}

// ═══════════════════════════════════════════════════════════════════════════════
// List builtins
// ═══════════════════════════════════════════════════════════════════════════════

fn split_list(s: &str) -> Vec<String> {
    let trimmed = s.trim();
    let inner = if trimmed.starts_with('(') && trimmed.ends_with(')') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for ch in inner.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                let t = current.trim().to_string();
                if !t.is_empty() {
                    parts.push(t);
                }
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    let t = current.trim().to_string();
    if !t.is_empty() {
        parts.push(t);
    }
    if parts.len() <= 1 && !inner.contains(',') {
        let by_ws: Vec<String> = inner
            .split_whitespace()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if by_ws.len() > parts.len() {
            return by_ws;
        }
    }
    parts
}

pub(crate) fn builtin_nth(args: &[String], ctx: &CompilerContext) -> String {
    if args.len() < 2 {
        return String::new();
    }
    let list = split_list(&super::substitute_vars(ctx, &args[0]));
    let idx: usize = super::substitute_vars(ctx, &args[1])
        .trim()
        .parse::<usize>()
        .unwrap_or(1)
        .saturating_sub(1);
    list.get(idx).cloned().unwrap_or_default()
}

pub(crate) fn builtin_length(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return "0".to_string();
    }
    let list = split_list(&args[0]);
    list.len().to_string()
}

pub(crate) fn builtin_append(args: &[String], ctx: &CompilerContext) -> String {
    if args.len() < 2 {
        return String::new();
    }
    let mut list = split_list(&args[0]);
    let val = super::substitute_vars(ctx, &args[1]);
    list.push(val);
    format!("({})", list.join(", "))
}

pub(crate) fn builtin_join(args: &[String], ctx: &CompilerContext) -> String {
    if args.len() < 2 {
        return String::new();
    }
    let list1 = split_list(&args[0]);
    let list2 = split_list(&super::substitute_vars(ctx, &args[1]));
    let mut result = list1;
    result.extend(list2);
    format!("({})", result.join(", "))
}

pub(crate) fn builtin_index(args: &[String], ctx: &CompilerContext) -> String {
    if args.len() < 2 {
        return "0".to_string();
    }
    let list = split_list(&args[0]);
    let needle = super::substitute_vars(ctx, &args[1]);
    list.iter()
        .position(|item| item.trim() == needle.trim())
        .map(|p| (p + 1).to_string())
        .unwrap_or_else(|| "0".to_string())
}

// ═══════════════════════════════════════════════════════════════════════════════
// String builtins
// ═══════════════════════════════════════════════════════════════════════════════

pub(crate) fn strip_quotes(s: &str) -> String {
    s.trim()
        .trim_start_matches('"')
        .trim_start_matches('\'')
        .trim_end_matches('"')
        .trim_end_matches('\'')
        .to_string()
}

fn add_quotes(s: &str) -> String {
    if s.starts_with('"') || s.starts_with('\'') {
        s.to_string()
    } else {
        format!("\"{s}\"")
    }
}

pub(crate) fn builtin_unquote(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return String::new();
    }
    strip_quotes(&args[0])
}

pub(crate) fn builtin_quote(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return String::new();
    }
    add_quotes(&args[0])
}

pub(crate) fn builtin_str_length(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return "0".to_string();
    }
    let s = strip_quotes(&args[0]);
    s.chars().count().to_string()
}

pub(crate) fn builtin_str_index(args: &[String], _ctx: &CompilerContext) -> String {
    if args.len() < 2 {
        return "0".to_string();
    }
    let s = strip_quotes(&args[0]);
    let sub = strip_quotes(&args[1]);
    s.find(sub.as_str())
        .map(|p| (p + 1).to_string())
        .unwrap_or_else(|| "0".to_string())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Color helpers & color builtins
// ═══════════════════════════════════════════════════════════════════════════════

fn parse_hex_color(s: &str) -> Option<(u8, u8, u8, f64)> {
    let s = s.trim_start_matches('#');
    match s.len() {
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some((r, g, b, 1.0))
        }
        3 => {
            let r = u8::from_str_radix(&s[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&s[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&s[2..3].repeat(2), 16).ok()?;
            Some((r, g, b, 1.0))
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            let a = u8::from_str_radix(&s[6..8], 16).ok()?;
            Some((r, g, b, a as f64 / 255.0))
        }
        _ => None,
    }
}

pub(crate) fn color_name_to_rgb(name: &str) -> Option<(u8, u8, u8)> {
    match name.trim().to_lowercase().as_str() {
        "red" => Some((255, 0, 0)),
        "green" => Some((0, 128, 0)),
        "blue" => Some((0, 0, 255)),
        "white" => Some((255, 255, 255)),
        "black" => Some((0, 0, 0)),
        "yellow" => Some((255, 255, 0)),
        "cyan" | "aqua" => Some((0, 255, 255)),
        "magenta" | "fuchsia" => Some((255, 0, 255)),
        "silver" => Some((192, 192, 192)),
        "gray" | "grey" => Some((128, 128, 128)),
        "maroon" => Some((128, 0, 0)),
        "olive" => Some((128, 128, 0)),
        "lime" => Some((0, 255, 0)),
        "teal" => Some((0, 128, 128)),
        "navy" => Some((0, 0, 128)),
        "purple" => Some((128, 0, 128)),
        "orange" => Some((255, 165, 0)),
        "pink" => Some((255, 192, 203)),
        "transparent" => Some((0, 0, 0)),
        _ => None,
    }
}

pub(crate) fn parse_color(s: &str) -> (u8, u8, u8, f64) {
    let s = s.trim();
    if let Some((r, g, b, a)) = parse_hex_color(s) {
        return (r, g, b, a);
    }
    if let Some((r, g, b)) = color_name_to_rgb(s) {
        return (r, g, b, 1.0);
    }
    let nums: Vec<u8> = s
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|p| p.parse::<u8>().ok())
        .collect();
    if nums.len() >= 3 {
        (nums[0], nums[1], nums[2], if nums.len() >= 4 { nums[3] as f64 / 255.0 } else { 1.0 })
    } else {
        (0, 0, 0, 1.0)
    }
}

pub(crate) fn rgb_to_string(r: u8, g: u8, b: u8) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        r.clamp(0, 255),
        g.clamp(0, 255),
        b.clamp(0, 255)
    )
}

fn parse_pct(s: &str) -> f64 {
    s.trim().trim_end_matches('%').parse::<f64>().unwrap_or(0.0)
}

pub(crate) fn builtin_mix(args: &[String], _ctx: &CompilerContext) -> String {
    if args.len() < 3 {
        return String::new();
    }
    let (r1, g1, b1, _) = parse_color(&args[0]);
    let (r2, g2, b2, _) = parse_color(&args[1]);
    let w = parse_pct(&args[2]).clamp(0.0, 100.0) / 100.0;
    let r = ((r1 as f64) * (1.0 - w) + (r2 as f64) * w).round() as u8;
    let g = ((g1 as f64) * (1.0 - w) + (g2 as f64) * w).round() as u8;
    let b = ((b1 as f64) * (1.0 - w) + (b2 as f64) * w).round() as u8;
    rgb_to_string(r, g, b)
}

pub(crate) fn builtin_darken(args: &[String], ctx: &CompilerContext) -> String {
    if args.len() < 2 {
        return String::new();
    }
    let (r, g, b, _) = parse_color(&args[0]);
    let pct = parse_pct(&super::substitute_vars(ctx, &args[1])).clamp(0.0, 100.0);
    let factor = 1.0 - pct / 100.0;
    rgb_to_string(
        ((r as f64) * factor).round() as u8,
        ((g as f64) * factor).round() as u8,
        ((b as f64) * factor).round() as u8,
    )
}

pub(crate) fn builtin_lighten(args: &[String], ctx: &CompilerContext) -> String {
    if args.len() < 2 {
        return String::new();
    }
    let (r, g, b, _) = parse_color(&args[0]);
    let pct = parse_pct(&super::substitute_vars(ctx, &args[1])).clamp(0.0, 100.0);
    let factor = pct / 100.0;
    let r = ((r as f64) + (255.0 - r as f64) * factor).round() as u8;
    let g = ((g as f64) + (255.0 - g as f64) * factor).round() as u8;
    let b = ((b as f64) + (255.0 - b as f64) * factor).round() as u8;
    rgb_to_string(r, g, b)
}

pub(crate) fn builtin_alpha(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return "1".to_string();
    }
    let (_, _, _, a) = parse_color(&args[0]);
    format!("{a}")
}

pub(crate) fn builtin_color_component(name: &str, args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return "0".to_string();
    }
    let (r, g, b, _) = parse_color(&args[0]);
    match name {
        "red" => r.to_string(),
        "green" => g.to_string(),
        "blue" => b.to_string(),
        _ => "0".to_string(),
    }
}

pub(crate) fn builtin_grayscale(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return String::new();
    }
    let (r, g, b, _) = parse_color(&args[0]);
    let gray = (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64).round() as u8;
    rgb_to_string(gray, gray, gray)
}

pub(crate) fn builtin_invert(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return String::new();
    }
    let (r, g, b, _) = parse_color(&args[0]);
    rgb_to_string(255 - r, 255 - g, 255 - b)
}
