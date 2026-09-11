//! Selector 内建函数入口 + 参数合并。
//!
//! 具体实现拆分到子模块：
//! - `selector_append` — selector-append
//! - `selector_nest` — selector-nest
//! - `selector_ops` — is-super / parse / simple-selectors / unify / extend / replace

use crate::error::Result;
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
    if pos_args.len() > param_names.len() {
        result.extend_from_slice(&pos_args[param_names.len()..]);
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
        "selector-append" => super::selector_append::call_append(args),
        "selector-nest" => super::selector_nest::call_nest(args),
        "selector-is-superselector" | "selector-is-super" => super::selector_ops::call_is_super(args),
        "selector-parse" => super::selector_ops::call_parse(args),
        "selector-simple-selectors" => super::selector_ops::call_simple_selectors(args),
        "selector-unify" => super::selector_ops::call_unify(args),
        "selector-extend" => super::selector_ops::call_extend(args),
        "selector-replace" => super::selector_ops::call_replace(args),
        _ => Ok(None),
    }
}
