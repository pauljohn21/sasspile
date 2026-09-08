//! 选择器格式转换——Selector Format 与 Value 之间的双向转换。
//!
//! Selector Format 定义（Sass 内部格式）：
//! - 外层：逗号分隔的 complex selectors（`Vec<Vec<String>>`）
//! - 内层：空格分隔的 compound strings（descendant 关系）
//!
//! 例：`"c d, e f"` ↔ `[["c", "d"], ["e", "f"]]`

use crate::error::{Result, SassError};
use crate::parse::ast::{Separator, Value};

/// 解析选择器字符串为 Selector Format（外层 = complex 列表，内层 = token 字符串列表，包含组合符）。
///
/// 显式组合符（>、+、~）会作为单独的字符串元素保留在列表中。
/// 示例：
/// - "a, b c" → [["a"], ["b", "c"]]
/// - "a > b" → [["a", ">", "b"]]
/// - "a + b ~ c" → [["a", "+", "b", "~", "c"]]
/// - ".a > .b" → [[".a", ">", ".b"]]
pub fn string_to_selector_format(input: &str) -> Vec<Vec<String>> {
    input
        .split(',')
        .map(|part| {
            part.split_whitespace()
                .filter(|s| !s.is_empty())
                .map(|s| {
                    // 处理开头是组合符的情况，比如 "> .b" 会 split成 [">", ".b"]，保留即可
                    s.trim().to_string()
                })
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty())
        .collect()
}

/// 将 Selector Format 转为 Value（list of lists of strings）。
pub fn selector_format_to_value(fmt: Vec<Vec<String>>) -> Value {
    let complexes: Vec<Value> = fmt
        .into_iter()
        .map(|compounds| {
            let elements: Vec<Value> = compounds
                .into_iter()
                .map(|s| Value::String(s, false))
                .collect();
            Value::List(elements, Separator::Space, false)
        })
        .collect();
    Value::List(complexes, Separator::Comma, false)
}

/// 将 Value（string 或 selector format）转换为 Selector Format。
pub fn value_to_selector_format(value: &Value) -> Result<Vec<Vec<String>>> {
    match value {
        Value::String(s, _) => Ok(string_to_selector_format(s)),
        Value::List(elements, _, _) => value_list_to_format(elements, 0),
        _ => Err(SassError::Eval(format!(
            "{value} is not a valid selector: it must be a string, \
             a list of strings, or a list of lists of strings."
        ))),
    }
}

/// 将 Value 列表（selector format）转换为 Selector Format。
/// depth: 当前递归深度，防止多层嵌套。
fn value_list_to_format(elements: &[Value], depth: usize) -> Result<Vec<Vec<String>>> {
    if depth > 2 {
        return Err(SassError::Eval(
            "Invalid selector: too many nested lists".into(),
        ));
    }

    // 首先检查是否是列表的列表（selector format）
    let is_list_of_lists = elements.iter().all(|e| matches!(e, Value::List(_, _, _)));
    if is_list_of_lists && !elements.is_empty() {
        return elements
            .iter()
            .map(|inner_list| match inner_list {
                Value::List(inner, _, _) => list_to_compounds(inner, depth + 1),
                _ => unreachable!(),
            })
            .collect();
    }

    // 否则当作complex selector处理：每个元素是字符串或compound字符串
    elements
        .iter()
        .map(|elem| match elem {
            Value::String(s, _) => Ok(string_to_selector_format(s)
                .into_iter()
                .next()
                .unwrap_or_default()),
            Value::List(inner, _, _) => list_to_compounds(inner, depth + 1),
            _ => Err(SassError::Eval(format!(
                "{elem} is not a valid selector: it must be a string, \
                 a list of strings, or a list of lists of strings."
            ))),
        })
        .collect()
}

/// 将内部 list 转换为 compound strings。
fn list_to_compounds(elements: &[Value], depth: usize) -> Result<Vec<String>> {
    if depth > 2 {
        return Err(SassError::Eval(
            "Invalid selector: too many nested lists".into(),
        ));
    }

    elements
        .iter()
        .map(|e| match e {
            Value::String(s, _) => Ok(s.clone()),
            Value::List(inner, _, _) => {
                // 嵌套的列表，合并为一个compound字符串
                list_to_compounds(inner, depth + 1).map(|parts| parts.join(""))
            }
            _ => Err(SassError::Eval(format!(
                "{e} is not a valid selector: it must be a string, \
                 a list of strings, or a list of lists of strings."
            ))),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_format_roundtrip() {
        let fmt = string_to_selector_format(".a, .b .c");
        assert_eq!(fmt, vec![vec![".a"], vec![".b", ".c"]]);
    }

    #[test]
    fn test_format_to_value_roundtrip() {
        let fmt = vec![vec!["c".to_string(), "d".to_string()], vec!["e".to_string(), "f".to_string()]];
        let value = selector_format_to_value(fmt.clone());
        let result = value_to_selector_format(&value).expect("roundtrip");
        assert_eq!(result, fmt);
    }
}
