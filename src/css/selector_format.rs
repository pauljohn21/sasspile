//! 选择器格式转换——Selector Format 与 Value 之间的双向转换。
//!
//! Selector Format 定义（Sass 内部格式）：
//! - 外层：逗号分隔的 complex selectors（`Vec<Vec<String>>`）
//! - 内层：空格分隔的 compound strings（descendant 关系）
//!
//! 例：`"c d, e f"` ↔ `[["c", "d"], ["e", "f"]]`

use crate::error::{Result, SassError};
use crate::parse::ast::{Separator, Value};

/// 验证单个token是否是有效的选择器部分
fn is_valid_selector_token(token: &str) -> bool {
    // 组合符有效
    if matches!(token, ">" | "+" | "~") {
        return true;
    }
    // 其他token：至少有一个合法字符（字母、数字、_、-、.、#、:、[]等）
    !token.is_empty() && token.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '#' || c == ':' || c == '[' || c == ']' || c == '(' || c == ')' || c == '%' || c == '&'
    })
}

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

/// 解析选择器字符串为 Selector Format，带验证
pub fn string_to_selector_format_validated(input: &str) -> Result<Vec<Vec<String>>> {
    let fmt = string_to_selector_format(input);
    // 验证所有token
    for complex in &fmt {
        for token in complex {
            if !is_valid_selector_token(token) {
                return Err(SassError::Eval(format!(
                    "Invalid selector: \"{token}\" is not a valid selector token"
                )));
            }
        }
    }
    Ok(fmt)
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
        Value::String(s, _) => Ok(string_to_selector_format_validated(s)?),
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

    if elements.is_empty() {
        return Ok(Vec::new());
    }

    // 判断元素类型
    let all_strings = elements.iter().all(|e| matches!(e, Value::String(_, _)));
    let all_lists = elements.iter().all(|e| matches!(e, Value::List(_, _, _)));

    if all_strings {
        // 所有元素都是字符串：整个列表是一个complex selector
        list_to_compounds(elements, depth).map(|v| vec![v])
    } else if all_lists {
        // 所有元素都是列表：每个元素是一个complex selector
        elements
            .iter()
            .map(|inner_list| match inner_list {
                Value::List(inner, _, _) => list_to_compounds(inner, depth + 1),
                _ => unreachable!(),
            })
            .collect()
    } else {
        // 混合类型，无效
        Err(SassError::Eval(
            "Invalid selector: list must contain all strings or all lists".into(),
        ))
    }
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

    #[test]
    fn test_string_with_combinators() {
        let fmt = string_to_selector_format("a > b + c ~ d");
        assert_eq!(fmt, vec![vec!["a", ">", "b", "+", "c", "~", "d"]]);
    }

    #[test]
    fn test_value_list_input() {
        let value = Value::List(vec![
            Value::String("a".into(), false),
            Value::String("b".into(), false),
        ], Separator::Space, false);
        let fmt = value_to_selector_format(&value).unwrap();
        assert_eq!(fmt, vec![vec!["a", "b"]]);
    }

    #[test]
    fn test_value_list_of_lists_input() {
        let value = Value::List(vec![
            Value::List(vec![
                Value::String("a".into(), false),
                Value::String("b".into(), false),
            ], Separator::Space, false),
            Value::List(vec![
                Value::String("c".into(), false),
                Value::String("d".into(), false),
            ], Separator::Space, false),
        ], Separator::Comma, false);
        let fmt = value_to_selector_format(&value).unwrap();
        assert_eq!(fmt, vec![vec!["a", "b"], vec!["c", "d"]]);
    }

    #[test]
    fn test_invalid_input_error() {
        let value = Value::Number(1.0, None);
        let result = value_to_selector_format(&value);
        assert!(result.is_err());
    }
}
