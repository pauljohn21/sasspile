//! Mixin 执行。

use crate::error::{Result, SassError};
use crate::parse::{ast::Arg, Node};
use crate::css::CssNode;
use super::env::{Env, MixinDef};
use super::{eval_nodes, eval_value};
use crate::eval::value::Value;

/// 执行 @include。
pub fn exec_include(
    name: &str,
    args: &[Arg],
    content: Option<&[Node]>,
    env: Env,
) -> Result<(Option<Vec<CssNode>>, Env)> {
    let mixin = env.get_mixin(name)
        .ok_or_else(|| SassError::eval(format!("Undefined mixin: {name}")))?
        .clone();

    // 绑定参数
    let child_env = bind_params(&mixin, args, &env)?;

    // 注入 @content
    let child_env = if let Some(body) = content {
        child_env.with_content(body.to_vec())
    } else {
        child_env
    };

    let css = eval_nodes(&mixin.body, child_env)?.0;
    Ok((Some(css), env))
}

/// 绑定参数到环境——支持位置参数、命名参数、rest 参数和 spread。
fn bind_params(mixin: &MixinDef, args: &[Arg], env: &Env) -> Result<Env> {
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
    for (i, param) in mixin.params.iter().enumerate() {
        if param.rest {
            // 收集剩余位置参数
            let rest: Vec<Value> = positional.iter().skip(i)
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

    Ok(child_env)
}
