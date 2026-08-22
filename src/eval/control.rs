//! 控制流求值：@if / @for / @each / @while。

use crate::error::Result;
use crate::parse::Node;
use crate::css::CssNode;
use crate::eval::value::Value;
use super::env::Env;
use super::{eval_nodes, eval_value};

/// @if 求值。
pub fn eval_if(
    branches: &[(Value, Vec<Node>)],
    else_body: Option<&[Node]>,
    env: Env,
) -> Result<(Option<Vec<CssNode>>, Env)> {
    for (cond, body) in branches {
        let c = eval_value(cond, &env);
        if c.is_truthy() {
            let child_env = env.enter_scope();
            let css = eval_nodes(body, child_env)?;
            return Ok((Some(css), env));
        }
    }
    if let Some(body) = else_body {
        let child_env = env.enter_scope();
        let css = eval_nodes(body, child_env)?;
        return Ok((Some(css), env));
    }
    Ok((None, env))
}

/// @for 求值。
pub fn eval_for(
    var: &str,
    from: &Value,
    to: &Value,
    inclusive: bool,
    body: &[Node],
    env: Env,
) -> Result<(Option<Vec<CssNode>>, Env)> {
    let from_val = match eval_value(from, &env) {
        Value::Number(n, _) => n as i64,
        _ => return Ok((None, env)),
    };
    let to_val = match eval_value(to, &env) {
        Value::Number(n, _) => n as i64,
        _ => return Ok((None, env)),
    };

    let end = if inclusive { to_val + 1 } else { to_val };
    let mut output = Vec::new();
    let mut env = env;

    for i in from_val..end {
        env = env.define_var(var, Value::Number(i as f64, None));
        let child_env = env.enter_scope();
        let css = eval_nodes(body, child_env)?;
        output.extend(css);
    }

    Ok((Some(output), env))
}

/// @each 求值。
pub fn eval_each(
    vars: &[String],
    list: &Value,
    body: &[Node],
    env: Env,
) -> Result<(Option<Vec<CssNode>>, Env)> {
    let items = match eval_value(list, &env) {
        Value::List(items, _, _) => items,
        Value::ArgList(items) => items,
        v => vec![v],
    };

    let mut output = Vec::new();
    let mut env = env;

    for item in items {
        // 多变量绑定
        if vars.len() == 1 {
            env = env.define_var(&vars[0], item);
        } else {
            let parts = match &item {
                Value::List(items, _, _) => items.clone(),
                _ => vec![item],
            };
            for (i, var) in vars.iter().enumerate() {
                let v = parts.get(i).cloned().unwrap_or(Value::Null);
                env = env.define_var(var, v);
            }
        }
        let child_env = env.enter_scope();
        let css = eval_nodes(body, child_env)?;
        output.extend(css);
    }

    Ok((Some(output), env))
}

/// @while 求值。
pub fn eval_while(cond: &Value, body: &[Node], env: Env) -> Result<(Option<Vec<CssNode>>, Env)> {
    let mut output = Vec::new();
    let mut env = env;
    let mut iterations = 0;

    loop {
        let c = eval_value(cond, &env);
        if !c.is_truthy() { break; }

        iterations += 1;
        if iterations > 100_000 {
            return Ok((Some(output), env));
        }

        let child_env = env.enter_scope();
        let css = eval_nodes(body, child_env)?;
        output.extend(css);
    }

    Ok((Some(output), env))
}
