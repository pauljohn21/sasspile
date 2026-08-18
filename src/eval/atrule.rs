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

    // Pre-evaluate all arguments in the *caller's* environment before
    // creating the mixin's child scope.  Same fix as call_user_function.
    let mut evaluated: Vec<(String, Value)> = Vec::new();
    for (i, param) in mixin.params.iter().enumerate() {
        if param.rest {
            let mut items = Vec::new();
            for arg in args.iter().skip(i) {
                items.push(expr::eval_expr(&arg.value, env, parent_sel)?);
            }
            evaluated.push((
                param.name.clone(),
                Value::List(crate::value::SassList::new(items, crate::ast::ListSeparator::Comma, false)),
            ));
            break;
        }

        let val = args.iter().find(|a| a.name.as_deref() == Some(param.name.as_str()))
            .or_else(|| args.get(i))
            .map(|a| &a.value);

        let value = if let Some(e) = val {
            expr::eval_expr(e, env, parent_sel)?
        } else if let Some(default) = &param.default {
            expr::eval_expr(default, env, parent_sel)?
        } else {
            Value::Null
        };
        evaluated.push((param.name.clone(), value));
    }

    // Create child env for mixin body
    let mut mixin_env = Env::new_child(std::mem::replace(env, Env::new_global()));
    for (name, value) in evaluated {
        mixin_env.set_var(name, value, false, false);
    }

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

/// Evaluate `@import "url"` — loads a file and injects its content
/// (CSS rules, variables, mixins, functions) into the current scope.
///
/// Unlike `@use`, `@import` does not create a namespace. All variables,
/// mixins, and functions from the imported file become available in
/// the current scope. CSS output is inserted at the import location.
///
/// File resolution follows the Sass spec:
/// 1. `url` as-is (if it's a plain CSS import starting with http://, https://, //)
/// 2. `base_dir/url` (exact path)
/// 3. `base_dir/url.scss`
/// 4. `base_dir/url.css`
/// 5. `base_dir/_url.scss`
pub fn eval_import_rule(
    url: &str,
    env: &mut Env,
    parent_sel: &[String],
    output: &mut Vec<super::CssRule>,
    extends: &mut Vec<super::ExtendEntry>,
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

    // Strip leading ./ or ../
    let rel = url.trim_start_matches("./").trim_start_matches("../");

    // Build candidates — underscore prefix goes on the last path component
    // e.g. "mixins/banner" → "mixins/_banner.scss", not "_mixins/banner.scss"
    let underscored = {
        let mut s = String::new();
        let parts: Vec<&str> = rel.rsplitn(2, '/').collect();
        if parts.len() == 2 {
            s.push_str(parts[1]);
            s.push('/');
            s.push('_');
            s.push_str(parts[0]);
        } else {
            s.push('_');
            s.push_str(parts[0]);
        }
        s
    };

    let candidates = [
        base_dir.join(rel),
        base_dir.join(format!("{}.scss", rel)),
        base_dir.join(format!("{}.css", rel)),
        base_dir.join(format!("{}.scss", underscored)),
        base_dir.join(format!("{}.css", underscored)),
    ];

    let file_path = candidates.iter().find(|p| p.is_file());

    if let Some(path) = file_path {
        let is_css = path.extension().and_then(|e| e.to_str()) == Some("css");
        let content = std::fs::read_to_string(path).map_err(|e| {
            SassError::parse(
                format!("Failed to read {}: {}", path.display(), e),
                crate::error::SourcePos { file: String::new(), line: 0, column: 0 },
            )
        })?;

        if is_css {
            // Plain CSS file — emit as raw @import
            output.push(super::CssRule::Raw(format!("@import \"{}\";", url)));
            tracing::debug!(stage = "eval", module = "import", url = %url, "CSS file import emitted as raw");
        } else {
            tracing::debug!(stage = "eval", module = "import", url = %url, path = %path.display(), "resolving SCSS import");

            let file_name = path.display().to_string();
            let tokens = crate::lexer::tokenize(&content, &file_name).map_err(|e| {
                SassError::parse(
                    format!("{}: {}", path.display(), e),
                    crate::error::SourcePos { file: file_name.clone(), line: 0, column: 0 },
                )
            })?;
            let ast = crate::parser::parse(tokens).map_err(|e| {
                SassError::parse(
                    format!("{}: {}", path.display(), e),
                    crate::error::SourcePos { file: path.display().to_string(), line: 0, column: 0 },
                )
            })?;

            // Save and update base_dir for nested imports
            let sub_dir = path.parent().map(|p| p.to_path_buf());
            let prev_dir = env.base_dir.take();
            if let Some(d) = sub_dir {
                env.base_dir = Some(d);
            }

            // @import injects content into the current scope
            // (variables, mixins, functions, and CSS rules)
            let sub_rules = eval_stmts(&ast, env, parent_sel, extends)?;
            env.base_dir = prev_dir;

            output.extend(sub_rules);
            tracing::info!(stage = "eval", module = "import", url = %url, rule_count = output.len(), "SCSS import resolved");
        }
    } else {
        tracing::warn!(stage = "eval", module = "import", url = %url, "file not found, skipping");
    }

    Ok(())
}
