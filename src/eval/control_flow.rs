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
        let span = crate::__tracing::info_span!("eval_for", var = var, inclusive = inclusive);
        let _enter = span.enter();
        let from_val = Self::eval_value(from, env)?;
        let to_val = Self::eval_value(to, env)?;
        // 循环变量使用 from 的单位（Sass 语义：$i 从 from 开始，带 from 的单位）
        let loop_unit = match &from_val {
            Value::Number(_, u) => u.clone(),
            _ => None,
        };
        let (start, end) = match (&from_val, &to_val) {
            (Value::Number(s, su), Value::Number(e, eu)) => {
                // 检查 from 和 to 是否为整数
                if s.fract() != 0.0 {
                    return Err(SassError::Eval(format!("{s} is not an int.")));
                }
                // 如果单位不同但兼容，将 to 转换为 from 的单位
                let end_val = if su == eu || su.is_none() || eu.is_none() {
                    *e
                } else if crate::eval::value::units_compatible(su.as_deref(), eu.as_deref()) {
                    // 简单单位转换：尝试常见长度单位
                    let s_u = su.as_deref().unwrap_or("");
                    let e_u = eu.as_deref().unwrap_or("");
                    let conv = unit_conversion_factor(e_u, s_u);
                    e * conv
                } else {
                    return Err(SassError::Eval(format!(
                        "@for incompatible units: {su:?} and {eu:?}"
                    )));
                };
                if end_val.fract() != 0.0 {
                    return Err(SassError::Eval(format!("{end_val} is not an int.")));
                }
                (*s as i64, end_val as i64)
            }
            (Value::String(s, _), _) => {
                return Err(SassError::Eval(format!("\"{s}\" is not a number.")));
            }
            (_, Value::String(s, _)) => {
                return Err(SassError::Eval(format!("\"{s}\" is not a number.")));
            }
            _ => return Err(SassError::Eval("@for range must be numbers".into())),
        };
        let mut css = Vec::new();
        let mut current_env = env.clone();
        let step: i64 = if start <= end { 1 } else { -1 };
        let stop = if inclusive { end + step } else { end };
        let mut i = start;
        let mut count = 0i64;
        while i != stop {
            if count > MAX_DEPTH as i64 {
                return Err(SassError::Eval("@for loop iteration limit exceeded".into()));
            }
            current_env = current_env.bind(var.to_string(), Value::Number(i as f64, loop_unit.clone()));
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
        let span = crate::__tracing::info_span!("eval_each", n_vars = vars.len());
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
                return Err(SassError::Eval("@each output node limit exceeded".into()));
            }
            if vars.len() == 1 {
                let val = item_group.first().cloned().unwrap_or(Value::Null);
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
                crate::__tracing::error!(iteration, cond_ast = %cond, "@while 超过 MAX_DEPTH");
                return Err(SassError::Eval(
                    "@while 循环次数超过限制（可能是无限循环）".into(),
                ));
            }
            let c = Self::eval_value(cond, &current_env)?;
            let truthy = Self::is_truthy(&c);
            crate::__tracing::trace!(iteration, cond_value = %c, is_truthy = truthy, "@while 条件求值");
            if !truthy {
                break;
            }
            let (mut out, e) = Self::eval_nodes(body, &current_env)?;
            css.append(&mut out);
            current_env = e;
            // 限制 CSS 输出大小
            if css.len() > 10000 {
                return Err(SassError::Eval("@while output node limit exceeded".into()));
            }
        }
        Ok((css, current_env))
    }
}

/// 单位转换因子——将 from_unit 转换为 to_unit 的倍数。
/// 仅支持常见长度单位，不支持的返回 1.0。
fn unit_conversion_factor(from_unit: &str, to_unit: &str) -> f64 {
    /// 将单位转换为基准单位（mm）的倍数
    fn to_mm(u: &str) -> Option<f64> {
        match u {
            "mm" => Some(1.0),
            "cm" => Some(10.0),
            "in" => Some(25.4),
            "pt" => Some(25.4 / 72.0),
            "pc" => Some(25.4 / 6.0),
            "px" => Some(25.4 / 96.0),
            "q" => Some(0.25),
            _ => None,
        }
    }
    match (to_mm(from_unit), to_mm(to_unit)) {
        (Some(f), Some(t)) => f / t,
        _ => 1.0,
    }
}
