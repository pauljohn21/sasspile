//! Selector 内建函数。
//!
//! 包含 selector-append/nest/is-super/parse/simple-selectors/unify/extend/replace。
//! 返回值统一为 Selector Format（list of lists of strings）。

use crate::css::selector_format;
use crate::css::selector_ops;
use crate::css::selector_parser::parse_selector;
use crate::error::{Result, SassError};
use crate::parse::ast::*;
use imbl::HashMap;

/// 返回每个 selector 函数的参数名列表（按位置顺序）。
fn selector_param_names(name: &str) -> &'static [&'static str] {
    match name {
        "selector-parse" => &["selector"],
        "selector-append" => &[],
        "selector-nest" => &[],
        "selector-is-superselector" | "selector-is-super" => &["super", "sub"],
        "selector-simple-selectors" => &["selector"],
        "selector-unify" => &["selector1", "selector2"],
        "selector-extend" => &["selector", "extendee", "extender"],
        "selector-replace" => &["selector", "original", "replacement"],
        _ => &[],
    }
}

/// 合并位置参数和命名参数。
fn merge_selector_args(
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    name: &str,
) -> Vec<Value> {
    let param_names = selector_param_names(name);
    match param_names.is_empty() {
        true => return pos_args.to_vec(),
        false => {}
    }
    let mut result: Vec<Value> = param_names
        .iter()
        .enumerate()
        .filter_map(|(i, pname)| {
            pos_args
                .get(i)
                .cloned()
                .or_else(|| kw_args.get(*pname).cloned())
                .or_else(|| kw_args.get(&format!("${pname}")).cloned())
        })
        .collect();
    match pos_args.len() > param_names.len() {
        true => result.extend_from_slice(&pos_args[param_names.len()..]),
        false => {}
    }
    result
}

pub fn call(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
) -> Result<Option<Value>> {
    let args = merge_selector_args(pos_args, kw_args, name);
    let args = args.as_slice();
    match name {
        "selector-append" => call_append(args),
        "selector-nest" => call_nest(args),
        "selector-is-superselector" | "selector-is-super" => call_is_super(args),
        "selector-parse" => call_parse(args),
        "selector-simple-selectors" => call_simple_selectors(args),
        "selector-unify" => call_unify(args),
        "selector-extend" => call_extend(args),
        "selector-replace" => call_replace(args),
        _ => Ok(None),
    }
}

// ─── 辅助函数 ───────────────────────────────────────────────────

/// 将 Selector Format（Vec<Vec<String>>）序列化为 CSS 字符串。
fn format_to_string(fmt: &[Vec<String>]) -> String {
    fmt.iter()
        .map(|complex| complex.join(" "))
        .collect::<Vec<_>>()
        .join(", ")
}

// ─── selector-append ─────────────────────────────────────────────

fn call_append(args: &[Value]) -> Result<Option<Value>> {
    if args.is_empty() {
        return Err(SassError::Eval(
            "$selectors: At least one selector must be passed.".into(),
        ));
    }
    let formats: Vec<Vec<Vec<String>>> = args
        .iter()
        .map(|a| selector_format::value_to_selector_format(a))
        .collect::<Result<Vec<_>>>()?;
    let init = formats[0].clone();
    let result = formats[1..]
        .iter()
        .try_fold(init, |acc, next| append_two(&acc, next))?;
    Ok(Some(selector_format::selector_format_to_value(result)))
}

/// 笛卡尔积合并两个 complex 列表。
fn append_two(a: &[Vec<String>], b: &[Vec<String>]) -> Result<Vec<Vec<String>>> {
    let mut results: Vec<Vec<String>> = Vec::new();
    for ca in a {
        for cb in b {
            match append_complex(ca, cb) {
                Ok(merged) => results.extend(merged),
                Err(e) => return Err(e),
            }
        }
    }
    Ok(results)
}

