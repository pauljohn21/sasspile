//! meta 内建函数——骨架。

use crate::error::{Result, SassError};
use crate::eval::value::Value;
use crate::eval::env::Env;
use crate::parse::ast::Arg;
use crate::eval::eval_value;
use crate::lex::token::QuoteStyle;

pub fn dispatch(field: &str, args: &[Arg], env: &Env) -> Result<Value> {
    let args: Vec<Value> = args.iter().map(|a| eval_value(&a.value, env)).collect();
    match field {
        "type_of" => match &args[..] {
            [v] => Ok(Value::String(type_name(v), QuoteStyle::None)),
            _ => Err(SassError::eval("type-of() expects one argument")),
        },
        "inspect" => match &args[..] {
            [v] => Ok(Value::String(v.to_css_string(), QuoteStyle::None)),
            _ => Err(SassError::eval("inspect() expects one argument")),
        },
        "feature_exists" => match &args[..] {
            [Value::String(s, _)] | [Value::Ident(s)] => {
                Ok(Value::Bool(matches!(s.as_str(), "global-variable-shadowing")))
            }
            _ => Err(SassError::eval("feature-exists() expects a string")),
        },
        "variable_exists" => match &args[..] {
            [Value::String(s, _)] | [Value::Ident(s)] => {
                Ok(Value::Bool(env.get_var(s).is_some()))
            }
            _ => Err(SassError::eval("variable-exists() expects a string")),
        },
        "global_variable_exists" => match &args[..] {
            [Value::String(s, _)] | [Value::Ident(s)] => {
                Ok(Value::Bool(env.get_var(s).is_some()))
            }
            _ => Err(SassError::eval("global-variable-exists() expects a string")),
        },
        "mixin_exists" => match &args[..] {
            [Value::String(s, _)] | [Value::Ident(s)] => {
                Ok(Value::Bool(env.get_mixin(s).is_some()))
            }
            _ => Err(SassError::eval("mixin-exists() expects a string")),
        },
        "function_exists" => match &args[..] {
            [Value::String(s, _)] | [Value::Ident(s)] => {
                Ok(Value::Bool(env.get_function(s).is_some()
                    || crate::eval::builtin::dispatch::is_known_builtin(s)))
            }
            _ => Err(SassError::eval("function-exists() expects a string")),
        },
        "content_exists" => Ok(Value::Bool(env.get_content().is_some())),
        "get_function" => match &args[..] {
            [Value::String(s, _)] | [Value::Ident(s)] => {
                Ok(Value::Function(crate::eval::value::FunctionRef {
                    name: s.clone(),
                    is_builtin: crate::eval::builtin::dispatch::is_known_builtin(s),
                }))
            }
            _ => Err(SassError::eval("get-function() expects a string")),
        },
        "call" => match &args[..] {
            [Value::Function(f), rest @ ..] => {
                let dummy_args: Vec<Arg> = rest.iter().map(|v| Arg {
                    name: None, value: v.clone(), spread: false,
                }).collect();
                crate::eval::function::call_function(&f.name, &dummy_args, env)
            }
            [Value::String(s, _), rest @ ..] | [Value::Ident(s), rest @ ..] => {
                let dummy_args: Vec<Arg> = rest.iter().map(|v| Arg {
                    name: None, value: v.clone(), spread: false,
                }).collect();
                crate::eval::function::call_function(s, &dummy_args, env)
            }
            _ => Err(SassError::eval("call() expects a function")),
        },
        "keywords" => match &args[..] {
            [Value::ArgList(_)] => Ok(Value::Map(Vec::new())),
            _ => Err(SassError::eval("keywords() expects an argument list")),
        },
        "get_mixin" => match &args[..] {
            [Value::String(s, _)] | [Value::Ident(s)] => {
                Ok(Value::String(s.clone(), QuoteStyle::None))
            }
            _ => Err(SassError::eval("get-mixin() expects a string")),
        },
        "module_functions" => match &args[..] {
            [Value::String(_, _)] | [Value::Ident(_)] => {
                Ok(Value::Map(Vec::new()))
            }
            _ => Err(SassError::eval("module-functions() expects a string")),
        },
        "module_variables" => match &args[..] {
            [Value::String(_, _)] | [Value::Ident(_)] => {
                Ok(Value::Map(Vec::new()))
            }
            _ => Err(SassError::eval("module-variables() expects a string")),
        },
        "load_css" => {
            // TODO: 实现真正的 CSS 加载
            Ok(Value::Null)
        }
        _ => Err(SassError::eval(format!("Unknown meta function: {field}"))),
    }
}

fn type_name(v: &Value) -> String {
    match v {
        Value::Number(_, _) => "number".to_string(),
        Value::String(_, QuoteStyle::None) => "string".to_string(),
        Value::String(_, _) => "string".to_string(),
        Value::Ident(_) => "string".to_string(),
        Value::Color(_) => "color".to_string(),
        Value::Bool(_) => "bool".to_string(),
        Value::Null => "null".to_string(),
        Value::List(_, _, _) => "list".to_string(),
        Value::Map(_) => "map".to_string(),
        Value::Function(_) => "function".to_string(),
        Value::Variable(_) => "string".to_string(),
        Value::ArgList(_) => "arglist".to_string(),
        // AST 级别——求值前 fallback
        Value::Call(_, _) | Value::Interp(_) | Value::BinOp(_)
        | Value::UnaryOp(_, _) | Value::Calc(_) | Value::Paren(_) => "string".to_string(),
    }
}
