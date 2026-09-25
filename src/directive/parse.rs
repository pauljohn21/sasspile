//! 解析辅助 (纯函数, 无副作用)
//!
//! 包含: mixin/for/each 签名解析, 变量定义解析, include 参数解析, mixin body 展开,
//!       变量替换 + 内置函数求值 (委托 src/eval/)

use crate::eval::eval_all_calls;

use super::state::{CompileState, MixinDef};

// ─── 签名解析 ─────────────────────────────────────────────────────────

pub fn parse_mixin_sig(s: &str) -> Option<(String, Vec<(String, Option<String>)>)> {
    let s = s.trim();
    // 无参 mixin: "foo {" / "foo" / "foo { ... }"
    if !s.contains('(') {
        let name = s.split(|c: char| c == '{' || c == ' ' || c == ';').next()?.trim().to_string();
        if name.is_empty() { return None; }
        return Some((name, vec![]));
    }
    // 有参 mixin: "name($a, $b: val)"
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

pub fn try_parse_var_def(line: &str) -> Option<(String, String, bool)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('$') {
        return None;
    }
    let after_dollar = &trimmed[1..];
    let (name, value_part) = after_dollar.split_once(':')?;
    let name = format!("${}", name.trim());
    let is_default = value_part.contains("!default");
    let value = value_part
        .trim()
        .trim_end_matches(';')
        .trim()
        .trim_end_matches("!default")
        .trim()
        .to_string();
    if value.is_empty() {
        return None;
    }
    Some((name, value, is_default))
}

pub fn substitute_vars(state: &CompileState, line: &str) -> String {
    let mut result = line.to_string();

    // 第一轮: 替换所有 #{$var} 插值 (选择器 + 值通用)
    // 模式: #{...} → 提取内部 $var 名 → 查表 → 替换
    while let Some(start) = result.find("#{") {
        if let Some(end) = result[start..].find('}').map(|p| start + p) {
            let inner = &result[start + 2..end]; // 去掉 #{ 和 }
            // inner 可能是 $var 或表达式
            if let Some(var_name) = inner.strip_prefix('$') {
                let full_name = format!("${var_name}");
                if let Some(value) = state.scope.variables.get(&full_name) {
                    result.replace_range(start..=end, value);
                } else {
                    break; // 未定义变量, 停止替换
                }
            } else {
                break; // 非变量插值, 停止
            }
        } else {
            break; // 未找到闭合 }, 停止
        }
    }

    // 第二轮: 替换裸 $var (值中残留)
    for (name, value) in &state.scope.variables {
        result = result.replace(name, value);
    }

    // 求值内置函数调用 (rgb/hsl/lighten/darken 等)
    result = eval_all_calls(&result);

    // 求值用户自定义函数 (@function / @return)
    eval_user_functions(state, &result)
}

/// 扫描行中所有用户自定义函数调用并替换为返回值
fn eval_user_functions(state: &CompileState, input: &str) -> String {
    let mut result = input.to_string();
    while let Some((start, end)) = find_function_call(&result) {
        let call = &result[start..=end];
        if let Some(func_name) = extract_func_name_from_call(call) {
            if let Some(def) = state.scope.functions.get(func_name) {
                let args = extract_args_from_call(call);
                let replacement = apply_function(def, &args);
                result.replace_range(start..=end, &replacement);
                continue;
            }
        }
        break;
    }
    result
}

/// 查找最右匹配的函数调用 (同 find_last_call 逻辑)
fn find_function_call(input: &str) -> Option<(usize, usize)> {
    let close = input.rfind(')')?;
    let open = input[..close].rfind('(')?;
    // 函数名必须由字母/数字/连字符/下划线组成，且不在字符串内
    let before = &input[..open];
    let name_end = before.rfind(|c: char| c.is_alphanumeric() || c == '-' || c == '_')?;
    let name_start = before[..name_end].rfind(|c: char| !c.is_alphanumeric() && c != '-' && c != '_' && c != '.' && c != '$')
        .map(|p| p + 1)
        .unwrap_or(0);
    Some((name_start, close))
}

/// 从函数调用中提取函数名
fn extract_func_name_from_call(call: &str) -> Option<&str> {
    let open = call.find('(')?;
    let before = &call[..open];
    let end = before.rfind(|c: char| c.is_alphanumeric() || c == '-' || c == '_')?;
    let start = before[..end].rfind(|c: char| !c.is_alphanumeric() && c != '-' && c != '_' && c != '.' && c != '$')
        .map(|p| p + 1)
        .unwrap_or(0);
    Some(&before[start..=end])
}

/// 从函数调用中提取参数列表
fn extract_args_from_call(call: &str) -> Vec<String> {
    let open = match call.find('(') {
        Some(p) => p,
        None => return vec![],
    };
    let close = match call.rfind(')') {
        Some(p) => p,
        None => return vec![],
    };
    if close <= open {
        return vec![];
    }
    let inner = &call[open + 1..close];
    if inner.trim().is_empty() {
        return vec![];
    }
    inner.split(',').map(|s| s.trim().to_string()).collect()
}

/// 应用用户自定义函数: 参数替换到返回值
fn apply_function(def: &super::state::FunctionDef, args: &[String]) -> String {
    let mut result = def.return_value.clone();
    for (i, (param_name, default_value)) in def.params.iter().enumerate() {
        let arg_value = args.get(i)
            .map(|s| s.as_str())
            .or_else(|| default_value.as_deref())
            .unwrap_or("");
        result = result.replace(param_name, arg_value);
    }
    result
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