/// 合并两个 complex selectors——末尾化合物字符串拼接。
/// 返回 Ok(merged) 或 Err(error_message)。
fn append_complex(a: &[String], b: &[String]) -> Result<Vec<Vec<String>>> {
    let a_first = a.first().cloned().unwrap_or_default();
    let b_first = b.first().cloned().unwrap_or_default();
    let a_last = a.last().cloned().unwrap_or_default();

    // A 不能以组合符开头或结尾 → error
    if matches!(a_first.as_str(), ">" | "+" | "~") || matches!(a_last.as_str(), ">" | "+" | "~") {
        return Err(SassError::Eval(format!("Can't append {b_first} to {a_first}")));
    }
    // 检查 B 是否只有 combinator → error
    if b.len() == 1 && matches!(b_first.as_str(), ">" | "+" | "~") {
        return Err(SassError::Eval(format!("Can't append {b_first} to {a_last}")));
    }
    // 检查 B 是否只有 namespace → error
    if b.len() == 1 && b_first.starts_with('|') {
        return Err(SassError::Eval(format!("Can't append {b_first} to {a_last}")));
    }
    // 检查 universal 开头 → error
    if b_first == "*" {
        return Err(SassError::Eval(format!("Can't append * to {a_last}")));
    }
    // 检查 parent reference — append 中不允许裸 &
    if b.len() == 1 && b[0] == "&" {
        return Err(SassError::Eval("Parent selectors aren't allowed here.".into()));
    }
    // 处理前导 combinator：直接拼接整个 b 到 a
    if matches!(b_first.as_str(), ">" | "+" | "~") {
        let mut result: Vec<String> = a.iter().cloned().collect();
        result.extend(b.iter().cloned());
        return Ok(vec![result]);
    }
    // 化合物选择器拼接：直接连接，比如.a + .b → .a.b，a + b → ab
    let merged = format!("{a_last}{b_first}");
    let mut result = a[..a.len() - 1].to_vec();
    result.push(merged);
    result.extend_from_slice(&b[1..]);
    Ok(vec![result])
}

// ─── selector-nest ───────────────────────────────────────────────

fn call_nest(args: &[Value]) -> Result<Option<Value>> {
    if args.is_empty() {
        return Ok(Some(Value::List(Vec::new(), Separator::Space, false)));
    }
    if args.len() == 1 {
        let fmt = selector_format::value_to_selector_format(&args[0])?;
        return Ok(Some(selector_format::selector_format_to_value(fmt)));
    }
    // 解析所有参数
    let all_formats: Vec<Vec<Vec<String>>> = args
        .iter()
        .map(|a| selector_format::value_to_selector_format(a))
        .collect::<Result<Vec<_>>>()?;
    let last = all_formats.last().expect("len > 1").clone();
    let leading = &all_formats[..all_formats.len() - 1];
    // 构建父级列表
    let parents = build_parents(leading)?;
    // 检查最后一个 arg 是否包含 &
    let has_amp = last
        .iter()
        .any(|complex| complex.iter().any(|c| c.contains('&')));
    let result: Vec<Vec<String>> = if has_amp {
        nest_with_amp(&parents, &last)?
    } else {
        nest_no_amp(&parents, &last)
    };
    Ok(Some(selector_format::selector_format_to_value(result)))
}

/// 将所有前置参数笛卡尔积后代连接为父级 complex 列表。
fn build_parents(leading: &[Vec<Vec<String>>]) -> Result<Vec<Vec<String>>> {
    match leading {
        [] => Ok(vec![vec![]]),
        [single] => Ok(single.clone()),
        [first, rest @ ..] => {
            let init = first.clone();
            rest.iter().try_fold(init, |acc, next| {
                cartesian_descendant(&acc, next)
            })
        }
    }
}

/// 笛卡尔积后代连接两个 complex 列表。
fn cartesian_descendant(a: &[Vec<String>], b: &[Vec<String>]) -> Result<Vec<Vec<String>>> {
    let results: Vec<Vec<String>> = a
        .iter()
        .flat_map(|ca| {
            b.iter().map(move |cb| {
                match (ca.is_empty(), cb.is_empty()) {
                    (true, _) => cb.clone(),
                    (_, true) => ca.clone(),
                    _ => {
                        let complex = ca.iter().chain(cb.iter()).cloned().collect::<Vec<_>>();
                        complex
                    }
                }
            })
        })
        .collect();
    Ok(results)
}

/// 无 & 的 nest：父级 × 内层，后代连接。
fn nest_no_amp(parents: &[Vec<String>], last: &[Vec<String>]) -> Vec<Vec<String>> {
    parents
        .iter()
        .flat_map(|p| last.iter().map(move |c| descendant_join(p, c)))
        .collect()
}

/// 后代连接两个 complex：A + descendant + B。
fn descendant_join(a: &[String], b: &[String]) -> Vec<String> {
    match (a.is_empty(), b.is_empty()) {
        (true, _) => b.to_vec(),
        (_, true) => a.to_vec(),
        _ => a.iter().chain(b.iter()).cloned().collect(),
    }
}

