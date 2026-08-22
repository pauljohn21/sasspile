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
        let parent_env = env.enter_scope();
        child_env.with_content(body.to_vec())
    } else {
        child_env
    };

    let css = eval_nodes(&mixin.body, child_env)?;
    Ok((Some(css), env))
}

/// 绑定参数到环境。
fn bind_params(mixin: &MixinDef, args: &[Arg], env: &Env) -> Result<Env> {
    let mut child_env = env.enter_scope();

    for (i, param) in mixin.params.iter().enumerate() {
        if param.rest {
            // 收集剩余参数
            let rest: Vec<Value> = args.iter().skip(i)
                .map(|a| eval_value(&a.value, env))
                .collect();
            child_env = child_env.define_var(&param.name, Value::ArgList(rest));
            break;
        }

        let value = if i < args.len() {
            eval_value(&args[i].value, env)
        } else if let Some(ref default) = param.default {
            eval_value(default, &child_env)
        } else {
            return Err(SassError::eval(format!("Missing argument for ${}", param.name)));
        };

        child_env = child_env.define_var(&param.name, value);
    }

    Ok(child_env)
}
