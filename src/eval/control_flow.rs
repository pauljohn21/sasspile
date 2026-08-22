use super::*;
use crate::css::node::CssNode;
use crate::error::{Result, SassError};

impl Evaluator {
    pub(crate) fn eval_if(
        branches: &[(Value, Vec<Node>)],
        else_body: &Option<Vec<Node>>,
        env: Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        for (cond, body) in branches {
            let c = Self::eval_value(cond, &env)?;
            if Self::is_truthy(&c) {
                return Self::eval_nodes(body, env);
            }
        }
        if let Some(body) = else_body {
            Self::eval_nodes(body, env)
        } else {
            Ok((vec![], env))
        }
    }

    pub(crate) fn eval_for(
        var: &str,
        from: &Value,
        to: &Value,
        inclusive: bool,
        body: &[Node],
        env: Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = crate::__tracing::info_span!("eval_for", var = var, inclusive = inclusive);
        let _enter = span.enter();
        let from_val = Self::eval_value(from, &env)?;
        let to_val = Self::eval_value(to, &env)?;
        let loop_unit = match &from_val {
            Value::Number(_, u) => u.clone(),
            _ => None,
        };
        let (start, end) = match (&from_val, &to_val) {
            (Value::Number(s, su), Value::Number(e, eu)) => {
                if s.fract() != 0.0 {
                    return Err(SassError::Eval(format!("{s} is not an int.")));
                }
                let end_val = if su == eu || su.is_none() || eu.is_none() {
                    *e
                } else if crate::eval::value::units_compatible(su.as_deref(), eu.as_deref()) {
                    let s_u = su.as_deref().unwrap_or("");
                    let e_u = eu.as_deref().unwrap_or("");
                    let conv = unit_conversion_factor(e_u, s_u);
                    e * conv
                } else {
                    return Err(SassError::Eval(format!("@for incompatible units: {su:?} and {eu:?}")));
                };
                if end_val.fract() != 0.0 {
                    return Err(SassError::Eval(format!("{end_val} is not an int.")));
                }
                (*s as i64, end_val as i64)
            }
            (Value::String(s, _), _) => return Err(SassError::Eval(format!("\"{s}\" is not a number."))),
            (_, Value::String(s, _)) => return Err(SassError::Eval(format!("\"{s}\" is not a number."))),
            _ => return Err(SassError::Eval("@for range must be numbers".into())),
        };
        let mut css = Vec::new();
        let mut env = env;
        let step: i64 = if start <= end { 1 } else { -1 };
        let stop = if inclusive { end + step } else { end };
        let mut i = start;
        let mut count = 0i64;
        while i != stop {
            if count > MAX_DEPTH as i64 {
                return Err(SassError::Eval("@for loop iteration limit exceeded".into()));
            }
            env = env.bind(var.to_string(), Value::Number(i as f64, loop_unit.clone()));
            let (mut out, new_env) = Self::eval_nodes(body, env)?;
            css.append(&mut out);
            env = new_env;
            i += step;
            count += 1;
        }
        Ok((css, env))
    }

    pub(crate) fn eval_each(
        vars: &[String],
        list: &Value,
        body: &[Node],
        env: Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = crate::__tracing::info_span!("eval_each", n_vars = vars.len());
        let _enter = span.enter();
        let evaluated = Self::eval_value(list, &env)?;
        let items: Vec<Vec<Value>> = match &evaluated {
            Value::Map(pairs) if vars.len() >= 2 => pairs.iter().map(|(k, v)| vec![k.clone(), v.clone()]).collect(),
            Value::Map(pairs) if vars.len() == 1 => pairs.iter().map(|(k, v)| vec![Value::List(vec![k.clone(), v.clone()], Separator::Space, false)]).collect(),
            Value::List(es, _, _) => es.iter().map(|e| vec![e.clone()]).collect(),
            Value::Map(pairs) => pairs.iter().flat_map(|(k, v)| vec![vec![k.clone()], vec![v.clone()]]).collect(),
            other => vec![vec![other.clone()]],
        };
        let mut css = Vec::new();
        let mut env = env;
        for item_group in &items {
            if css.len() > 10000 {
                return Err(SassError::Eval("@each output node limit exceeded".into()));
            }
            if vars.len() == 1 {
                let val = item_group.first().cloned().unwrap_or(Value::Null);
                env = env.bind(vars[0].clone(), val);
            } else {
                for (j, v) in vars.iter().enumerate() {
                    let val = item_group.get(j).cloned().unwrap_or(Value::Null);
                    env = env.bind(v.clone(), val);
                }
            }
            let (mut out, new_env) = Self::eval_nodes(body, env)?;
            css.append(&mut out);
            env = new_env;
        }
        Ok((css, env))
    }

    pub(crate) fn eval_while(
        cond: &Value,
        body: &[Node],
        env: Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let mut css = Vec::new();
        let mut env = env;
        let mut iteration = 0;
        loop {
            iteration += 1;
            if iteration > MAX_DEPTH {
                crate::__tracing::error!(iteration, cond_ast = %cond, "@while 超过 MAX_DEPTH");
                return Err(SassError::Eval("@while 循环次数超过限制（可能是无限循环）".into()));
            }
            let c = Self::eval_value(cond, &env)?;
            let truthy = Self::is_truthy(&c);
            crate::__tracing::trace!(iteration, cond_value = %c, is_truthy = truthy, "@while 条件求值");
            if !truthy { break; }
            let (mut out, new_env) = Self::eval_nodes(body, env)?;
            css.append(&mut out);
            env = new_env;
            if css.len() > 10000 {
                return Err(SassError::Eval("@while output node limit exceeded".into()));
            }
        }
        Ok((css, env))
    }
}

fn unit_conversion_factor(from_unit: &str, to_unit: &str) -> f64 {
    fn to_mm(u: &str) -> Option<f64> {
        match u {
            "mm" => Some(1.0), "cm" => Some(10.0), "in" => Some(25.4),
            "pt" => Some(25.4 / 72.0), "pc" => Some(25.4 / 6.0), "px" => Some(25.4 / 96.0),
            "q" => Some(0.25), _ => None,
        }
    }
    match (to_mm(from_unit), to_mm(to_unit)) {
        (Some(f), Some(t)) => f / t,
        _ => 1.0,
    }
}
