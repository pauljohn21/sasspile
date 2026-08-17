//! Evaluator — traverses AST and produces CSS output tree.

pub mod expr;
pub mod atrule;
pub mod interp;
pub mod css;

use crate::ast::*;
use crate::env::Env;
use crate::error::SassError;
use tracing::instrument;

pub use css::{value_to_css, calc_arg_to_css};
pub use interp::{eval_interpolation_in_str, resolve_selector};

/// The CSS output tree.
#[derive(Debug, Clone)]
pub struct CssTree {
    pub rules: Vec<CssRule>,
    /// Extend requests collected during evaluation.
    pub extends: Vec<ExtendEntry>,
}

/// A single @extend request entry.
#[derive(Debug, Clone)]
pub struct ExtendEntry {
    pub extender: String,
    pub extendee: String,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub enum CssRule {
    Style {
        selector: String,
        declarations: Vec<(String, String)>,
        nested: Vec<CssRule>,
    },
    AtRule {
        name: String,
        value: String,
        body: Vec<CssRule>,
    },
    Comment(String),
    /// Raw CSS text (from `@use` of a `.css` file).
    Raw(String),
}

/// Evaluate AST statements into a CSS output tree.
#[instrument(name = "evaluate", skip_all, fields(stage = "eval"))]
pub fn evaluate(stmts: Vec<Stmt>) -> Result<CssTree, SassError> {
    let span = tracing::info_span!("evaluate", stage = "eval", stmt_count = stmts.len());
    let _enter = span.enter();

    let mut env = Env::new_global();
    crate::builtins::register_all(&mut env);
    let parent_sel: Vec<String> = Vec::new();
    let mut extends = Vec::new();
    let rules = eval_stmts(&stmts, &mut env, &parent_sel, &mut extends)?;

    tracing::debug!(stage = "eval", rule_count = rules.len(), extend_count = extends.len(), "evaluation complete");
    Ok(CssTree { rules, extends })
}

/// Evaluate AST statements with a virtual file system.
#[instrument(name = "evaluate_with_vfs", skip_all, fields(stage = "eval"))]
pub fn evaluate_with_vfs(
    stmts: Vec<Stmt>,
    vfs: &std::collections::HashMap<String, String>,
) -> Result<CssTree, SassError> {
    let span = tracing::info_span!("evaluate", stage = "eval", stmt_count = stmts.len());
    let _enter = span.enter();

    let mut env = Env::new_global();
    crate::builtins::register_all(&mut env);
    for (name, content) in vfs {
        env.set_vfs_file(name.clone(), content.clone());
    }
    let parent_sel: Vec<String> = Vec::new();
    let mut extends = Vec::new();
    let rules = eval_stmts(&stmts, &mut env, &parent_sel, &mut extends)?;

    tracing::debug!(stage = "eval", rule_count = rules.len(), extend_count = extends.len(), "evaluation complete");
    Ok(CssTree { rules, extends })
}

pub fn eval_stmts(
    stmts: &[Stmt],
    env: &mut Env,
    parent_sel: &[String],
    extends: &mut Vec<ExtendEntry>,
) -> Result<Vec<CssRule>, SassError> {
    let span = tracing::debug_span!("eval_stmts", stage = "eval", stmt_count = stmts.len());
    let _enter = span.enter();
    let mut rules = Vec::new();
    for stmt in stmts {
        eval_stmt(stmt, env, parent_sel, &mut rules, extends)?;
    }
    Ok(rules)
}

pub(crate) fn eval_stmt(
    stmt: &Stmt,
    env: &mut Env,
    parent_sel: &[String],
    output: &mut Vec<CssRule>,
    extends: &mut Vec<ExtendEntry>,
) -> Result<(), SassError> {
    let span = tracing::trace_span!("eval_stmt", stage = "eval", node = ?stmt.node_name());
    let _enter = span.enter();

    match stmt {
        Stmt::StyleRule { selector, body } => {
            let span = tracing::debug_span!("eval_style_rule", stage = "eval", module = "style", selector = %selector);
            let _enter = span.enter();

            let eval_sel = eval_interpolation_in_str(selector, env, parent_sel)?;
            let resolved = resolve_selector(&eval_sel, parent_sel);
            let mut new_parent = parent_sel.to_vec();
            new_parent.push(resolved.clone());
            let nested = eval_stmts(body, env, &new_parent, extends)?;
            let mut declarations = Vec::new();
            let mut nest_rules = Vec::new();
            for r in nested {
                match r {
                    CssRule::Style { selector: s, declarations: d, nested: n } => {
                        if s.is_empty() {
                            declarations.extend(d);
                            nest_rules.extend(n);
                        } else {
                            nest_rules.push(CssRule::Style { selector: s, declarations: d, nested: n });
                        }
                    }
                    other => nest_rules.push(other),
                }
            }
            if !declarations.is_empty() || !nest_rules.is_empty() {
                output.push(CssRule::Style {
                    selector: resolved,
                    declarations,
                    nested: nest_rules,
                });
            }
        }
        Stmt::Declaration { property, value } => {
            let span = tracing::trace_span!("eval_declaration", stage = "eval", module = "decl", property = %property);
            let _enter = span.enter();

            let val = expr::eval_expr(value, env, parent_sel)?;
            let css_val = value_to_css(&val);
            let eval_prop = eval_interpolation_in_str(property, env, parent_sel)?;
            if !css_val.is_empty() {
                output.push(CssRule::Style {
                    selector: String::new(),
                    declarations: vec![(eval_prop, css_val)],
                    nested: Vec::new(),
                });
            }
        }
        Stmt::VariableDecl { name, value, default, global } => {
            let span = tracing::trace_span!("eval_var_decl", stage = "eval", module = "var", name = %name);
            let _enter = span.enter();

            let val = expr::eval_expr(value, env, parent_sel)?;
            env.set_var(name.clone(), val, *global, *default);
        }
        Stmt::Comment(text) => {
            output.push(CssRule::Comment(text.clone()));
        }
        Stmt::IfStmt { branches, else_body } => {
            atrule::eval_if(branches, else_body, env, parent_sel, output, extends)?;
        }
        Stmt::ForStmt { var, from, to, exclusive, body } => {
            atrule::eval_for(var, from, to, *exclusive, body, env, parent_sel, output, extends)?;
        }
        Stmt::EachStmt { vars, list, body } => {
            atrule::eval_each(vars, list, body, env, parent_sel, output, extends)?;
        }
        Stmt::WhileStmt { cond, body } => {
            atrule::eval_while(cond, body, env, parent_sel, output, extends)?;
        }
        Stmt::MixinDef { name, params, body } => {
            let span = tracing::debug_span!("eval_mixin_def", stage = "eval", module = "mixin", name = %name);
            let _enter = span.enter();

            env.set_mixin(name.clone(), crate::env::Mixin {
                params: params.clone(),
                body: body.clone(),
            });
        }
        Stmt::FunctionDef { name, params, body } => {
            let span = tracing::debug_span!("eval_func_def", stage = "eval", module = "function", name = %name);
            let _enter = span.enter();

            env.set_function(name.clone(), crate::env::UserFunction {
                params: params.clone(),
                body: body.clone(),
            });
        }
        Stmt::IncludeCall { name, args, content } => {
            atrule::eval_include(name, args, content.as_deref(), env, parent_sel, output, extends)?;
        }
        Stmt::ErrorStmt(expr) => {
            let val = expr::eval_expr(expr, env, parent_sel)?;
            return Err(SassError::user_error(val.to_string(), crate::error::SourcePos::default()));
        }
        Stmt::WarnStmt(expr) => {
            let val = expr::eval_expr(expr, env, parent_sel)?;
            tracing::warn!(stage = "eval", module = "warn", msg = %val, "@warn");
        }
        Stmt::DebugStmt(expr) => {
            let val = expr::eval_expr(expr, env, parent_sel)?;
            tracing::debug!(stage = "eval", module = "debug", msg = %val, "@debug");
        }
        Stmt::MediaRule { query, body } => {
            let span = tracing::debug_span!("eval_media", stage = "eval", module = "media", query = %query);
            let _enter = span.enter();

            let eval_query = eval_interpolation_in_str(query, env, parent_sel)?;
            let body_rules = eval_stmts(body, env, parent_sel, extends)?;
            output.push(CssRule::AtRule {
                name: "media".to_string(),
                value: eval_query,
                body: body_rules,
            });
        }
        Stmt::AtRootRule(body) => {
            let span = tracing::debug_span!("eval_at_root", stage = "eval", module = "at-root");
            let _enter = span.enter();

            let rules = eval_stmts(body, env, &[], extends)?;
            output.extend(rules);
        }
        Stmt::SupportsRule { condition, body } => {
            let span = tracing::debug_span!("eval_supports", stage = "eval", module = "supports");
            let _enter = span.enter();

            let eval_cond = eval_interpolation_in_str(condition, env, parent_sel)?;
            let body_rules = eval_stmts(body, env, parent_sel, extends)?;
            output.push(CssRule::AtRule {
                name: "supports".to_string(),
                value: eval_cond,
                body: body_rules,
            });
        }
        Stmt::CssAtRule { name, value, body } => {
            let eval_val = eval_interpolation_in_str(value, env, parent_sel)?;
            let body_rules = if let Some(b) = body {
                eval_stmts(b, env, parent_sel, extends)?
            } else {
                Vec::new()
            };
            output.push(CssRule::AtRule {
                name: name.clone(),
                value: eval_val,
                body: body_rules,
            });
        }
        Stmt::ReturnStmt(_) => {}
        Stmt::ContentRule => {
            let span = tracing::trace_span!("eval_content", stage = "eval", module = "content");
            let _enter = span.enter();

            if let Some(content_stmts) = env.get_content() {
                let content_stmts = content_stmts.to_vec();
                let rules = eval_stmts(&content_stmts, env, parent_sel, extends)?;
                output.extend(rules);
            }
        }
        Stmt::ExtendRule { selector, optional } => {
            let span = tracing::debug_span!("eval_extend", stage = "eval", module = "extend", selector = %selector);
            let _enter = span.enter();

            let extender = parent_sel.last().cloned().unwrap_or_default();
            extends.push(ExtendEntry {
                extender,
                extendee: selector.clone(),
                optional: *optional,
            });
            tracing::trace!(stage = "eval", module = "extend", extendee = %selector, "extend recorded");
        }
        Stmt::UseRule { url, namespace, config } => {
            let span = tracing::debug_span!("eval_use", stage = "eval", module = "use", url = %url);
            let _enter = span.enter();
            eval_use_rule(url, namespace.as_deref(), config, env, output)?;
        }
        Stmt::ForwardRule { .. } => {
            tracing::trace!(stage = "eval", module = "forward", "forward not yet fully implemented");
        }
        Stmt::ImportRule(_) => {
            tracing::trace!(stage = "eval", module = "import", "import not yet fully implemented");
        }
    }
    Ok(())
}

/// Evaluate `@use` rule — loads a module and registers it in the environment.
/// For built-in modules (`sass:math`, `sass:string`, etc.), registers the
/// module's functions and variables under a namespace.
/// For external files, loads from VFS if available.
fn eval_use_rule(
    url: &str,
    namespace: Option<&str>,
    _config: &[(String, Expr)],
    env: &mut Env,
    output: &mut Vec<CssRule>,
) -> Result<(), SassError> {
    // Built-in sass: modules
    if url.starts_with("sass:") {
        let module_name = url.strip_prefix("sass:").unwrap_or(url);
        let ns = namespace.unwrap_or(module_name).to_string();

        let span = tracing::info_span!("load_module", stage = "eval", module = "use", url = %url, ns = %ns);
        let _enter = span.enter();

        let module = crate::env::ModuleEnv::new();
        env.set_module(ns, module);

        tracing::debug!(stage = "eval", module = "use", url = %url, "builtin module loaded");
        return Ok(());
    }

    // Try VFS resolution
    let module_name = url.trim_start_matches("./").trim_start_matches("../");
    let base_name = module_name.rsplit('/').next().unwrap_or(module_name);

    let span = tracing::debug_span!("eval_use", stage = "eval", module = "use", url = %url, key = %base_name);
    let _enter = span.enter();

    // Try to find the file in VFS.
    // First try with the base name as-is (may include extension).
    // Then try common extensions: .scss, .css, _prefix.scss
    let content = env
        .get_vfs_file(base_name)
        .or_else(|| env.get_vfs_file(&format!("{}.scss", base_name)))
        .or_else(|| env.get_vfs_file(&format!("{}.css", base_name)))
        .or_else(|| env.get_vfs_file(&format!("_{}.scss", base_name)))
        .map(|s| s.to_string());

    // Determine if the resolved file is a .css file
    let resolved_name = if env.get_vfs_file(base_name).is_some() {
        base_name.to_string()
    } else if env.get_vfs_file(&format!("{}.css", base_name)).is_some() {
        format!("{}.css", base_name)
    } else if env.get_vfs_file(&format!("{}.scss", base_name)).is_some() {
        format!("{}.scss", base_name)
    } else if env.get_vfs_file(&format!("_{}.scss", base_name)).is_some() {
        format!("_{}.scss", base_name)
    } else {
        base_name.to_string()
    };
    let is_css = resolved_name.ends_with(".css");

    if let Some(content) = content {
        if is_css {
            // Plain CSS files are output as-is, not evaluated through the Sass pipeline
            output.push(CssRule::Raw(content));
            tracing::debug!(stage = "eval", module = "use", url = %url, "vfs CSS module loaded as raw");
        } else {
            // SCSS files: tokenize, parse, and evaluate
            let tokens = crate::lexer::tokenize(&content)?;
            let ast = crate::parser::parse(tokens)?;
            let sub_rules = eval_stmts(&ast, env, &[], &mut Vec::new())?;
            output.extend(sub_rules);
            tracing::debug!(stage = "eval", module = "use", url = %url, "vfs SCSS module loaded");
        }
    } else {
        tracing::debug!(stage = "eval", module = "use", url = %url, "external module skipped (not in VFS)");
    }

    Ok(())
}
