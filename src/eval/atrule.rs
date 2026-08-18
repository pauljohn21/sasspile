//! At-rule evaluator — handles @if, @for, @each, @while, @include.

use crate::ast::*;
use crate::env::Env;
use crate::error::SassError;
use crate::resolver::ModuleResolver;
use crate::value::Value;
use super::eval_stmts;
use super::expr;
use super::func;
use super::ExtendEntry;
use super::ModuleCache;

/// Evaluate @if/@else if/@else
pub fn eval_if(
    branches: &[(Expr, Vec<Stmt>)],
    else_body: &Option<Vec<Stmt>>,
    env: &mut Env,
    parent_sel: &[String],
    output: &mut Vec<super::CssRule>,
    extends: &mut Vec<ExtendEntry>,
    resolver: &mut dyn ModuleResolver,
    module_cache: &mut ModuleCache,
) -> Result<(), SassError> {
    let span = tracing::debug_span!("eval_if", stage = "eval", module = "if");
    let _enter = span.enter();

    for (cond, body) in branches {
        let val = expr::eval_expr(cond, env, parent_sel, resolver)?;
        if val.is_truthy() {
            let rules = eval_stmts(body, env, parent_sel, extends, resolver, module_cache)?;
            output.extend(rules);
            return Ok(());
        }
    }
    if let Some(body) = else_body {
        let rules = eval_stmts(body, env, parent_sel, extends, resolver, module_cache)?;
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
    resolver: &mut dyn ModuleResolver,
    module_cache: &mut ModuleCache,
) -> Result<(), SassError> {
    let span = tracing::info_span!(
        "eval_for", stage = "eval", module = "for", var = %var, exclusive = exclusive
    );
    let _enter = span.enter();

    let from_val = expr::eval_expr(from, env, parent_sel, resolver)?;
    let to_val = expr::eval_expr(to, env, parent_sel, resolver)?;

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
        let rules = eval_stmts(body, env, parent_sel, extends, resolver, module_cache)?;
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
    resolver: &mut dyn ModuleResolver,
    module_cache: &mut ModuleCache,
) -> Result<(), SassError> {
    let span = tracing::info_span!(
        "eval_each", stage = "eval", module = "each", var_count = vars.len()
    );
    let _enter = span.enter();

    let list_val = expr::eval_expr(list_expr, env, parent_sel, resolver)?;
    let items: Vec<Value> = match &list_val {
        Value::List(l) => l.items.clone(),
        Value::Null => Vec::new(),
        Value::Map(m) => {
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
                    let rules = eval_stmts(body, env, parent_sel, extends, resolver, module_cache)?;
                    output.extend(rules);
                }
            }
            Value::List(l) if vars.len() > 1 && l.items.len() == vars.len() => {
                for (i, v) in l.items.iter().enumerate() {
                    env.set_var(vars[i].clone(), v.clone(), false, false);
                }
                let rules = eval_stmts(body, env, parent_sel, extends, resolver, module_cache)?;
                output.extend(rules);
            }
            _ => {
                env.set_var(vars[0].clone(), item.clone(), false, false);
                let rules = eval_stmts(body, env, parent_sel, extends, resolver, module_cache)?;
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
    resolver: &mut dyn ModuleResolver,
    module_cache: &mut ModuleCache,
) -> Result<(), SassError> {
    let span = tracing::info_span!("eval_while", stage = "eval", module = "while");
    let _enter = span.enter();

    let mut iterations = 0;
    loop {
        let val = expr::eval_expr(cond, env, parent_sel, resolver)?;
        if !val.is_truthy() {
            break;
        }
        let rules = eval_stmts(body, env, parent_sel, extends, resolver, module_cache)?;
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
    resolver: &mut dyn ModuleResolver,
    module_cache: &mut ModuleCache,
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

    // Pre-evaluate all arguments in the *caller's* environment before
    // creating the mixin's child scope.  Same fix as call_user_function.
    // Also expand spread args ($val...) — maps become named args,
    // lists become positional args.
    let expanded = func::expand_spread_args(args, env, parent_sel, resolver)?;

    // Separate into named and positional args
    let mut named: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    let mut positional: Vec<Value> = Vec::new();
    for (name, val) in expanded {
        if let Some(n) = name {
            named.insert(n, val);
        } else {
            positional.push(val);
        }
    }

    let mut evaluated: Vec<(String, Value)> = Vec::new();
    let mut pos_idx = 0;
    for (_i, param) in mixin.params.iter().enumerate() {
        if param.rest {
            let mut items = Vec::new();
            while pos_idx < positional.len() {
                items.push(positional[pos_idx].clone());
                pos_idx += 1;
            }
            evaluated.push((
                param.name.clone(),
                Value::List(crate::value::SassList::new(items, crate::ast::ListSeparator::Comma, false)),
            ));
            break;
        }

        // Try named match first, then positional
        let value = if let Some(v) = named.get(&param.name) {
            v.clone()
        } else if pos_idx < positional.len() {
            let v = positional[pos_idx].clone();
            pos_idx += 1;
            v
        } else if let Some(default) = &param.default {
            expr::eval_expr(default, env, parent_sel, resolver)?
        } else {
            Value::Null
        };
        evaluated.push((param.name.clone(), value));
    }

    // Create child env for mixin body using with_child_scope
    env.with_child_scope(|mixin_env| -> Result<(), SassError> {
        for (name, value) in evaluated {
            mixin_env.set_var(name, value, false, false);
        }

        // Store content block in env so @content can access it
        if let Some(content_stmts) = content {
            mixin_env.set_content(content_stmts.to_vec());
        }

        // Evaluate mixin body — @content blocks will be handled by eval_stmt
        let rules = eval_stmts(&mixin.body, mixin_env, parent_sel, extends, resolver, module_cache)?;
        output.extend(rules);

        Ok(())
    })
}

/// Evaluate `@import "url"` — loads a file and injects its content
/// (CSS rules, variables, mixins, functions) into the current scope.
///
/// Unlike `@use`, `@import` does not create a namespace. All variables,
/// mixins, and functions from the imported file become available in
/// the current scope. CSS output is inserted at the import location.
///
/// File resolution is delegated to the `ModuleResolver`.
pub fn eval_import_rule(
    url: &str,
    env: &mut Env,
    parent_sel: &[String],
    output: &mut Vec<super::CssRule>,
    extends: &mut Vec<super::ExtendEntry>,
    resolver: &mut dyn ModuleResolver,
    module_cache: &mut ModuleCache,
) -> Result<(), SassError> {
    let span = tracing::info_span!(
        "eval_import",
        stage = "eval",
        module = "import",
        url = %url
    );
    let _enter = span.enter();

    // Plain CSS imports (http://, https://, //) — emit as raw @import
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("//") {
        output.push(super::CssRule::AtRule {
            name: "import".to_string(),
            value: format!("\"{}\"", url),
            body: Vec::new(),
        });
        tracing::debug!(stage = "eval", module = "import", url = %url, "plain CSS import emitted");
        return Ok(());
    }

    // Resolve relative to base_dir on the filesystem
    let base_dir = match env.get_base_dir() {
        Some(d) => d.clone(),
        None => {
            tracing::warn!(stage = "eval", module = "import", url = %url, "no base_dir, cannot resolve import");
            return Ok(());
        }
    };

    // Use the resolver to load the module
    let module = match resolver.resolve(url, &base_dir) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!(stage = "eval", module = "import", url = %url, error = %e, "file not found, skipping");
            return Ok(());
        }
    };

    if module.is_css {
        // Plain CSS file — emit as raw @import
        output.push(super::CssRule::Raw(format!("@import \"{}\";", url)));
        tracing::debug!(stage = "eval", module = "import", url = %url, "CSS file import emitted as raw");
    } else {
        tracing::debug!(stage = "eval", module = "import", url = %url, path = %module.source_path.display(), "resolving SCSS import");

        // Save and update base_dir for nested imports
        let sub_dir = module.source_path.parent().map(|p| p.to_path_buf());
        let prev_dir = env.base_dir.take();
        if let Some(d) = sub_dir {
            env.base_dir = Some(d);
        }

        // @import injects content into the current scope
        // (variables, mixins, functions, and CSS rules)
        let sub_rules = eval_stmts(&module.ast, env, parent_sel, extends, resolver, module_cache)?;
        env.base_dir = prev_dir;

        output.extend(sub_rules);
        tracing::info!(stage = "eval", module = "import", url = %url, rule_count = output.len(), "SCSS import resolved");
    }

    Ok(())
}
