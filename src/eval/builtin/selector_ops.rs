//! `selector-*` 操作函数（is-super / parse / simple-selectors / unify / extend / replace）。

use crate::css::selector_ast::Selector;
use crate::css::selector_format;
use crate::css::selector_ops;
use crate::css::selector_parser::parse_selector;
use crate::error::{Result, SassError};
use crate::parse::ast::*;

/// 将 Selector Format（Vec<Vec<String>>）序列化为 CSS 字符串。
fn format_to_string(fmt: &[Vec<String>]) -> String {
    fmt.iter()
        .map(|complex| complex.join(" "))
        .collect::<Vec<_>>()
        .join(", ")
}

// ─── selector-is-superselector ──────────────────────────────────

/// `selector-is-superselector($super, $sub)` — 检查是否为超集选择器。
pub fn call_is_super(args: &[Value]) -> Result<Option<Value>> {
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

/// `selector-parse($selector)` — 解析选择器并返回规范化格式。
pub fn call_parse(args: &[Value]) -> Result<Option<Value>> {
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

/// `selector-simple-selectors($selector)` — 分解复合选择器为简单选择器列表。
pub fn call_simple_selectors(args: &[Value]) -> Result<Option<Value>> {
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
///
/// 使用 fold 累积，避免显式 for + push。
fn split_simple_selectors(compound: &str) -> Vec<String> {
    let chars = compound.chars();
    let (mut result, last) = chars.fold(
        (Vec::new(), String::new()),
        |(mut acc, current), c| match c {
            '.' | '#' | ':' | '[' if !current.is_empty() => {
                let mut new_current = String::new();
                new_current.push(c);
                acc.push(current);
                (acc, new_current)
            }
            _ => {
                let mut new_current = current;
                new_current.push(c);
                (acc, new_current)
            }
        },
    );
    if !last.is_empty() {
        result.push(last);
    }
    result
}

// ─── selector-unify ─────────────────────────────────────────────

/// `selector-unify($selector1, $selector2)` — 合并两个选择器。
pub fn call_unify(args: &[Value]) -> Result<Option<Value>> {
    match args.len() < 2 {
        true => {
            let missing = match args.len() {
                0 => "selector1",
                _ => "selector2",
            };
            return Err(SassError::Eval(format!("Missing argument ${missing}.")));
        }
        false => {}
    }
    match args.len() > 2 {
        true => {
            return Err(SassError::Eval(format!(
                "Only 2 arguments allowed, but {} were passed.",
                args.len()
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

/// 校验参数数量——要求恰好 N 个参数。
fn validate_exact_args(args: &[Value], n: usize, name: &str) -> Result<()> {
    match args.len() {
        len if len < n => Err(SassError::Eval(format!(
            "{name} requires {n} arguments, but {} were provided.",
            args.len()
        ))),
        len if len > n => Err(SassError::Eval(format!(
            "Only {n} arguments allowed, but {len} were passed."
        ))),
        _ => Ok(()),
    }
}

/// 解析 extend 参数为 complex selector 列表（支持单个选择器或逗号分隔的列表）
fn parse_extend_arg(arg: &Value) -> Result<Vec<Vec<String>>> {
    match arg {
        Value::String(s, _) => Ok(selector_format::string_to_selector_format_validated(s)?),
        _ => Ok(selector_format::value_to_selector_format(arg)?),
    }
}

/// 将 Selector AST 转换为 Vec<Vec<String>> (selector format)。
///
/// 组合器（>、+、~）会作为单独的字符串元素保留在列表中。
/// 例如：`.a > .b` → `[[".a", ">", ".b"]]`
/// 尾随组合器也会保留：`.a +` → `[[".a", "+"]]`
fn fmt_to_vec_vec(selector: &Selector) -> Vec<Vec<String>> {
    use crate::css::selector_ast::Combinator;

    selector
        .0
        .iter()
        .map(|complex| {
            complex
                .compounds
                .iter()
                .flat_map(|(comb, compound)| {
                    let comb_token = match comb {
                        Some(Combinator::Child) => Some(">".to_string()),
                        Some(Combinator::Adjacent) => Some("+".to_string()),
                        Some(Combinator::Sibling) => Some("~".to_string()),
                        Some(Combinator::Descendant) | None => None,
                    };
                    let compound_token = (!compound.0.is_empty()).then(|| compound.to_string());
                    comb_token.into_iter().chain(compound_token)
                })
                .collect::<Vec<String>>()
        })
        .collect()
}

/// `selector-extend($selector, $extendee, $extender)` — 选择器扩展。
pub fn call_extend(args: &[Value]) -> Result<Option<Value>> {
    validate_exact_args(args, 3, "selector-extend")?;
    // 解析为 Selector AST
    let sel_arg = parse_extend_arg(&args[0])?;
    let ext_arg = parse_extend_arg(&args[1])?;
    let extender_arg = parse_extend_arg(&args[2])?;
    let sel = parse_selector(&format_to_string(&sel_arg));
    let extendee = parse_selector(&format_to_string(&ext_arg));
    let extender = parse_selector(&format_to_string(&extender_arg));

    // Sass 规范：extendee 不能是复杂选择器（含组合器）
    if extendee.0.iter().any(|c| c.compounds.len() > 1) {
        return Err(SassError::Eval(format!(
            "Can't extend complex selector {extendee}."
        )));
    }

    // Sass 规范：任何参数都不能包含父选择器 &
    use crate::css::selector_ast::SimpleSelector;
    let has_parent_ref = |s: &Selector| s.0.iter().any(|c| {
        c.compounds.iter().any(|(_, comp)| comp.0.iter().any(|s| matches!(s, SimpleSelector::ParentReference)))
    });
    if has_parent_ref(&sel) {
        return Err(SassError::Eval("$selector: Parent selectors aren't allowed here.".to_string()));
    }
    if has_parent_ref(&extendee) {
        return Err(SassError::Eval("$extendee: Parent selectors aren't allowed here.".to_string()));
    }
    if has_parent_ref(&extender) {
        return Err(SassError::Eval("$extender: Parent selectors aren't allowed here.".to_string()));
    }

    let uses_format = matches!(args[0], Value::List(_, _, _))
        || matches!(args[1], Value::List(_, _, _))
        || matches!(args[2], Value::List(_, _, _));

    let result = if uses_format {
        // List 输入模式：仅进行 FULL compound 匹配
        selector_ops::extend_selector_with_mode(&sel, &extendee, &extender, true)
    } else {
        selector_ops::extend_selector(&sel, &extendee, &extender)
    };

    // selector-extend 始终返回 selector format（list of lists of strings）
    let fmt = fmt_to_vec_vec(&result);
    Ok(Some(selector_format::selector_format_to_value(fmt)))
}

// ─── selector-replace ───────────────────────────────────────────

/// `selector-replace($selector, $original, $replacement)` — 替换选择器中的片段。
pub fn call_replace(args: &[Value]) -> Result<Option<Value>> {
    validate_exact_args(args, 3, "selector-replace")?;
    let sel_fmt = selector_format::value_to_selector_format(&args[0])?;
    let orig_fmt = selector_format::value_to_selector_format(&args[1])?;
    let repl_fmt = selector_format::value_to_selector_format(&args[2])?;
    let sel = parse_selector(&format_to_string(&sel_fmt));
    let orig = parse_selector(&format_to_string(&orig_fmt));
    let repl = parse_selector(&format_to_string(&repl_fmt));
    let result = selector_ops::replace_selector(&sel, &orig, &repl);
    Ok(Some(Value::String(result.to_string(), false)))
}
