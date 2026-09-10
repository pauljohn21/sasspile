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
    // 其他token：至少有一个合法字符（字母、数字、_、-、.、#、:、[]、*、|、"、'=等）
    // * = 通用选择器, | = 命名空间分隔符, " 和 ' 用于属性值引号
    !token.is_empty() && token.chars().all(|c| {
        c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '#' | ':' | '[' | ']' | '(' | ')' | '%' | '&' | '*' | '|' | '"' | '\'' | '=' | '~' | '^' | '$')
    })
}

/// 解析选择器字符串为 Selector Format（外层 = complex 列表，内层 = token 字符串列表，包含组合符）。
///
/// 显式组合符（>、+、~）会作为单独的字符串元素保留在列表中。
/// 示例：
/// - `"a, b c"` → `[["a"], ["b", "c"]]`
/// - `"a > b"` → `[["a", ">", "b"]]`
/// - `"a + b ~ c"` → `[["a", "+", "b", "~", "c"]]`
/// - `".a > .b"` → `[[".a", ">", ".b"]]`
/// - `"[c]d"` → `[["[c]", "d"]]`
pub fn string_to_selector_format(input: &str) -> Vec<Vec<String>> {
    input
        .split(',')
        .map(|part| split_compound_selector(part))
        .filter(|v| !v.is_empty())
        .collect()
}

/// 将单个 complex selector 字符串拆分为 compound 标记列表。
///
/// 处理复合选择器边界（如 `[c]d` → `["[c]", "d"]`）。
fn split_compound_selector(input: &str) -> Vec<String> {
    let chars: Vec<char> = input.chars().collect();
    let mut result = Vec::new();
    let mut current = String::new();
    let mut i = 0;
    let mut bracket_depth: i32 = 0;
    while i < chars.len() {
        let c = chars[i];
        // 在顶层（不在括号/引号/括号内）检查是否是新 simple selector 的开始
        if bracket_depth == 0 && !current.is_empty() {
            let starts_new = matches!(c, '.' | '#' | '%' | '[' | ':' | '&' | '*')
                || c.is_ascii_alphanumeric();
            // 特殊情况：当前以 ] 结尾，后跟字母/数字 → 新 type selector
            let prev_bracket_close = current.ends_with(']');
            if starts_new && (prev_bracket_close || !c.is_ascii_alphanumeric()) {
                // 但如果当前是组合符（> + ~）或空，不分割
                if !matches!(current.as_str(), ">" | "+" | "~") && !current.is_empty() {
                    result.push(current);
                    current = String::new();
                }
            }
        }
        current.push(c);
        match c {
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            _ => {}
        }
        i += 1;
    }
    if !current.is_empty() && !current.trim().is_empty() {
        result.push(current);
    }
    // 分割组合符：如果某个 token 包含组合符（> + ~），进一步拆分
    result
        .into_iter()
        .flat_map(|token| split_combinators(token))
        .filter(|s| !s.is_empty())
        .collect()
}

/// 将包含组合符的 token 拆分为独立标记。
/// 例如 `"a > b"` → `["a", ">", "b"]`。
fn split_combinators(token: String) -> Vec<String> {
    let trimmed = token.trim();
    // 如果是纯组合符
    if matches!(trimmed, ">" | "+" | "~") {
        return vec![trimmed.to_string()];
    }
    // 不包含组合符
    if !trimmed.contains(['>', '+', '~']) {
        return vec![trimmed.to_string()];
    }
    // 拆分组合符
    let mut result = Vec::new();
    let mut current = String::new();
    for c in trimmed.chars() {
        if matches!(c, '>' | '+' | '~') {
            if !current.trim().is_empty() {
                result.push(current.trim().to_string());
            }
            result.push(c.to_string());
            current = String::new();
        } else if c.is_whitespace() {
            if !current.trim().is_empty() {
                result.push(current.trim().to_string());
                current = String::new();
            }
        } else {
            current.push(c);
        }
    }
    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }
    result
}

/// 解析选择器字符串为 Selector Format，带验证。
///
/// 检测未闭合的属性选择器（如 `[c`、`[foo=bar`）并返回错误。
#[tracing::instrument(level = "debug", skip(input))]
pub fn string_to_selector_format_validated(input: &str) -> Result<Vec<Vec<String>>> {
    // 预检查：属性选择器闭合性
    check_attribute_closure(input)?;

    let fmt = string_to_selector_format(input);
    // 验证所有token（迭代器链替代 for 循环）
    fmt.iter()
        .flatten()
        .try_for_each(|token| {
            if !is_valid_selector_token(token) {
                return Err(SassError::Eval(format!(
                    "Invalid selector: \"{token}\" is not a valid selector token"
                )));
            }
            Ok(())
        })?;
    Ok(fmt)
}

