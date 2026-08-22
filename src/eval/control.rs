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
            let css = eval_nodes(body, child_env)?.0;
            return Ok((Some(css), env));
        }
    }
    if let Some(body) = else_body {
        let child_env = env.enter_scope();
        let css = eval_nodes(body, child_env)?.0;
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

    // 处理负方向（from > to 时递减）
    let (start, end, step) = if from_val <= to_val {
        (from_val, to_val, 1i64)
    } else {
        (from_val, to_val, -1i64)
    };

    let end_cond = if inclusive {
        end + step
    } else {
        end
    };

    let mut output = Vec::new();
    let env = env;

    let mut i = start;
    loop {
        if step > 0 && i >= end_cond { break; }
        if step < 0 && i <= end_cond { break; }

        // 循环变量定义在子作用域中
        let child_env = env.enter_scope().define_var(var, Value::Number(i as f64, None));
        let css = eval_nodes(body, child_env)?.0;
        output.extend(css);
        i += step;
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
    let evaluated = eval_value(list, &env);
    let items = match &evaluated {
        Value::List(items, _, _) => items.clone(),
        Value::ArgList(items) => items.clone(),
        Value::Map(pairs) => {
            // Map → (key, value) 对
            pairs.iter().flat_map(|(k, v)| vec![k.clone(), v.clone()]).collect()
        }
        v => vec![v.clone()],
    };

    let mut output = Vec::new();
    let env = env;

    for item in items {
        let child_env = env.enter_scope();
        // 多变量绑定
        if vars.len() == 1 {
            let child_env = child_env.define_var(&vars[0], item);
            let css = eval_nodes(body, child_env)?.0;
            output.extend(css);
        } else {
            // 解构：List → 多变量
            let parts = match &item {
                Value::List(items, _, _) => items.clone(),
                _ => vec![item],
            };
            let mut child_env = child_env;
            for (i, var) in vars.iter().enumerate() {
                let v = parts.get(i).cloned().unwrap_or(Value::Null);
                child_env = child_env.define_var(var, v);
            }
            let css = eval_nodes(body, child_env)?.0;
            output.extend(css);
        }
    }

    Ok((Some(output), env))
}

/// @while 求值。
pub fn eval_while(cond: &Value, body: &[Node], env: Env) -> Result<(Option<Vec<CssNode>>, Env)> {
    let mut output = Vec::new();
    let env = env;
    let mut iterations = 0;

    loop {
        let c = eval_value(cond, &env);
        if !c.is_truthy() { break; }

        iterations += 1;
        if iterations > 100_000 {
            tracing::warn!(iterations, "@while reached iteration limit");
            return Ok((Some(output), env));
        }

        let child_env = env.enter_scope();
        let css = eval_nodes(body, child_env)?.0;
        output.extend(css);
    }

    Ok((Some(output), env))
}
