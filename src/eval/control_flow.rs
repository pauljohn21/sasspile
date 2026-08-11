use super::*;
use crate::css::node::CssNode;
use crate::error::{Result, SassError};

impl Evaluator {
    pub(crate) fn eval_if(
        branches: &[(Value, Vec<Node>)],
        else_body: &Option<Vec<Node>>,
        env: &Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        for (cond, body) in branches {
            let c = Self::eval_value(cond, env)?;
            if Self::is_truthy(&c) {
                return Self::eval_nodes(body, env);
            }
        }
        if let Some(body) = else_body {
            Self::eval_nodes(body, env)
        } else {
            Ok((vec![], env.clone()))
        }
    }

    pub(crate) fn eval_for(
        var: &str,
        from: &Value,
        to: &Value,
        inclusive: bool,
        body: &[Node],
        env: &Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = tracing::info_span!("eval_for", var = var, inclusive = inclusive);
        let _enter = span.enter();
        let from_val = Self::eval_value(from, env)?;
        let to_val = Self::eval_value(to, env)?;
        let (start, end) = match (from_val, to_val) {
            (Value::Number(s, _), Value::Number(e, _)) => (s as i64, e as i64),
            _ => return Err(SassError::Eval("@for 范围必须是数字".into())),
        };
        let mut css = Vec::new();
        let mut current_env = env.clone();
        let step: i64 = if start <= end { 1 } else { -1 };
        let stop = if inclusive { end + step } else { end };
        let mut i = start;
        let mut count = 0i64;
        while i != stop {
            if count > MAX_DEPTH as i64 {
                return Err(SassError::Eval("@for 循环次数超过限制".into()));
            }
            current_env = current_env.bind(var.to_string(), Value::Number(i as f64, None));
            let (mut out, e) = Self::eval_nodes(body, &current_env)?;
            css.append(&mut out);
            current_env = e;
            i += step;
            count += 1;
        }
        Ok((css, current_env))
    }

    pub(crate) fn eval_each(
        vars: &[String],
        list: &Value,
        body: &[Node],
        env: &Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = tracing::info_span!("eval_each", n_vars = vars.len());
        let _enter = span.enter();
        let evaluated = Self::eval_value(list, env)?;
        // 对 Map，按 (key, value) 对迭代
        let items: Vec<Vec<Value>> = match &evaluated {
            Value::Map(pairs) if vars.len() >= 2 => pairs
                .iter()
                .map(|(k, v)| vec![k.clone(), v.clone()])
                .collect(),
            Value::Map(pairs) if vars.len() == 1 => {
                // 单变量遍历 Map：每对作为一个子列表
                pairs
                    .iter()
                    .map(|(k, v)| {
                        vec![Value::List(
                            vec![k.clone(), v.clone()],
                            Separator::Space,
                            false,
                        )]
                    })
                    .collect()
            }
            Value::List(es, _, _) => es.iter().map(|e| vec![e.clone()]).collect(),
            Value::Map(pairs) => pairs
                .iter()
                .flat_map(|(k, v)| vec![vec![k.clone()], vec![v.clone()]])
                .collect(),
            other => vec![vec![other.clone()]],
        };
        let mut css = Vec::new();
        let mut current_env = env.clone();
        for item_group in &items {
            if css.len() > 10000 {
                return Err(SassError::Eval("@each 输出节点过多".into()));
            }
            if vars.len() == 1 {
                let val = item_group.get(0).cloned().unwrap_or(Value::Null);
                current_env = current_env.bind(vars[0].clone(), val);
            } else {
                for (j, v) in vars.iter().enumerate() {
                    let val = item_group.get(j).cloned().unwrap_or(Value::Null);
                    current_env = current_env.bind(v.clone(), val);
                }
            }
            let (mut out, e) = Self::eval_nodes(body, &current_env)?;
            css.append(&mut out);
            current_env = e;
        }
        Ok((css, current_env))
    }

    pub(crate) fn eval_while(
        cond: &Value,
        body: &[Node],
        env: &Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let mut css = Vec::new();
        let mut current_env = env.clone();
        let mut iteration = 0;
        loop {
            iteration += 1;
            if iteration > MAX_DEPTH {
                tracing::error!(iteration, cond_ast = %cond, "@while 超过 MAX_DEPTH");
                return Err(SassError::Eval(
                    "@while 循环次数超过限制（可能是无限循环）".into(),
                ));
            }
            let c = Self::eval_value(cond, &current_env)?;
            let truthy = Self::is_truthy(&c);
            tracing::trace!(iteration, cond_value = %c, is_truthy = truthy, "@while 条件求值");
            if !truthy {
                break;
            }
            let (mut out, e) = Self::eval_nodes(body, &current_env)?;
            css.append(&mut out);
            current_env = e;
            // 限制 CSS 输出大小
            if css.len() > 10000 {
                return Err(SassError::Eval("@while 输出节点过多".into()));
            }
        }
        Ok((css, current_env))
    }
}
