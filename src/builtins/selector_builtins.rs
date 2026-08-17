//! sass:selector built-in module.
//!
//! Implements: append, nest, extend, replace, is-superselector,
//! parse, unify.

use crate::ast::Arg;
use crate::env::Env;
use crate::error::{SassError, SourcePos};
use crate::value::{SassList, Value};
use crate::ast::ListSeparator;
use super::helpers::*;

/// Register all selector builtins.
pub fn register(env: &mut Env) {
    let span = tracing::debug_span!("register_selector", stage = "init", module = "selector");
    let _enter = span.enter();

    env.register_builtin("selector-append".into(), selector_append);
    env.register_builtin("selector-nest".into(), selector_nest);
    env.register_builtin("selector-extend".into(), selector_extend);
    env.register_builtin("selector-replace".into(), selector_replace);
    env.register_builtin("selector-is-superselector".into(), selector_is_super);
    env.register_builtin("selector-parse".into(), selector_parse);
    env.register_builtin("selector-unify".into(), selector_unify);
    env.register_builtin("selector-simple".into(), selector_simple);
}

fn get_args(args: &[Arg], env: &mut Env) -> Result<Vec<Value>, SassError> {
    eval_args(args, env, &[])
}

/// selector.append(".foo", ".bar") → ".foo.bar"
fn selector_append(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("selector-append: expected at least 2 arguments", SourcePos::default()));
    }
    let mut result = String::new();
    for v in &vals {
        let s = expect_string(v, "selector-append")?;
        // Remove leading & or space from subsequent selectors
        let cleaned = s.value.trim_start_matches('&').trim_start_matches(' ');
        if result.is_empty() {
            result.push_str(&s.value);
        } else {
            // If the next starts with a class/id, append directly
            if cleaned.starts_with('.') || cleaned.starts_with('#') || cleaned.starts_with(':') {
                result.push_str(cleaned);
            } else {
                result.push_str(&s.value);
            }
        }
    }
    Ok(unquoted_str(&result))
}

/// selector.nest(".foo", ".bar") → ".foo .bar"
fn selector_nest(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("selector-nest: expected at least 2 arguments", SourcePos::default()));
    }
    let mut result = String::new();
    for v in &vals {
        let s = expect_string(v, "selector-nest")?;
        let sel = s.value.trim();
        if result.is_empty() {
            result = sel.to_string();
        } else {
            // Handle & replacement
            if sel.contains('&') {
                result = sel.replace('&', &result);
            } else {
                result = format!("{} {}", result, sel);
            }
        }
    }
    Ok(unquoted_str(&result))
}

/// selector.extend(".foo", ".bar", ".baz") — not fully implemented
fn selector_extend(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 3 {
        return Err(SassError::eval("selector-extend: expected 3 arguments", SourcePos::default()));
    }
    let selector = expect_string(&vals[0], "selector-extend")?;
    let extendee = expect_string(&vals[1], "selector-extend")?;
    let extender = expect_string(&vals[2], "selector-extend")?;
    // Simple implementation: if selector contains extendee, add extender
    if selector.value.contains(&extendee.value) {
        Ok(unquoted_str(&format!("{}, {}", selector.value, extender.value)))
    } else {
        Ok(unquoted_str(&selector.value))
    }
}

/// selector.replace(".foo", ".bar", ".baz")
fn selector_replace(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 3 {
        return Err(SassError::eval("selector-replace: expected 3 arguments", SourcePos::default()));
    }
    let selector = expect_string(&vals[0], "selector-replace")?;
    let original = expect_string(&vals[1], "selector-replace")?;
    let replacement = expect_string(&vals[2], "selector-replace")?;
    let result = selector.value.replace(&original.value, &replacement.value);
    Ok(unquoted_str(&result))
}

/// selector.is-superselector(".foo", ".foo.bar") → true
fn selector_is_super(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("is-superselector: expected 2 arguments", SourcePos::default()));
    }
    let parent = expect_string(&vals[0], "is-superselector")?;
    let child = expect_string(&vals[1], "is-superselector")?;
    // Simple: parent is a superselector if child starts with parent
    let p = parent.value.trim();
    let c = child.value.trim();
    let is_super = c.starts_with(p) && (c.len() == p.len() || c.as_bytes()[p.len()] == b'.' || c.as_bytes()[p.len()] == b':');
    Ok(Value::Bool(is_super))
}

/// selector.parse(".foo .bar") → list of selectors
fn selector_parse(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("selector-parse: expected 1 argument", SourcePos::default()));
    }
    let s = expect_string(&vals[0], "selector-parse")?;
    // Parse into a list of compound selectors
    let parts: Vec<Value> = s.value.split(',').map(|p| {
        let subparts: Vec<Value> = p.trim().split(' ').map(|sp| unquoted_str(sp)).collect();
        if subparts.len() == 1 {
            subparts[0].clone()
        } else {
            Value::List(SassList::new(subparts, ListSeparator::Space, false))
        }
    }).collect();
    if parts.len() == 1 {
        Ok(parts[0].clone())
    } else {
        Ok(Value::List(SassList::new(parts, ListSeparator::Comma, false)))
    }
}

/// selector.unify(".foo", ".bar") → ".foo.bar"
fn selector_unify(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("selector-unify: expected 2 arguments", SourcePos::default()));
    }
    let s1 = expect_string(&vals[0], "selector-unify")?;
    let s2 = expect_string(&vals[1], "selector-unify")?;
    // Simple unification: merge two selectors
    let a = s1.value.trim();
    let b = s2.value.trim();
    if a == b {
        return Ok(unquoted_str(a));
    }
    // If both start with class/id, merge them
    if (a.starts_with('.') || a.starts_with('#')) && (b.starts_with('.') || b.starts_with('#')) {
        return Ok(unquoted_str(&format!("{}{}", a, b)));
    }
    Ok(unquoted_str(&format!("{} {}", a, b)))
}

/// selector.simple — extract simple selectors
fn selector_simple(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("selector-simple: expected 1 argument", SourcePos::default()));
    }
    let s = expect_string(&vals[0], "selector-simple")?;
    // Return the selector as-is (simplified)
    Ok(unquoted_str(&s.value))
}
