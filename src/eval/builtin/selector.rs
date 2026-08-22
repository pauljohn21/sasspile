//! selector 内建函数。

use crate::error::{Result, SassError};
use crate::eval::value::Value;
use crate::eval::env::Env;
use crate::parse::ast::Arg;
use crate::lex::token::QuoteStyle;

pub fn dispatch(field: &str, args: &[Arg], env: &Env) -> Result<Value> {
    let args: Vec<Value> = args.iter().map(|a| crate::eval::eval_value(&a.value, env)).collect();
    match field {
        "nest" => {
            if args.is_empty() {
                return Err(SassError::eval("selector-nest() expects at least one argument"));
            }
            let result = nest_selectors(&args);
            Ok(Value::String(result, QuoteStyle::None))
        }
        "append" => {
            if args.is_empty() {
                return Err(SassError::eval("selector-append() expects at least one argument"));
            }
            let result = append_selectors(&args);
            Ok(Value::String(result, QuoteStyle::None))
        }
        "parse" => match &args[..] {
            [v] => {
                let s = v.to_css_string();
                // 返回一个简单的选择器列表结构
                let selectors: Vec<Value> = s.split(',').map(|s| {
                    Value::String(s.trim().to_string(), QuoteStyle::None)
                }).collect();
                Ok(Value::List(selectors, crate::eval::value::Separator::Comma, false))
            }
            _ => Err(SassError::eval("selector-parse() expects one argument")),
        },
        "is_super_selector" => match &args[..] {
            [super_sel, sub_sel] => {
                let sup = super_sel.to_css_string();
                let sub = sub_sel.to_css_string();
                // 简化：如果 super 包含 sub 的所有选择器
                let is_super = sup.split(',').all(|s| {
                    sub.split(',').any(|t| t.trim().contains(s.trim()))
                });
                Ok(Value::Bool(is_super))
            }
            _ => Err(SassError::eval("is-superselector() expects two selectors")),
        },
        "extend" | "replace" | "unify" | "simple" => {
            Err(SassError::eval(format!("selector.{field}() not yet implemented")))
        }
        _ => Err(SassError::eval(format!("Unknown selector function: {field}"))),
    }
}

/// 嵌套选择器——`selector-nest(".a", ".b")` → `.a .b`。
fn nest_selectors(selectors: &[Value]) -> String {
    let parts: Vec<String> = selectors.iter().map(|v| v.to_css_string()).collect();
    parts.join(" ")
}

/// 追加选择器——`selector-append(".a", "-b")` → `.a-b`。
fn append_selectors(selectors: &[Value]) -> String {
    selectors.iter().map(|v| v.to_css_string()).collect()
}
