use crate::types::{ListSeparator, Value};

/// Dispatch a built-in function call by name.
/// Returns Some(Value) if the function was handled, None if unknown.
pub fn call_builtin(name: &str, args: &[Value]) -> Option<Value> {
    match name {
        // ── Map functions ──────────────────────────────────────────────────
        "map-get" => map_get(args),
        "map-has-key" => map_has_key(args),
        "map-merge" => map_merge(args),
        "map-keys" => map_keys(args),
        "map-values" => map_values(args),
        "map-loop" => map_loop(args),

        // ── Conditional ────────────────────────────────────────────────────
        "if" => if_fn(args),

        // ── List functions ─────────────────────────────────────────────────
        "nth" => nth(args),
        "join" => list_join(args),
        "append" => list_append(args),
        "length" => list_length(args),
        "separator" | "list-separator" => list_separator(args),
        "zip" | "list-zip" => list_zip(args),

        // ── Math functions ─────────────────────────────────────────────────
        "percentage" => percentage(args),
        "round" => math_round(args),
        "ceil" => math_ceil(args),
        "floor" => math_floor(args),
        "abs" => math_abs(args),
        "min" => math_min(args),
        "max" => math_max(args),
        "random" => math_random(args),

        // ── Color functions ────────────────────────────────────────────────
        "mix" => color_mix(args),
        "shade-color" => color_shade(args),
        "tint-color" => color_tint(args),
        "to-rgb" => color_to_rgb(args),
        "red" => color_channel(args, "red"),
        "green" => color_channel(args, "green"),
        "blue" => color_channel(args, "blue"),
        "alpha" => color_channel(args, "alpha"),
        "lighten" => color_lighten(args),
        "darken" => color_darken(args),
        "saturate" => color_saturate(args),
        "desaturate" => color_desaturate(args),
        "rgba" => color_rgb_rgba(args, "rgba"),
        "rgb" => color_rgb_rgba(args, "rgb"),
        "var" => var_function(args),
        "transparentize" | "fade-out" => color_transparentize(args),
        "opacify" | "fade-in" => color_opacify(args),
        "invert" => color_invert(args),
        "complement" => color_complement(args),

        // ── Type / meta ────────────────────────────────────────────────────
        "type-of" => type_of(args),
        "unit" => unit(args),
        "inspect" => inspect(args),
        "not" => not_fn(args),

        // ── String ─────────────────────────────────────────────────────────
        "quote" => quote(args),
        "unquote" => unquote(args),
        "str-length" => str_length(args),
        "str-index" => str_index(args),
        "str-slice" => str_slice(args),
        "str-replace" => str_replace(args),
        "to-upper-case" => to_upper_case(args),
        "to-lower-case" => to_lower_case(args),

        _ => None,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Map functions
// ─────────────────────────────────────────────────────────────────────────

fn map_get(args: &[Value]) -> Option<Value> {
    let map = args.first()?;
    let key = args.get(1)?;
    match map {
        Value::Map(entries) => {
            let key_str = key.to_string();
            Some(entries.iter().find(|(k, _)| k == &key_str).map(|(_, v)| v.clone()).unwrap_or(Value::Null))
        }
        _ => Some(Value::Null),
    }
}

fn map_has_key(args: &[Value]) -> Option<Value> {
    let map = args.first()?;
    let key = args.get(1)?;
    match map {
        Value::Map(entries) => {
            let key_str = key.to_string();
            Some(Value::Bool(entries.iter().any(|(k, _)| k == &key_str)))
        }
        _ => Some(Value::Bool(false)),
    }
}

fn map_merge(args: &[Value]) -> Option<Value> {
    let mut result = Vec::new();
    for arg in args {
        if let Value::Map(entries) = arg {
            for (k, v) in entries {
                if let Some(pos) = result.iter().position(|(rk, _)| rk == k) {
                    result[pos] = (k.clone(), v.clone());
                } else {
                    result.push((k.clone(), v.clone()));
                }
            }
        }
    }
    Some(Value::Map(result))
}

fn map_keys(args: &[Value]) -> Option<Value> {
    let map = args.first()?;
    match map {
        Value::Map(entries) => Some(Value::List(
            entries.iter().map(|(k, _)| Value::String(k.clone())).collect(),
            ListSeparator::Comma,
        )),
        _ => Some(Value::List(Vec::new(), ListSeparator::Comma)),
    }
}

fn map_values(args: &[Value]) -> Option<Value> {
    let map = args.first()?;
    match map {
        Value::Map(entries) => Some(Value::List(
            entries.iter().map(|(_, v)| v.clone()).collect(),
            ListSeparator::Comma,
        )),
        _ => Some(Value::List(Vec::new(), ListSeparator::Comma)),
    }
}

/// map-loop($map, $func, $args...): 遍历 map，对每个 value 调用指定函数，返回新 map
/// $args 中 "$key" 替换为键名，"$value" 替换为值，其余参数原样传递
fn map_loop(args: &[Value]) -> Option<Value> {
    let map = args.first()?;
    let func_name = args.get(1)?.to_string();
    let extra_args = &args[2..];
    match map {
        Value::Map(entries) => {
            let mut result = Vec::new();
            for (key, value) in entries {
                // 构建实际参数列表：替换 "$key" 和 "$value"
                let call_args: Vec<Value> = extra_args.iter().map(|arg| {
                    let s = arg.to_string();
                    if s == "$key" {
                        Value::String(key.clone())
                    } else if s == "$value" {
                        value.clone()
                    } else {
                        arg.clone()
                    }
                }).collect();
                if let Some(func_result) = call_builtin(&func_name, &call_args) {
                    result.push((key.clone(), func_result));
                } else {
                    result.push((key.clone(), value.clone()));
                }
            }
            Some(Value::Map(result))
        }
        _ => Some(Value::Map(Vec::new())),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Conditional
// ─────────────────────────────────────────────────────────────────────────

fn if_fn(args: &[Value]) -> Option<Value> {
    let cond = args.first()?;
    let if_true = args.get(1)?;
    let if_false = args.get(2)?;
    if truthy(cond) { Some(if_true.clone()) } else { Some(if_false.clone()) }
}

// ─────────────────────────────────────────────────────────────────────────
// List functions
// ─────────────────────────────────────────────────────────────────────────

fn nth(args: &[Value]) -> Option<Value> {
    let list = args.first()?;
    let n = args.get(1)?;
    let n_val = value_to_int(n);
    if n_val < 1 {
        return Some(Value::Null);
    }
    match list {
        Value::List(items, _) => Some(items.get(n_val - 1).cloned().unwrap_or(Value::Null)),
        // For maps, convert to list of [key, value] pairs and pick the nth
        Value::Map(entries) => {
            let pair = entries.get(n_val - 1)?;
            Some(Value::List(vec![Value::String(pair.0.clone()), pair.1.clone()], ListSeparator::Comma))
        }
        _ => Some(list.clone()),
    }
}

fn list_join(args: &[Value]) -> Option<Value> {
    let list1 = args.first()?;
    let list2 = args.get(1)?;
    let _sep = args.get(2);
    let items1 = list_to_items(list1);
    let items2 = list_to_items(list2);
    let mut all = items1;
    all.extend(items2);
    Some(Value::List(all, ListSeparator::Comma))
}

fn list_append(args: &[Value]) -> Option<Value> {
    let list = args.first()?;
    let val = args.get(1)?;
    let items = list_to_items(list);
    let mut result = items;
    result.push(val.clone());
    Some(Value::List(result, ListSeparator::Comma))
}

fn list_length(args: &[Value]) -> Option<Value> {
    let list = args.first()?;
    match list {
        Value::List(items, _) => Some(Value::Number(items.len() as f64, None)),
        Value::Map(entries) => Some(Value::Number(entries.len() as f64, None)),
        _ => Some(Value::Number(1.0, None)),
    }
}

fn list_separator(args: &[Value]) -> Option<Value> {
    let list = args.first()?;
    match list {
        // Empty list or single item → "space" (default Sass behavior)
        Value::List(items, _) if items.len() <= 1 => Some(Value::String("space".to_string())),
        // Multi-item list → "comma" (most common in Bootstrap SCSS)
        Value::List(_, _) => Some(Value::String("comma".to_string())),
        // Map with entries acts as comma-separated list
        Value::Map(entries) if !entries.is_empty() => Some(Value::String("comma".to_string())),
        _ => Some(Value::String("space".to_string())),
    }
}

/// list-zip($lists...) — combine multiple lists into nested lists by position
fn list_zip(args: &[Value]) -> Option<Value> {
    if args.is_empty() {
        return Some(Value::List(Vec::new(), ListSeparator::Comma));
    }
    // Convert each arg into a list of items
    let lists: Vec<Vec<Value>> = args.iter().map(list_to_items).collect();
    // Find the minimum length across all lists
    let min_len = lists.iter().map(|l| l.len()).min().unwrap_or(0);
    // Build nested lists
    let zipped: Vec<Value> = (0..min_len)
        .map(|i| Value::List(lists.iter().map(|l| l[i].clone()).collect(), ListSeparator::Comma))
        .collect();
    Some(Value::List(zipped, ListSeparator::Comma))
}

// ─────────────────────────────────────────────────────────────────────────
// Math functions
// ─────────────────────────────────────────────────────────────────────────

fn percentage(args: &[Value]) -> Option<Value> {
    let n = args.first()?;
    match n {
        Value::Number(v, _) => Some(Value::Number(v * 100.0, Some("%".to_string()))),
        _ => Some(Value::Number(100.0, Some("%".to_string()))),
    }
}

fn math_round(args: &[Value]) -> Option<Value> {
    let n = args.first()?;
    match n {
        Value::Number(v, u) => Some(Value::Number(v.round(), u.clone())),
        _ => Some(Value::Number(0.0, None)),
    }
}

fn math_ceil(args: &[Value]) -> Option<Value> {
    let n = args.first()?;
    match n {
        Value::Number(v, u) => Some(Value::Number(v.ceil(), u.clone())),
        _ => Some(Value::Number(0.0, None)),
    }
}

fn math_floor(args: &[Value]) -> Option<Value> {
    let n = args.first()?;
    match n {
        Value::Number(v, u) => Some(Value::Number(v.floor(), u.clone())),
        _ => Some(Value::Number(0.0, None)),
    }
}

fn math_abs(args: &[Value]) -> Option<Value> {
    let n = args.first()?;
    match n {
        Value::Number(v, u) => Some(Value::Number(v.abs(), u.clone())),
        _ => Some(Value::Number(0.0, None)),
    }
}

fn math_min(args: &[Value]) -> Option<Value> {
    let nums: Vec<f64> = args.iter().filter_map(|v| match v { Value::Number(n, _) => Some(*n), _ => None }).collect();
    nums.into_iter().reduce(f64::min).map(|n| Value::Number(n, None))
}

fn math_max(args: &[Value]) -> Option<Value> {
    let nums: Vec<f64> = args.iter().filter_map(|v| match v { Value::Number(n, _) => Some(*n), _ => None }).collect();
    nums.into_iter().reduce(f64::max).map(|n| Value::Number(n, None))
}

fn math_random(args: &[Value]) -> Option<Value> {
    let max = args.first().map(value_to_int).unwrap_or(100) as f64;
    // Simple LCG deterministic pseudo-random for reproducibility
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as f64;
    let val = ((seed.sin() + 1.0) / 2.0) * max;
    Some(Value::Number(val, None))
}

// ─────────────────────────────────────────────────────────────────────────
// Color functions
// ─────────────────────────────────────────────────────────────────────────

fn color_mix(args: &[Value]) -> Option<Value> {
    let c1 = args.first()?;
    let c2 = args.get(1)?;
    let weight = args.get(2).map(value_to_number).unwrap_or(50.0);
    // weight 是 c1 的占比；mix_channel(a, b, w) 中 w 是 b 的权重，所以需要反转
    let w = 1.0 - (weight / 100.0).clamp(0.0, 1.0);
    match (c1, c2) {
        (Value::Color(r1, g1, b1, a1), Value::Color(r2, g2, b2, a2)) => {
            let r = mix_channel(*r1, *r2, w);
            let g = mix_channel(*g1, *g2, w);
            let b = mix_channel(*b1, *b2, w);
            let a = mix_alpha(*a1, *a2, w);
            Some(Value::Color(r, g, b, a))
        }
        _ => Some(Value::Null),
    }
}

fn color_lighten(args: &[Value]) -> Option<Value> {
    let color = args.first()?;
    let amount = args.get(1).map(value_to_number).unwrap_or(0.0);
    match color {
        Value::Color(r, g, b, a) => {
            // lighten($color, 20%) = mix(white, $color, 20%) = 20% white + 80% color
            // mix_channel_255(r, 255, t) 中 t 是 255(white) 的权重 = amount/100
            let t = (amount / 100.0).clamp(0.0, 1.0);
            Some(Value::Color(
                mix_channel_255(*r, 255, t),
                mix_channel_255(*g, 255, t),
                mix_channel_255(*b, 255, t),
                *a,
            ))
        }
        _ => Some(color.clone()),
    }
}

fn color_darken(args: &[Value]) -> Option<Value> {
    let color = args.first()?;
    let amount = args.get(1).map(value_to_number).unwrap_or(0.0);
    match color {
        Value::Color(r, g, b, a) => {
            // darken($color, 20%) = mix(black, $color, 20%) = 20% black + 80% color
            // mix_channel_255(r, 0, t) 中 t 是 0(black) 的权重 = amount/100
            let t = (amount / 100.0).clamp(0.0, 1.0);
            Some(Value::Color(
                mix_channel_255(*r, 0, t),
                mix_channel_255(*g, 0, t),
                mix_channel_255(*b, 0, t),
                *a,
            ))
        }
        _ => Some(color.clone()),
    }
}

fn color_saturate(args: &[Value]) -> Option<Value> {
    let color = args.first()?;
    let amount = args.get(1).map(value_to_number).unwrap_or(0.0);
    match color {
        Value::Color(r, g, b, a) => {
            let t = (amount / 100.0).clamp(0.0, 1.0);
            let max = *r.max(g).max(b);
            Some(Value::Color(
                shift_toward_max(*r, max, t),
                shift_toward_max(*g, max, t),
                shift_toward_max(*b, max, t),
                *a,
            ))
        }
        _ => Some(color.clone()),
    }
}

fn color_desaturate(args: &[Value]) -> Option<Value> {
    let color = args.first()?;
    let amount = args.get(1).map(value_to_number).unwrap_or(0.0);
    match color {
        Value::Color(r, g, b, a) => {
            let t = (amount / 100.0).clamp(0.0, 1.0);
            let gray = ((*r as u16 + *g as u16 + *b as u16) / 3) as u8;
            Some(Value::Color(
                mix_channel_255(*r, gray, t),
                mix_channel_255(*g, gray, t),
                mix_channel_255(*b, gray, t),
                *a,
            ))
        }
        _ => Some(color.clone()),
    }
}

fn color_rgb_rgba(args: &[Value], fn_name: &str) -> Option<Value> {
    // Special case 1: when first arg is a var() expression (string starting with "var("),
    // emit fn_name(var(--name), alpha) instead of trying to resolve to a concrete Color.
    if let Some(Value::String(s)) = args.first()
        && s.starts_with("var(")
    {
        // Alpha might also be a var() — preserve its string form
        let alpha_val = args.get(1);
        let alpha_str = match alpha_val {
            Some(Value::String(a)) if a.starts_with("var(") => a.clone(),
            _ => {
                let alpha = alpha_val.map(value_to_number).unwrap_or(1.0);
                if alpha == alpha.trunc() {
                    format!("{}", alpha as i64)
                } else {
                    format!("{}", alpha)
                }
            }
        };
        return Some(Value::String(format!("{}({}, {})", fn_name, s, alpha_str)));
    }
    // Special case 2: rgba($color, $alpha) — 2 args, first is a Color
    if args.len() == 2
        && let Some(Value::Color(r, g, b, _)) = args.first()
    {
        let alpha = args.get(1).map(value_to_number).unwrap_or(1.0);
        // CSS output: rgba(r, g, b, alpha) instead of hex — preserves transparency
        let alpha_str = if alpha == alpha.trunc() {
            format!("{}", alpha as i64)
        } else {
            format!("{}", alpha)
        };
        return Some(Value::String(format!("{}({}, {}, {}, {})", fn_name, *r, *g, *b, alpha_str)));
    }
    // Standard 4-arg form: rgba(r, g, b, a) or rgb(r, g, b)
    let r = args.first().map(|v| value_to_int(v) as u8).unwrap_or(0);
    let g = args.get(1).map(|v| value_to_int(v) as u8).unwrap_or(0);
    let b = args.get(2).map(|v| value_to_int(v) as u8).unwrap_or(0);
            let a = args.get(3).map(value_to_number).unwrap_or(1.0);
    Some(Value::Color(r, g, b, a))
}

/// var(--name) / var(--name, fallback) — CSS custom property reference
/// Returns the CSS var() expression as a raw string so it can be embedded
/// in other CSS functions like rgba(var(--name), alpha).
fn var_function(args: &[Value]) -> Option<Value> {
    let raw = args.first()?.to_string();
    // CSS custom property names (--xxx) must not contain spaces or commas.
    // When the input is assembled from PropSegment parts via List display,
    // it becomes "--, bs-, suffix" (comma-separated after our list format change).
    // Strip both spaces and commas to produce the correct "--bssuffix" form.
    let name = if raw.starts_with("--") {
        raw.replace([' ', ','], "")
    } else {
        raw
    };
    if args.len() >= 2 {
        let fallback = args.get(1)?.to_string();
        Some(Value::String(format!("var({}, {})", name, fallback)))
    } else {
        Some(Value::String(format!("var({})", name)))
    }
}

fn color_transparentize(args: &[Value]) -> Option<Value> {
    let color = args.first()?;
    let amount = args.get(1).map(value_to_number).unwrap_or(0.0);
    match color {
        Value::Color(r, g, b, a) => {
            let a_new = *a * (1.0 - amount / 100.0);
            Some(Value::Color(*r, *g, *b, a_new))
        }
        _ => Some(color.clone()),
    }
}

fn color_opacify(args: &[Value]) -> Option<Value> {
    let color = args.first()?;
    let amount = args.get(1).map(value_to_number).unwrap_or(0.0);
    match color {
        Value::Color(r, g, b, a) => {
            let a_new = (*a + amount / 100.0).min(1.0);
            Some(Value::Color(*r, *g, *b, a_new))
        }
        _ => Some(color.clone()),
    }
}

fn color_invert(args: &[Value]) -> Option<Value> {
    let color = args.first()?;
    match color {
        Value::Color(r, g, b, a) => Some(Value::Color(255 - r, 255 - g, 255 - b, *a)),
        _ => Some(color.clone()),
    }
}

fn color_complement(args: &[Value]) -> Option<Value> {
    // Simple 180° hue rotation approximation
    let color = args.first()?;
    match color {
        Value::Color(r, g, b, a) => {
            let max = *r.max(g).max(b);
            let min = *r.min(g).min(b);
            Some(Value::Color(
                (max + min).saturating_sub(*r),
                (max + min).saturating_sub(*g),
                (max + min).saturating_sub(*b),
                *a,
            ))
        }
        _ => Some(color.clone()),
    }
}

/// shade-color($color, $weight) — mix with black by weight
fn color_shade(args: &[Value]) -> Option<Value> {
    let color = args.first()?.clone();
    let weight = args.get(1).map(value_to_number).unwrap_or(50.0);
    let black = Value::Color(0, 0, 0, 1.0);
    color_mix(&[black, color, Value::Number(weight, Some("%".to_string()))])
}

/// tint-color($color, $weight) — mix with white by weight
fn color_tint(args: &[Value]) -> Option<Value> {
    let color = args.first()?.clone();
    let weight = args.get(1).map(value_to_number).unwrap_or(50.0);
    let white = Value::Color(255, 255, 255, 1.0);
    color_mix(&[white, color, Value::Number(weight, Some("%".to_string()))])
}

/// to-rgb($color) — returns "R, G, B" comma-separated string
fn color_to_rgb(args: &[Value]) -> Option<Value> {
    let color = args.first()?;
    match color {
        Value::Color(r, g, b, _) => Some(Value::String(format!("{}, {}, {}", r, g, b))),
        _ => Some(Value::Null),
    }
}

/// Channel extraction: red/green/blue return 0-255 int, alpha returns 0.0-1.0
fn color_channel(args: &[Value], channel: &str) -> Option<Value> {
    let color = args.first()?;
    match color {
        Value::Color(r, g, b, a) => match channel {
            "red" => Some(Value::Number(*r as f64, None)),
            "green" => Some(Value::Number(*g as f64, None)),
            "blue" => Some(Value::Number(*b as f64, None)),
            "alpha" => Some(Value::Number(*a, None)),
            _ => Some(Value::Null),
        },
        _ => Some(Value::Null),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Type / meta functions
// ─────────────────────────────────────────────────────────────────────────

fn type_of(args: &[Value]) -> Option<Value> {
    let v = args.first()?;
    let s = match v {
        Value::Number(_, _) => "number",
        Value::String(_) => "string",
        Value::Color(_, _, _, _) => "color",
        Value::Calc(_) => "string",
        Value::List(_, _) => "list",
        Value::Map(_) => "map",
        Value::Bool(_) => "bool",
        Value::Null => "null",
    };
    Some(Value::String(s.to_string()))
}

fn unit(args: &[Value]) -> Option<Value> {
    let v = args.first()?;
    match v {
        Value::Number(_, Some(u)) => Some(Value::String(u.clone())),
        Value::Number(_, None) => Some(Value::String(String::new())),
        _ => Some(Value::String("".to_string())),
    }
}

fn inspect(args: &[Value]) -> Option<Value> {
    let v = args.first()?;
    Some(Value::String(v.to_string()))
}

fn not_fn(args: &[Value]) -> Option<Value> {
    let v = args.first()?;
    Some(Value::Bool(!truthy(v)))
}

// ─────────────────────────────────────────────────────────────────────────
// String functions
// ─────────────────────────────────────────────────────────────────────────

fn quote(args: &[Value]) -> Option<Value> {
    let v = args.first()?;
    Some(Value::String(format!("\"{}\"", v)))
}

fn unquote(args: &[Value]) -> Option<Value> {
    // Unquote just returns the value as-is for non-string, strips quotes for strings
    args.first().cloned()
}

fn str_length(args: &[Value]) -> Option<Value> {
    let v = args.first()?;
    Some(Value::Number(v.to_string().chars().count() as f64, None))
}

/// str-index($string, $substring) - 1-based index of first occurrence, or null
fn str_index(args: &[Value]) -> Option<Value> {
    let s = args.first()?.to_string();
    let search = args.get(1)?.to_string();
    if search.is_empty() {
        return Some(Value::Null);
    }
    s.find(&search).map(|pos| Value::Number((pos + 1) as f64, None))
}

/// str-slice($string, $start, $end) - substring from start (1-based) to end (1-based, inclusive)
fn str_slice(args: &[Value]) -> Option<Value> {
    let s = args.first()?.to_string();
    let start = args.get(1).map(value_to_int).unwrap_or(1);
    let end = args.get(2).map(value_to_int).unwrap_or_else(|| s.chars().count());
    if start < 1 || start > end {
        return Some(Value::String(String::new()));
    }
    let chars: Vec<char> = s.chars().collect();
    if start > chars.len() {
        return Some(Value::String(String::new()));
    }
    let actual_end = end.min(chars.len());
    let sliced: String = chars[(start - 1)..actual_end].iter().collect();
    Some(Value::String(sliced))
}

/// str-replace($string, $search, $replacement) — global substring replacement
fn str_replace(args: &[Value]) -> Option<Value> {
    let input = args.first()?.to_string();
    let search = args.get(1)?.to_string();
    let replacement = args.get(2)?.to_string();
    if search.is_empty() {
        return Some(Value::String(input));
    }
    Some(Value::String(input.replace(&search, &replacement)))
}

fn to_upper_case(args: &[Value]) -> Option<Value> {
    let v = args.first()?;
    match v {
        Value::String(s) => Some(Value::String(s.to_uppercase())),
        _ => Some(Value::String(v.to_string().to_uppercase())),
    }
}

fn to_lower_case(args: &[Value]) -> Option<Value> {
    let v = args.first()?;
    match v {
        Value::String(s) => Some(Value::String(s.to_lowercase())),
        _ => Some(Value::String(v.to_string().to_lowercase())),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────

/// Determine if a value is "truthy" in SCSS semantics.
/// false and null are falsy; everything else is truthy.
pub fn truthy(v: &Value) -> bool {
    !matches!(v, Value::Bool(false) | Value::Null)
}

/// Extract a numeric value from a Value.
fn value_to_number(v: &Value) -> f64 {
    match v {
        Value::Number(n, _) => *n,
        Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
        Value::Bool(true) => 1.0,
        Value::Bool(false) => 0.0,
        _ => 0.0,
    }
}

/// Extract an integer value from a Value.
fn value_to_int(v: &Value) -> usize {
    value_to_number(v) as usize
}

/// Convert a Value to a list of items.
fn list_to_items(v: &Value) -> Vec<Value> {
    match v {
        Value::List(items, _) => items.clone(),
        Value::Map(entries) => entries.iter().map(|(k, v)| {
            Value::List(vec![Value::String(k.clone()), v.clone()], ListSeparator::Comma)
        }).collect(),
        Value::Null => Vec::new(),
        other => vec![other.clone()],
    }
}

/// Mix two channels (0-255) by weight 0.0..1.0
fn mix_channel(a: u8, b: u8, w: f64) -> u8 {
    let a_f = a as f64;
    let b_f = b as f64;
    let result = a_f + (b_f - a_f) * w;
    result.round() as u8
}

/// Mix channel toward a target value by weight
fn mix_channel_255(val: u8, target: u8, w: f64) -> u8 {
    mix_channel(val, target, w)
}

/// Mix two alpha values (0.0-1.0) by weight 0.0..1.0
fn mix_alpha(a: f64, b: f64, w: f64) -> f64 {
    a + (b - a) * w
}

/// Shift channel toward max value (for saturate)
fn shift_toward_max(val: u8, max: u8, t: f64) -> u8 {
    mix_channel_255(val, max, t)
}