/// 检查属性选择器 `[...]` 是否正确闭合。
#[tracing::instrument(level = "debug", skip(input))]
fn check_attribute_closure(input: &str) -> Result<()> {
    let (bracket_count, in_quotes) = input.chars().fold(
        (0isize, None::<char>),
        |(count, in_quotes), c| match in_quotes {
            Some(quote) if c == quote => (count, None),
            Some(_) => (count, in_quotes),
            None => match c {
                '"' | '\'' => (count, Some(c)),
                '[' => (count + 1, None),
                ']' => (count - 1, None),
                _ => (count, None),
            },
        },
    );

    if bracket_count == 0 && in_quotes.is_none() {
        Ok(())
    } else {
        Err(SassError::Eval(
            "Invalid selector: missing closing `]` for attribute selector".into(),
        ))
    }
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
        Value::List(elements, separator, _) => {
            value_list_to_format(elements, separator.clone(), 0)
        }
        _ => Err(SassError::Eval(format!(
            "{value} is not a valid selector: it must be a string, \
             a list of strings, or a list of lists of strings."
        ))),
    }
}

/// 将 Value 列表（selector format）转换为 Selector Format。
/// separator: 外层列表的分隔符（Comma 或 Space）。
/// depth: 当前递归深度，防止多层嵌套。
///
/// 支持三种模式：
/// - 纯 strings：
///   - Space separator: 整个列表是一个 complex selector
///   - Comma separator: 每个 string 是一个 complex selector
/// - 纯 lists：每个内部列表是一个 complex selector（comma 分隔）
/// - 混合类型：string 视为单 compound complex，list 视为多 compound complex
fn value_list_to_format(
    elements: &[Value],
    separator: Separator,
    depth: usize,
) -> Result<Vec<Vec<String>>> {
    if depth > 2 {
        return Err(SassError::Eval(
            "Invalid selector: too many nested lists".into(),
        ));
    }

    if elements.is_empty() {
        return Ok(Vec::new());
    }

    let all_strings = elements.iter().all(|e| matches!(e, Value::String(_, _)));
    let all_lists = elements.iter().all(|e| matches!(e, Value::List(_, _, _)));

    if all_strings {
        match separator {
            Separator::Comma => {
                // Comma-separated strings: 每个 string 是一个 complex selector
                elements
                    .iter()
                    .map(|e| match e {
                        Value::String(s, _) => {
                            Ok(string_to_selector_format_validated(s)?)
                        }
                        _ => unreachable!(),
                    })
                    .try_fold(Vec::new(), |mut acc, result| {
                        result.map(|mut v| {
                            acc.append(&mut v);
                            acc
                        })
                    })
            }
            _ => {
                // Space-separated strings: 整个列表是一个 complex selector
                list_to_compounds(elements, depth).map(|v| vec![v])
            }
        }
    } else if all_lists {
        // 所有元素都是列表：每个元素是一个 complex selector
        elements
            .iter()
            .map(|inner_list| match inner_list {
                Value::List(inner, sep, _) => value_list_to_format(inner, sep.clone(), depth + 1),
                _ => unreachable!(),
            })
            .try_fold(Vec::new(), |mut acc, result| {
                result.map(|mut v| {
                    acc.append(&mut v);
                    acc
                })
            })
    } else {
        // 混合类型：逐个元素处理，string → 单 compound complex，list → 多 compound complex
        elements
            .iter()
            .map(|e| match e {
                Value::String(s, _) => {
                    // string 视为单 compound complex：验证并解析
                    let fmt = string_to_selector_format_validated(s)?;
                    Ok(fmt)
                }
                Value::List(inner, sep, _) => {
                    // list 视为多 compound complex
                    value_list_to_format(inner, sep.clone(), depth + 1)
                }
                _ => Err(SassError::Eval(format!(
                    "{e} is not a valid selector: it must be a string, \
                     a list of strings, or a list of lists of strings."
                ))),
            })
            .try_fold(Vec::new(), |mut acc, result| {
                result.map(|mut v| {
                    acc.append(&mut v);
                    acc
                })
            })
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


