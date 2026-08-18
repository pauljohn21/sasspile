//! sass:meta built-in module.
//!
//! Implements: type-of, inspect, function-exists, mixin-exists,
//! variable-exists, global-variable-exists, get-function, get-mixin,
//! call, apply, content-exists, feature-exists, keywords,
//! module-functions, module-mixins, module-variables, load-css,
//! accepts-content.

use crate::ast::Arg;
use crate::env::Env;
use crate::error::{SassError, SourcePos};
use crate::value::Value;
use super::helpers::*;

/// Register all meta builtins.
pub fn register(env: &mut Env) {
    let span = tracing::debug_span!("register_meta", stage = "init", module = "meta");
    let _enter = span.enter();

    env.register_builtin("type-of".into(), meta_type_of);
    env.register_builtin("if".into(), meta_if);
    env.register_builtin("inspect".into(), meta_inspect);
    env.register_builtin("function-exists".into(), meta_function_exists);
    env.register_builtin("mixin-exists".into(), meta_mixin_exists);
    env.register_builtin("variable-exists".into(), meta_variable_exists);
    env.register_builtin("global-variable-exists".into(), meta_global_variable_exists);
    env.register_builtin("get-function".into(), meta_get_function);
    env.register_builtin("get-mixin".into(), meta_get_mixin);
    env.register_builtin("call".into(), meta_call);
    env.register_builtin("content-exists".into(), meta_content_exists);
    env.register_builtin("feature-exists".into(), meta_feature_exists);
    env.register_builtin("keywords".into(), meta_keywords);
    env.register_builtin("meta-type-of".into(), meta_type_of);
    env.register_builtin("meta-inspect".into(), meta_inspect);
    env.register_builtin("meta-function-exists".into(), meta_function_exists);
    env.register_builtin("meta-mixin-exists".into(), meta_mixin_exists);
    env.register_builtin("meta-variable-exists".into(), meta_variable_exists);
    env.register_builtin("meta-global-variable-exists".into(), meta_global_variable_exists);
    env.register_builtin("meta-get-function".into(), meta_get_function);
    env.register_builtin("meta-get-mixin".into(), meta_get_mixin);
    env.register_builtin("meta-call".into(), meta_call);
    env.register_builtin("meta-content-exists".into(), meta_content_exists);
    env.register_builtin("meta-feature-exists".into(), meta_feature_exists);
    env.register_builtin("meta-keywords".into(), meta_keywords);
    env.register_builtin("meta-module-functions".into(), meta_module_functions);
    env.register_builtin("meta-module-mixins".into(), meta_module_mixins);
    env.register_builtin("meta-module-variables".into(), meta_module_variables);
    env.register_builtin("meta-load-css".into(), meta_load_css);
    env.register_builtin("meta-accepts-content".into(), meta_accepts_content);
    env.register_builtin("meta-apply".into(), meta_apply);
}

fn get_args(args: &[Arg], env: &mut Env) -> Result<Vec<Value>, SassError> {
    eval_args(args, env, &[])
}

fn meta_type_of(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("type-of: expected 1 argument", SourcePos::default()));
    }
    // Check if an unquoted string is actually a known color name
    match &vals[0] {
        Value::String(s) if !s.quoted => {
            if is_color_name(&s.value) {
                return Ok(unquoted_str("color"));
            }
            Ok(unquoted_str(vals[0].type_name()))
        }
        _ => Ok(unquoted_str(vals[0].type_name())),
    }
}

/// Check if a string is a known CSS color name.
fn is_color_name(name: &str) -> bool {
    matches!(name.to_lowercase().as_str(),
        "red" | "green" | "blue" | "white" | "black" | "yellow" |
        "orange" | "purple" | "pink" | "cyan" | "magenta" |
        "gray" | "grey" | "brown" | "lime" | "navy" | "teal" |
        "aqua" | "fuchsia" | "silver" | "maroon" | "olive" |
        "transparent"
    )
}

fn meta_inspect(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("inspect: expected 1 argument", SourcePos::default()));
    }
    Ok(unquoted_str(&inspect_value(&vals[0])))
}

fn meta_function_exists(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("function-exists: expected 1 argument", SourcePos::default()));
    }
    let name = expect_string(&vals[0], "function-exists")?;
    // Check without namespace
    if env.function_exists(&name.value) || env.get_builtin(&name.value).is_some() {
        return Ok(Value::Bool(true));
    }
    // Check with namespace if arg has namespace
    if vals.len() >= 2 {
        if let Value::String(ns) = &vals[1] {
            if env.get_module_function(&ns.value, &name.value).is_some() {
                return Ok(Value::Bool(true));
            }
        }
    }
    Ok(Value::Bool(false))
}

fn meta_mixin_exists(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("mixin-exists: expected 1 argument", SourcePos::default()));
    }
    let name = expect_string(&vals[0], "mixin-exists")?;
    if env.mixin_exists(&name.value) {
        return Ok(Value::Bool(true));
    }
    Ok(Value::Bool(false))
}

fn meta_variable_exists(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("variable-exists: expected 1 argument", SourcePos::default()));
    }
    let name = expect_string(&vals[0], "variable-exists")?;
    Ok(Value::Bool(env.variable_exists(&name.value)))
}

fn meta_global_variable_exists(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("global-variable-exists: expected 1 argument", SourcePos::default()));
    }
    let name = expect_string(&vals[0], "global-variable-exists")?;
    Ok(Value::Bool(env.global_variable_exists(&name.value)))
}