/// 带 & 的 nest 处理。
fn nest_with_amp(parents: &[Vec<String>], last: &[Vec<String>]) -> Result<Vec<Vec<String>>> {
    let max_k = last
        .iter()
        .map(|complex| complex.iter().filter(|c| c.contains('&')).count())
        .max()
        .unwrap_or(0);
    let mut results: Vec<Vec<String>> = Vec::new();
    if max_k >= 2 {
        // 含多个 &：按 complex 顺序遍历，k>=2 的使用全局笛卡尔
        for complex in last {
            let k = complex.iter().filter(|c| c.contains('&')).count();
            if k >= 2 {
                results.extend(cartesian_replace(parents, complex, k));
            } else {
                for parent in parents {
                    results.push(replace_amp_in_complex(complex, parent));
                }
            }
        }
    } else {
        // k <= 1：按 parent × complex 遍历
        for parent in parents {
            for complex in last {
                results.push(replace_amp_in_complex(complex, parent));
            }
        }
    }
    Ok(results)
}

/// 全局笛卡尔替换多个 &。
fn cartesian_replace(parents: &[Vec<String>], complex: &[String], k: usize) -> Vec<Vec<String>> {
    let mut all_combos: Vec<Vec<usize>> = vec![vec![]];
    for _ in 0..k {
        all_combos = all_combos
            .iter()
            .flat_map(|prefix| {
                (0..parents.len()).map(move |i| {
                    let mut new = prefix.clone();
                    new.push(i);
                    new
                })
            })
            .collect();
    }
    all_combos
        .iter()
        .map(|indices| {
            let result: Vec<String> = complex
                .iter()
                .map(|compound| {
                    let mut replaced = compound.clone();
                    for &idx in indices {
                        let parent_str = parent_display(&parents[idx]);
                        if replaced.contains('&') {
                            replaced = replaced.replacen('&', &parent_str, 1);
                        }
                    }
                    replaced
                })
                .collect();
            result
        })
        .collect()
}

/// 替换 complex 中的 & 为父级字符串。
fn replace_amp_in_complex(complex: &[String], parent: &[String]) -> Vec<String> {
    if complex.iter().any(|c| c.contains('&')) {
        let parent_str = parent_display(parent);
        complex.iter().map(|c| c.replace('&', &parent_str)).collect()
    } else {
        // 无 & 的 complex 做后代连接
        descendant_join(parent, complex)
    }
}

/// 获取父级的显示字符串。
fn parent_display(parent: &[String]) -> String {
    parent.join(" ")
}

// ─── selector-is-superselector ──────────────────────────────────

fn call_is_super(args: &[Value]) -> Result<Option<Value>> {
    match args {
        [Value::String(a, _), Value::String(b, _)] => {
            let super_sel = parse_selector(a);
            let sub_sel = parse_selector(b);
            Ok(Some(Value::Bool(selector_ops::is_superselector(
                &super_sel, &sub_sel,
            ))))
        }
        _ => Ok(Some(Value::Bool(false))),
    }
}

// ─── selector-parse ─────────────────────────────────────────────

fn call_parse(args: &[Value]) -> Result<Option<Value>> {
    match args.len() {
        0 => return Err(SassError::Eval("Missing argument $selector.".into())),
        1 => {}
        n => {
            return Err(SassError::Eval(format!(
                "Only 1 argument allowed, but {n} {} passed.",
                match n == 1 {
                    true => "was",
                    false => "were",
                }
            )))
        }
    }
    let fmt = selector_format::value_to_selector_format(&args[0])?;
    Ok(Some(selector_format::selector_format_to_value(fmt)))
}

// ─── selector-simple-selectors ──────────────────────────────────

fn call_simple_selectors(args: &[Value]) -> Result<Option<Value>> {
    match args.len() {
        0 => return Err(SassError::Eval("Missing argument $selector.".into())),
        1 => {}
        n => {
            return Err(SassError::Eval(format!(
                "Only 1 argument allowed, but {n} {} passed.",
                match n == 1 {
                    true => "was",
                    false => "were",
                }
            )))
        }
    }
    let fmt = match &args[0] {
        Value::String(s, _) => selector_format::string_to_selector_format(s),
        other => selector_format::value_to_selector_format(other)?,
    };
    match fmt.first() {
        Some(complex) => {
            let simples: Vec<Value> = complex
                .iter()
                .flat_map(|c| split_simple_selectors(c.as_str()))
                .map(|s| Value::String(s, false))
                .collect();
            Ok(Some(Value::List(simples, Separator::Comma, false)))
        }
        None => Ok(Some(Value::List(Vec::new(), Separator::Comma, false))),
    }
}

