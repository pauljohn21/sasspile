//! 函数调用——用户函数 + 内建函数。

use crate::error::{Result, SassError};
use crate::parse::ast::Arg;
use crate::eval::value::Value;
use super::env::Env;
use super::{eval_nodes, eval_value};
use crate::css::CssNode;
use crate::parse::Node;

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

/// 调用用户定义的函数。
fn call_user_function(func: super::env::FunctionDef, args: &[Arg], env: &Env) -> Result<Value> {
    let mut child_env = env.enter_scope();

    // 绑定参数
    for (i, param) in func.params.iter().enumerate() {
        let value = if i < args.len() {
            eval_value(&args[i].value, env)
        } else if let Some(ref default) = param.default {
            eval_value(default, &child_env)
        } else {
            return Err(SassError::eval(format!("Missing argument for ${}", param.name)));
        };
        child_env = child_env.define_var(&param.name, value);
    }

    // 求值函数体——查找 Return 节点
    let css = eval_nodes(&func.body, child_env)?;
    for node in &css {
        if let CssNode::Return(v) = node {
            return Ok(v.clone());
        }
    }

    // 没有显式 return
    Ok(Value::Null)
}
