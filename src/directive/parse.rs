//! 解析辅助 (纯函数, 无副作用)
//!
//! 包含: mixin/for/each 签名解析, 变量定义解析, include 参数解析, mixin body 展开,
//!       变量替换 + 内置函数求值 (委托 src/eval/)

use crate::eval::eval_all_calls;

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
    // 求值内置函数调用 (rgb/hsl/lighten/darken 等)
    eval_all_calls(&result)
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
