//! `selector-append` 实现。

use crate::css::selector_format;
use crate::error::{Result, SassError};
use crate::parse::ast::Value;

/// `selector-append($selectors...)` — 将多个选择器字符串拼接。
pub fn call_append(args: &[Value]) -> Result<Option<Value>> {
    if args.is_empty() {
        return Err(SassError::Eval(
            "$selectors: At least one selector must be passed.".into(),
        ));
    }
    let formats: Vec<Vec<Vec<String>>> = args
        .iter()
        .map(selector_format::value_to_selector_format)
        .collect::<Result<Vec<_>>>()?;
    let init = formats[0].clone();
    let result = formats[1..]
        .iter()
        .try_fold(init, |acc, next| append_two(&acc, next))?;
    Ok(Some(selector_format::selector_format_to_value(result)))
}

/// 笛卡尔积合并两个 complex 列表。
fn append_two(a: &[Vec<String>], b: &[Vec<String>]) -> Result<Vec<Vec<String>>> {
    a.iter()
        .flat_map(|ca| b.iter().map(move |cb| append_complex(ca, cb)))
        .try_fold(Vec::new(), |mut acc, result| {
            result.map(|mut v| {
                acc.append(&mut v);
                acc
            })
        })
}

/// 合并两个 complex selectors——末尾化合物字符串拼接。
fn append_complex(a: &[String], b: &[String]) -> Result<Vec<Vec<String>>> {
    let a_last = a.last().cloned().unwrap_or_default();
    let b_first = b.first().cloned().unwrap_or_default();

    match a_last.as_str() {
        ">" | "+" | "~" => {
            return Err(SassError::Eval(format!("Can't append {b_first} to {a_last}")));
        }
        _ => {}
    }
    match b_first.as_str() {
        ">" | "+" | "~" if b.len() == 1 => {
            return Err(SassError::Eval(format!("Can't append {b_first} to {a_last}")));
        }
        "*" => {
            return Err(SassError::Eval(format!("Can't append * to {a_last}")));
        }
        s if s.starts_with('|') && b.len() == 1 => {
            return Err(SassError::Eval(format!("Can't append {b_first} to {a_last}")));
        }
        _ => {}
    }
    if b.len() == 1 && b[0] == "&" {
        return Err(SassError::Eval("Parent selectors aren't allowed here.".into()));
    }
    if matches!(b_first.as_str(), ">" | "+" | "~") {
        let b_display = b.join(" ");
        let a_display = a.join(" ");
        return Err(SassError::Eval(format!(
            "Can't append {b_display} to {a_display}."
        )));
    }
    let merged = format!("{a_last}{b_first}");
    let mut result = a[..a.len() - 1].to_vec();
    result.push(merged);
    result.extend_from_slice(&b[1..]);
    Ok(vec![result])
}