/// 按 simple selector 分界点拆分 compound 字符串。
fn split_simple_selectors(compound: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut current = String::new();
    for c in compound.chars() {
        match c {
            '.' | '#' | ':' | '[' => {
                if !current.is_empty() {
                    result.push(std::mem::take(&mut current));
                }
                current.push(c);
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

// ─── selector-unify ─────────────────────────────────────────────

fn call_unify(args: &[Value]) -> Result<Option<Value>> {
    let params = selector_param_names("selector-unify");
    match args.len() < params.len() {
        true => {
            let missing = params[args.len()];
            return Err(SassError::Eval(format!("Missing argument ${missing}.")));
        }
        false => {}
    }
    match args.len() > params.len() {
        true => {
            return Err(SassError::Eval(format!(
                "Only {} arguments allowed, but {} {} passed.",
                params.len(),
                args.len(),
                match args.len() == 1 {
                    true => "was",
                    false => "were",
                }
            )))
        }
        false => {}
    }
    let a_fmt = selector_format::value_to_selector_format(&args[0])?;
    let b_fmt = selector_format::value_to_selector_format(&args[1])?;
    // 用现有 AST unify 实现
    let sel_a = parse_selector(&format_to_string(&a_fmt));
    let sel_b = parse_selector(&format_to_string(&b_fmt));
    match selector_ops::unify(&sel_a, &sel_b) {
        Some(unified) => Ok(Some(Value::String(unified.to_string(), false))),
        None => Ok(Some(Value::Null)),
    }
}

// ─── selector-extend ────────────────────────────────────────────

fn call_extend(args: &[Value]) -> Result<Option<Value>> {
    let params = selector_param_names("selector-extend");
    match args.len() < params.len() {
        true => {
            return Err(SassError::Eval(format!(
                "Missing argument ${}.",
                params[args.len()]
            )))
        }
        false => {}
    }
    match args.len() > params.len() {
        true => {
            return Err(SassError::Eval(format!(
                "Only {} arguments allowed, but {} {} passed.",
                params.len(),
                args.len(),
                match args.len() == 1 {
                    true => "was",
                    false => "were",
                }
            )))
        }
        false => {}
    }
    let sel_fmt = selector_format::value_to_selector_format(&args[0])?;
    let ext_fmt = selector_format::value_to_selector_format(&args[1])?;
    let new_fmt = selector_format::value_to_selector_format(&args[2])?;
    let sel = parse_selector(&format_to_string(&sel_fmt));
    let ext = parse_selector(&format_to_string(&ext_fmt));
    let new = parse_selector(&format_to_string(&new_fmt));
    let result = selector_ops::extend_selector(&sel, &ext, &new);
    Ok(Some(Value::String(result.to_string(), false)))
}

// ─── selector-replace ───────────────────────────────────────────

fn call_replace(args: &[Value]) -> Result<Option<Value>> {
    let params = selector_param_names("selector-replace");
    match args.len() < params.len() {
        true => {
            return Err(SassError::Eval(format!(
                "Missing argument ${}.",
                params[args.len()]
            )))
        }
        false => {}
    }
    match args.len() > params.len() {
        true => {
            return Err(SassError::Eval(format!(
                "Only {} arguments allowed, but {} {} passed.",
                params.len(),
                args.len(),
                match args.len() == 1 {
                    true => "was",
                    false => "were",
                }
            )))
        }
        false => {}
    }
    let sel_fmt = selector_format::value_to_selector_format(&args[0])?;
    let orig_fmt = selector_format::value_to_selector_format(&args[1])?;
    let repl_fmt = selector_format::value_to_selector_format(&args[2])?;
    let sel = parse_selector(&format_to_string(&sel_fmt));
    let orig = parse_selector(&format_to_string(&orig_fmt));
    let repl = parse_selector(&format_to_string(&repl_fmt));
    let result = selector_ops::replace_selector(&sel, &orig, &repl);
    Ok(Some(Value::String(result.to_string(), false)))
}
