//! 解析辅助 (纯函数, 无副作用)
//!
//! 包含: mixin/for/each 签名解析, 变量定义解析, include 参数解析, mixin body 展开

use std::borrow::Cow;

use super::state::{CompileState, MixinDef};

// ─── 签名解析 ─────────────────────────────────────────────────────────

pub fn parse_mixin_sig(s: &str) -> Option<(String, Vec<(String, Option<String>)>)> {
    let (name, rest) = s.split_once('(')?;
    let name = name.trim().to_string();
    let params_end = rest.find(')')?;
    let params_str = &rest[..params_end].trim();
    let params = if params_str.is_empty() {
        vec![]
    } else {
        params_str.split(',')
            .map(|param| {
                let p = param.trim();
                p.split_once(':')
                    .map(|(n, d)| (n.trim().to_string(), Some(d.trim().to_string())))
                    .unwrap_or_else(|| (p.to_string(), None))
            })
            .collect()
    };
    Some((name, params))
}

pub fn parse_for_sig(s: &str) -> Option<(String, Vec<String>)> {
    let s = s.trim();
    let (_, rest) = s.split_once('$')?;
    let rest = rest.trim();
    let (var_name, rest) = rest.split_once(' ')?;
    let var_name = format!("${}", var_name.trim());
    let rest = rest.trim().strip_prefix("from")?.trim();
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() < 3 { return None; }

    let from: i64 = parts[0].trim_end_matches(|c: char| c == '{' || c == '}' || c == ';' || c == ')').parse().ok()?;
    let to: i64 = parts[2].trim_end_matches(|c: char| c == '{' || c == '}' || c == ';' || c == ')').parse().ok()?;

    let inclusive = parts[1] == "through";
    Some((var_name, generate_range(from, to, inclusive)))
}

pub fn generate_range(from: i64, to: i64, inclusive: bool) -> Vec<String> {
    if from <= to {
        let end = if inclusive { to + 1 } else { to };
        (from..end).map(|i| i.to_string()).collect()
    } else {
        let end = if inclusive { to - 1 } else { to };
        (end + 1..=from).rev().map(|i| i.to_string()).collect()
    }
}

pub fn parse_each_sig(s: &str) -> Option<(String, Vec<String>)> {
    let s = s.trim();
    let (_, rest) = s.split_once('$')?;
    let rest = rest.trim();
    let (var_name, rest) = rest.split_once(' ')?;
    let var_name = format!("${}", var_name.trim());
    let rest = rest.trim().strip_prefix("in")?.trim();
    let rest = rest
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(|c: char| c == ')' || c == '{' || c == '}')
        .trim();
    let items: Vec<String> = rest
        .split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect();
    if items.is_empty() {
        return None;
    }
    Some((var_name, items))
}

pub fn parse_include_sig(s: &str) -> (String, Vec<String>) {
    let s = s.trim().trim_end_matches(';').trim();
    match s.split_once('(') {
        Some((name, rest)) => {
            let name = name.trim().to_string();
            let rest = rest.trim().trim_end_matches(|c: char| c == ')' || c == ';' || c == '}').trim();
            let args = if rest.is_empty() {
                vec![]
            } else {
                rest.split(',').map(|a| a.trim().to_string()).collect()
            };
            (name, args)
        }
        None => (s.trim().to_string(), vec![]),
    }
}

// ─── 变量定义 + 替换 ──────────────────────────────────────────────────

pub fn try_parse_var_def(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('$') {
        return None;
    }
    let after_dollar = &trimmed[1..];
    let (name, value_part) = after_dollar.split_once(':')?;
    let name = format!("${}", name.trim());
    let value = value_part
        .trim()
        .trim_end_matches(';')
        .trim()
        .to_string();
    if value.is_empty() {
        return None;
    }
    Some((name, value))
}

pub fn substitute_vars(state: &CompileState, line: &str) -> String {
    let mut result = line.to_string();
    for (name, value) in &state.scope.variables {
        // 先替换 #{$var} 形式,再替换 $var,避免顺序导致模式破坏
        let interp = format!("#{{{name}}}");
        result = result.replace(&interp, value);
        result = result.replace(name, value);
    }
    eval_builtins(&result).into_owned()
}