fn meta_get_function(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("get-function: expected 1 argument", SourcePos::default()));
    }
    let name = expect_string(&vals[0], "get-function")?;
    Ok(Value::FunctionRef(name.value.clone()))
}

fn meta_get_mixin(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("get-mixin: expected 1 argument", SourcePos::default()));
    }
    let name = expect_string(&vals[0], "get-mixin")?;
    Ok(Value::MixinRef(name.value.clone()))
}

fn meta_call(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("call: expected at least 1 argument", SourcePos::default()));
    }
    match &vals[0] {
        Value::FunctionRef(name) => {
            // Build new args from remaining values, expanding spread args.
            // When an arg has `spread: true`, its evaluated value should be
            // a List whose items become individual positional arguments.
            let mut new_args: Vec<Arg> = Vec::new();
            for (i, arg) in args.iter().enumerate().skip(1) {
                if arg.spread {
                    if let Some(Value::List(l)) = vals.get(i) {
                        for item in &l.items {
                            new_args.push(Arg {
                                name: None,
                                value: crate::ast::Expr::Literal(item.clone()),
                                spread: false,
                            });
                        }
                    }
                } else if let Some(v) = vals.get(i) {
                    new_args.push(Arg {
                        name: None,
                        value: crate::ast::Expr::Literal(v.clone()),
                        spread: false,
                    });
                }
            }

            let func = env.get_function(name).cloned();
            if let Some(f) = func {
                return crate::eval::expr::bind_params_and_call(&f, &new_args, env, &[]);
            }
            if let Some(builtin) = env.get_builtin(name).copied() {
                return builtin(&new_args, env);
            }
            Err(SassError::eval(format!("call: function {} not found", name), SourcePos::default()))
        }
        _ => Err(SassError::eval("call: expected function reference", SourcePos::default())),
    }
}

fn meta_content_exists(_args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    Ok(Value::Bool(env.get_content().is_some()))
}

fn meta_feature_exists(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("feature-exists: expected 1 argument", SourcePos::default()));
    }
    let name = expect_string(&vals[0], "feature-exists")?;
    // Sass spec: return false for most features, true for a few known ones
    let known = matches!(name.value.as_str(), "global-variable-shadowing" | "extend-selector-pseudoclass");
    Ok(Value::Bool(known))
}

fn meta_keywords(_args: &[Arg], _env: &mut Env) -> Result<Value, SassError> {
    // Returns a map of keyword arguments from an arglist
    // For now, return an empty map
    Ok(Value::Map(crate::value::SassMap::new()))
}

fn meta_module_functions(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("module-functions: expected 1 argument", SourcePos::default()));
    }
    let _ns = expect_string(&vals[0], "module-functions")?;
    let map = crate::value::SassMap::new();
    // Return a map of function name → function reference
    Ok(Value::Map(map))
}

fn meta_module_mixins(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("module-mixins: expected 1 argument", SourcePos::default()));
    }
    let _ns = expect_string(&vals[0], "module-mixins")?;
    let map = crate::value::SassMap::new();
    Ok(Value::Map(map))
}

fn meta_module_variables(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("module-variables: expected 1 argument", SourcePos::default()));
    }
    let _ns = expect_string(&vals[0], "module-variables")?;
    let map = crate::value::SassMap::new();
    Ok(Value::Map(map))
}

fn meta_load_css(_args: &[Arg], _env: &mut Env) -> Result<Value, SassError> {
    // @load-css loads a CSS file — return null (no CSS to inject in this context)
    Ok(Value::Null)
}

fn meta_accepts_content(_args: &[Arg], _env: &mut Env) -> Result<Value, SassError> {
    // Check if a mixin accepts content
    // For now return true for most mixins
    Ok(Value::Bool(true))
}

fn meta_apply(_args: &[Arg], _env: &mut Env) -> Result<Value, SassError> {
    // meta.apply is like meta.call but for mixins
    Ok(Value::Null)
}

/// Inspect a value — produce a Sass representation string.
fn inspect_value(val: &Value) -> String {
    match val {
        Value::Number(n) => n.to_css_string(),
        Value::String(s) => {
            if s.quoted {
                format!("\"{}\"", s.value)
            } else {
                s.value.clone()
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Color(c) => c.to_string(),
        Value::List(l) => {
            let sep = match l.separator {
                crate::ast::ListSeparator::Comma => ", ",
                crate::ast::ListSeparator::Slash => " / ",
                _ => " ",
            };
            let parts: Vec<String> = l.items.iter().map(inspect_value).collect();
            if l.bracketed {
                format!("[{}]", parts.join(sep))
            } else {
                format!("({})", parts.join(sep))
            }
        }
        Value::Map(m) => {
            let parts: Vec<String> = m.entries.iter()
                .map(|(k, v)| format!("{}: {}", inspect_value(k), inspect_value(v)))
                .collect();
            format!("({})", parts.join(", "))
        }
        Value::Calculation(c) => format!("{}(...)", c.name),
        Value::FunctionRef(name) => format!("get-function(\"{}\")", name),
        Value::MixinRef(name) => format!("get-mixin(\"{}\")", name),
    }
}

/// `if($condition, $if-true, $if-false)` — conditional value.
///
/// Evaluates `$condition`; if truthy, returns `$if-true`, otherwise `$if-false`.
/// Both branches are pre-evaluated by the caller (standard builtin semantics).
fn meta_if(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 3 {
        return Err(SassError::eval(
            "if() requires 3 arguments: $condition, $if-true, $if-false",
            SourcePos::default(),
        ));
    }
    if vals[0].is_truthy() {
        Ok(vals[1].clone())
    } else {
        Ok(vals[2].clone())
    }
}
