//! At-rule evaluator — handles @if, @for, @each, @while, @include.

use crate::ast::*;
use crate::env::Env;
use crate::error::SassError;
use crate::value::Value;
use super::eval_stmts;
use super::expr;
use super::ExtendEntry;

/// Evaluate @if/@else if/@else
pub fn eval_if(
    branches: &[(Expr, Vec<Stmt>)],
    else_body: &Option<Vec<Stmt>>,
    env: &mut Env,
    parent_sel: &[String],
    output: &mut Vec<super::CssRule>,
    extends: &mut Vec<ExtendEntry>,
) -> Result<(), SassError> {
    let span = tracing::debug_span!("eval_if", stage = "eval", module = "if");
    let _enter = span.enter();

    for (cond, body) in branches {
        let val = expr::eval_expr(cond, env, parent_sel)?;
        if val.is_truthy() {
            let rules = eval_stmts(body, env, parent_sel, extends)?;
            output.extend(rules);
            return Ok(());
        }
    }
    if let Some(body) = else_body {
        let rules = eval_stmts(body, env, parent_sel, extends)?;
        output.extend(rules);
    }
    Ok(())
}

/// Evaluate @for $var from start through/to end { ... }
pub fn eval_for(
    var: &str,
    from: &Expr,
    to: &Expr,
    exclusive: bool,
    body: &[Stmt],
    env: &mut Env,
    parent_sel: &[String],
    output: &mut Vec<super::CssRule>,
    extends: &mut Vec<ExtendEntry>,
) -> Result<(), SassError> {
    let span = tracing::info_span!(
        "eval_for", stage = "eval", module = "for", var = %var, exclusive = exclusive
    );
    let _enter = span.enter();

    let from_val = expr::eval_expr(from, env, parent_sel)?;
    let to_val = expr::eval_expr(to, env, parent_sel)?;

    let start = match &from_val {
        Value::Number(n) => n.value as i64,
        _ => return Err(SassError::eval("@for range must be numbers", crate::error::SourcePos::default())),
    };
    let end = match &to_val {
        Value::Number(n) => n.value as i64,
        _ => return Err(SassError::eval("@for range must be numbers", crate::error::SourcePos::default())),
    };

    let end_actual = if exclusive { end } else { end + 1 };
    let count = (end_actual - start).max(0);
    tracing::debug!(stage = "eval", module = "for", iterations = count, "for loop range");

    for i in start..end_actual {
        env.set_var(var.to_string(), Value::Number(crate::value::Number::unitless(i as f64)), false, false);
        let rules = eval_stmts(body, env, parent_sel, extends)?;
        output.extend(rules);
    }
    Ok(())
}

/// Evaluate @each $vars in list { ... }
pub fn eval_each(
    vars: &[String],
    list_expr: &Expr,
    body: &[Stmt],
    env: &mut Env,
    parent_sel: &[String],
    output: &mut Vec<super::CssRule>,
    extends: &mut Vec<ExtendEntry>,
) -> Result<(), SassError> {
    let span = tracing::info_span!(
        "eval_each", stage = "eval", module = "each", var_count = vars.len()
    );
    let _enter = span.enter();

    let list_val = expr::eval_expr(list_expr, env, parent_sel)?;
    let items: Vec<Value> = match &list_val {
        Value::List(l) => l.items.clone(),
        Value::Null => Vec::new(),
        Value::Map(m) => {
            // For maps, iterate as key-value pairs
            let mut pairs = Vec::new();
            for (k, v) in &m.entries {
                pairs.push(Value::List(crate::value::SassList::new(
                    vec![k.clone(), v.clone()],
                    crate::ast::ListSeparator::Space,
                    false,
                )));
            }
            pairs
        }
        other => vec![other.clone()],
    };

    tracing::debug!(stage = "eval", module = "each", item_count = items.len(), "each iteration count");

    for item in items {
        match &item {
            Value::Map(m) if vars.len() == 2 => {
                for (k, v) in &m.entries {
                    env.set_var(vars[0].clone(), k.clone(), false, false);
                    env.set_var(vars[1].clone(), v.clone(), false, false);
                    let rules = eval_stmts(body, env, parent_sel, extends)?;
                    output.extend(rules);
                }
            }
            Value::List(l) if vars.len() > 1 && l.items.len() == vars.len() => {
                for (i, v) in l.items.iter().enumerate() {
                    env.set_var(vars[i].clone(), v.clone(), false, false);
                }
                let rules = eval_stmts(body, env, parent_sel, extends)?;
                output.extend(rules);
            }
            _ => {
                env.set_var(vars[0].clone(), item.clone(), false, false);
                let rules = eval_stmts(body, env, parent_sel, extends)?;
                output.extend(rules);
            }
        }
    }
    Ok(())
}

/// Evaluate @while cond { ... }
pub fn eval_while(
    cond: &Expr,
    body: &[Stmt],
    env: &mut Env,
    parent_sel: &[String],
    output: &mut Vec<super::CssRule>,
    extends: &mut Vec<ExtendEntry>,
) -> Result<(), SassError> {
    let span = tracing::info_span!("eval_while", stage = "eval", module = "while");
    let _enter = span.enter();

    let mut iterations = 0;
    loop {
        let val = expr::eval_expr(cond, env, parent_sel)?;
        if !val.is_truthy() {
            break;
        }
        let rules = eval_stmts(body, env, parent_sel, extends)?;
        output.extend(rules);

        iterations += 1;
        if iterations > 100000 {
            return Err(SassError::eval("@while loop limit exceeded", crate::error::SourcePos::default()));
        }
    }
    tracing::debug!(stage = "eval", module = "while", iterations, "while loop complete");
    Ok(())
}

/// Evaluate @include mixin_name(args) { @content }
pub fn eval_include(
    name: &str,
    args: &[Arg],
    content: Option<&[Stmt]>,
    env: &mut Env,
    parent_sel: &[String],
    output: &mut Vec<super::CssRule>,
    extends: &mut Vec<ExtendEntry>,
) -> Result<(), SassError> {
    let span = tracing::info_span!(
        "eval_include", stage = "eval", module = "include", name = %name
    );
    let _enter = span.enter();

    let mixin = env.get_mixin(name).cloned();
    let mixin = match mixin {
        Some(m) => m,
        None => return Err(SassError::eval(
            format!("Undefined mixin: {}", name),
            crate::error::SourcePos::default(),
        )),
    };

    // Create child env for mixin body
    let mut mixin_env = Env::new_child(std::mem::replace(env, Env::new_global()));
    expr::bind_params(&mixin.params, args, &mut mixin_env, env, parent_sel)?;

    // Store content block in env so @content can access it
    if let Some(content_stmts) = content {
        mixin_env.set_content(content_stmts.to_vec());
    }

    // Evaluate mixin body — @content blocks will be handled by eval_stmt
    let rules = eval_stmts(&mixin.body, &mut mixin_env, parent_sel, extends)?;
    output.extend(rules);

    // Restore env
    *env = *mixin_env.parent.take().unwrap();
    Ok(())
}
