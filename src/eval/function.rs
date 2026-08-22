//! 函数调用——用户函数 + 内建函数。

use crate::error::{Result, SassError};
use crate::parse::ast::Arg;
use crate::eval::value::Value;
use super::env::{Env, FunctionDef};
use super::{eval_nodes, eval_value};
use crate::css::CssNode;

/// 调用函数——先查用户函数，再查内建函数。
pub fn call_function(name: &str, args: &[Arg], env: &Env) -> Result<Value> {
    // 用户函数
    if let Some(func) = env.get_function(name) {
        return call_user_function(func.clone(), args, env);
    }

    // 内建函数
    if let Some(result) = super::builtin::dispatch::dispatch_builtin(name, args, env) {
        return result;
    }

    Err(SassError::eval(format!("Undefined function: {name}")))
}

/// 调用用户定义的函数——支持命名参数。
fn call_user_function(func: FunctionDef, args: &[Arg], env: &Env) -> Result<Value> {
    let mut child_env = env.enter_scope();

    // 先处理命名参数
    let mut named: std::collections::HashMap<&str, &Arg> = std::collections::HashMap::new();
    let mut positional: Vec<&Arg> = Vec::new();
    for a in args {
        if let Some(ref name) = a.name {
            named.insert(name.as_str(), a);
        } else {
            positional.push(a);
        }
    }

    let mut pos_idx = 0;
    for param in &func.params {
        if param.rest {
            let rest: Vec<Value> = positional.iter().skip(pos_idx)
                .map(|a| eval_value(&a.value, env))
                .collect();
            child_env = child_env.define_var(&param.name, Value::ArgList(rest));
            break;
        }

        // 优先命名参数
        let value = if let Some(a) = named.get(param.name.as_str()) {
            eval_value(&a.value, env)
        } else if pos_idx < positional.len() {
            let v = eval_value(&positional[pos_idx].value, env);
            pos_idx += 1;
            v
        } else if let Some(ref default) = param.default {
            eval_value(default, &child_env)
        } else {
            return Err(SassError::eval(format!(
                "Missing argument for ${}",
                param.name
            )));
        };

        child_env = child_env.define_var(&param.name, value);
    }

    // 求值函数体——查找 Return 节点
    let css = eval_nodes(&func.body, child_env)?.0;
    for node in &css {
        if let CssNode::Return(v) = node {
            return Ok(v.clone());
        }
    }

    // 没有显式 return
    Ok(Value::Null)
}
