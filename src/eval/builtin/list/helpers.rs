//! List 内建函数的辅助函数——参数名和参数合并。

use crate::parse::ast::*;
use imbl::HashMap;

/// 返回每个 list 函数的参数名列表（按位置顺序）。
pub(super) fn list_param_names(name: &str) -> &'static [&'static str] {
    match name {
        "length" | "list-length" => &["list"],
        "nth" => &["list", "n"],
        "append" => &["list", "val", "separator"],
        "join" => &["list1", "list2", "separator", "bracketed"],
        "index" => &["list", "value"],
        "list-separator" | "separator" => &["list"],
        "set-nth" => &["list", "n", "value"],
        "is-bracketed" => &["list"],
        "list-slash" => &[],
        "zip" => &[],
        _ => &[],
    }
}

/// 合并位置参数和命名参数（复用 string 模块的 `merge_args` 逻辑）。
pub(super) fn merge_list_args(
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    name: &str,
) -> Vec<Value> {
    let param_names = list_param_names(name);
    let result: Vec<Value> = param_names
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
        true => {
            let mut r = result;
            r.extend_from_slice(&pos_args[param_names.len()..]);
            r
        }
        false => result,
    }
}