/// 展开内置 Sass 函数调用 (颜色 → hex)
/// 返回 Cow: 无替换时零分配(借用输入), 有替换时一次分配新 String
pub fn eval_builtins(input: &str) -> Cow<'_, str> {
    match eval_color_calls(input) {
        Some(s) => Cow::Owned(s),
        None => Cow::Borrowed(input),
    }
}

/// 扫描字符串中所有 rgb(...)/hsl(...) 并替换为 hex; 若无匹配返回 None
fn eval_color_calls(input: &str) -> Option<String> {
    let rgb_positions: Vec<(usize, usize)> = find_color_call_spans(input, "rgb(");
    let hsl_positions: Vec<(usize, usize)> = find_color_call_spans(input, "hsl(");
    if rgb_positions.is_empty() && hsl_positions.is_empty() {
        return None;
    }

    // 合并所有 spans 并按起始位置排序
    let mut all_spans: Vec<(usize, usize, ColorFn)> = rgb_positions
        .into_iter()
        .map(|(s, e)| (s, e, ColorFn::Rgb))
        .chain(hsl_positions.into_iter().map(|(s, e)| (s, e, ColorFn::Hsl)))
        .collect();
    all_spans.sort_by_key(|(s, _, _)| *s);

    let mut out = String::with_capacity(input.len());
    let mut last_end = 0;
    for (start, end, kind) in &all_spans {
        out.push_str(&input[last_end..*start]);
        let inner = &input[start + 4..*end]; // 跳过 "rgb(" 或 "hsl("
        let replacement = match kind {
            ColorFn::Rgb => eval_rgb_inner(inner),
            ColorFn::Hsl => eval_hsl_inner(inner),
        };
        match replacement {
            Some(hex) => out.push_str(&hex),
            None => out.push_str(&input[*start..=*end]), // 解析失败保留原样
        }
        last_end = end + 1;
    }
    out.push_str(&input[last_end..]);
    Some(out)
}

#[derive(Clone, Copy)]
enum ColorFn { Rgb, Hsl }

fn find_color_call_spans(input: &str, prefix: &str) -> Vec<(usize, usize)> {
    input
        .match_indices(prefix)
        .filter_map(|(start, _)| {
            input[start..].find(')').map(|rel| (start, start + rel))
        })
        .collect()
}

fn eval_rgb_inner(inner: &str) -> Option<String> {
    let mut parts = inner.split(',').map(str::trim);
    let r = parts.next()?.parse::<u64>().ok()?.min(255) as u8;
    let g = parts.next()?.parse::<u64>().ok()?.min(255) as u8;
    let b = parts.next()?.parse::<u64>().ok()?.min(255) as u8;
    Some(format!("#{r:02x}{g:02x}{b:02x}"))
}

fn eval_hsl_inner(inner: &str) -> Option<String> {
    let mut parts = inner.split(',').map(str::trim);
    let h = parts.next()?.parse::<f64>().ok()?;
    let s = strip_pct(parts.next()?)? / 100.0;
    let l = strip_pct(parts.next()?)? / 100.0;
    let (r, g, b) = hsl_to_rgb(h, s, l);
    Some(format!("#{r:02x}{g:02x}{b:02x}"))
}

fn strip_pct(s: &str) -> Option<f64> {
    s.trim().trim_end_matches('%').parse::<f64>().ok()
}

/// HSL → RGB 转换, h ∈ [0,360), s ∈ [0,1], l ∈ [0,1]
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

// ─── mixin 展开 ────────────────────────────────────────────────────────

/// 展开 mixin: 参数替换 + 默认值应用 (纯函数, fold + Option 组合子)
pub fn expand_mixin(mixin_def: &MixinDef, args: &[String]) -> Vec<String> {
    let defaults: Vec<Option<&str>> = mixin_def.params.iter()
        .map(|(_, d)| d.as_deref()).collect();

    mixin_def.body.iter()
        .map(|body_token| {
            mixin_def.params.iter().enumerate().fold(
                body_token.clone(),
                |acc, (i, (p_name, _))| {
                    // 取参数值: args 优先, 否则用 default
                    args.get(i)
                        .map(|s| s.as_str())
                        .or_else(|| defaults.get(i).copied().flatten())
                        .map(|r| acc.replace(p_name, r))  // 有值则替换
                        .unwrap_or(acc)                   // 无值则保持原样
                },
            )
        })
        .collect()
}
