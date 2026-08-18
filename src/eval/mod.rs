//! Evaluator — traverses AST and produces CSS output tree.

pub mod expr;
pub mod func;
pub mod atrule;
pub mod interp;
pub mod css;
pub mod module_cache;

use crate::ast::*;
use crate::env::Env;
use crate::error::SassError;
use crate::resolver::ModuleResolver;
use tracing::instrument;
use std::collections::HashMap;

pub use css::{value_to_css, calc_arg_to_css};
pub use interp::{eval_interpolation_in_str, resolve_selector};
pub use module_cache::{ModuleCache, EvaluatedModule};

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
///
/// `resolver` handles module loading for `@use` and `@import`.
#[instrument(name = "evaluate", skip_all, fields(stage = "eval"))]
pub fn evaluate(stmts: Vec<Stmt>, resolver: &mut dyn ModuleResolver) -> Result<CssTree, SassError> {
    let span = tracing::info_span!("evaluate", stage = "eval", stmt_count = stmts.len());
    let _enter = span.enter();

    let mut env = Env::new_global();
    crate::builtins::register_all(&mut env);
    let parent_sel: Vec<String> = Vec::new();
    let mut extends = Vec::new();
    let mut module_cache = ModuleCache::new();
    let rules = eval_stmts(&stmts, &mut env, &parent_sel, &mut extends, resolver, &mut module_cache)?;

    tracing::debug!(stage = "eval", rule_count = rules.len(), extend_count = extends.len(), "evaluation complete");
    Ok(CssTree { rules, extends })
}

/// Evaluate AST statements with a base directory for @use resolution.
///
/// `resolver` handles module loading for `@use` and `@import`.
#[instrument(name = "evaluate_with_dir", skip_all, fields(stage = "eval"))]
pub fn evaluate_with_dir(
    stmts: Vec<Stmt>,
    base_dir: std::path::PathBuf,
    resolver: &mut dyn ModuleResolver,
) -> Result<CssTree, SassError> {
    let span = tracing::info_span!("evaluate", stage = "eval", stmt_count = stmts.len());
    let _enter = span.enter();

    let mut env = Env::new_global();
    env.base_dir = Some(base_dir);
    crate::builtins::register_all(&mut env);
    let parent_sel: Vec<String> = Vec::new();
    let mut extends = Vec::new();
    let mut module_cache = ModuleCache::new();
    let rules = eval_stmts(&stmts, &mut env, &parent_sel, &mut extends, resolver, &mut module_cache)?;

    tracing::debug!(stage = "eval", rule_count = rules.len(), extend_count = extends.len(), "evaluation complete");
    Ok(CssTree { rules, extends })
}

pub fn eval_stmts(
    stmts: &[Stmt],
    env: &mut Env,
    parent_sel: &[String],
    extends: &mut Vec<ExtendEntry>,
    resolver: &mut dyn ModuleResolver,
    module_cache: &mut ModuleCache,
) -> Result<Vec<CssRule>, SassError> {
    let span = tracing::debug_span!("eval_stmts", stage = "eval", stmt_count = stmts.len());
    let _enter = span.enter();
    let mut rules = Vec::new();
    for stmt in stmts {
        eval_stmt(stmt, env, parent_sel, &mut rules, extends, resolver, module_cache)?;
    }
    Ok(rules)
}

pub(crate) fn eval_stmt(
    stmt: &Stmt,
    env: &mut Env,
    parent_sel: &[String],
    output: &mut Vec<CssRule>,
    extends: &mut Vec<ExtendEntry>,
    resolver: &mut dyn ModuleResolver,
    module_cache: &mut ModuleCache,
) -> Result<(), SassError> {
    let span = tracing::trace_span!("eval_stmt", stage = "eval", node = ?stmt.node_name());
    let _enter = span.enter();

    match stmt {
        Stmt::StyleRule { selector, body } => {
            let span = tracing::debug_span!("eval_style_rule", stage = "eval", module = "style", selector = %selector);
            let _enter = span.enter();

            let eval_sel = eval_interpolation_in_str(selector, env, parent_sel, resolver)?;
            let resolved = resolve_selector(&eval_sel, parent_sel);
            let mut new_parent = parent_sel.to_vec();
            new_parent.push(resolved.clone());
            let nested = eval_stmts(body, env, &new_parent, extends, resolver, module_cache)?;
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

            let val = expr::eval_expr(value, env, parent_sel, resolver)?;
            let css_val = value_to_css(&val);
            let eval_prop = eval_interpolation_in_str(property, env, parent_sel, resolver)?;
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

            let val = expr::eval_expr(value, env, parent_sel, resolver)?;
            env.set_var(name.clone(), val, *global, *default);
        }
        Stmt::Comment(text) => {
            output.push(CssRule::Comment(text.clone()));
        }
        Stmt::IfStmt { branches, else_body } => {
            atrule::eval_if(branches, else_body, env, parent_sel, output, extends, resolver, module_cache)?;
        }
        Stmt::ForStmt { var, from, to, exclusive, body } => {
            atrule::eval_for(var, from, to, *exclusive, body, env, parent_sel, output, extends, resolver, module_cache)?;
        }
        Stmt::EachStmt { vars, list, body } => {
            atrule::eval_each(vars, list, body, env, parent_sel, output, extends, resolver, module_cache)?;
        }
        Stmt::WhileStmt { cond, body } => {
            atrule::eval_while(cond, body, env, parent_sel, output, extends, resolver, module_cache)?;
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
            atrule::eval_include(name, args, content.as_deref(), env, parent_sel, output, extends, resolver, module_cache)?;
        }
        Stmt::ErrorStmt(expr) => {
            let val = expr::eval_expr(expr, env, parent_sel, resolver)?;
            return Err(SassError::user_error(val.to_string(), crate::error::SourcePos::default()));
        }
        Stmt::WarnStmt(expr) => {
            let val = expr::eval_expr(expr, env, parent_sel, resolver)?;
            tracing::warn!(stage = "eval", module = "warn", msg = %val, "@warn");
        }
        Stmt::DebugStmt(expr) => {
            let val = expr::eval_expr(expr, env, parent_sel, resolver)?;
            tracing::debug!(stage = "eval", module = "debug", msg = %val, "@debug");
        }
        Stmt::MediaRule { query, body } => {
            let span = tracing::debug_span!("eval_media", stage = "eval", module = "media", query = %query);
            let _enter = span.enter();

            let eval_query = eval_interpolation_in_str(query, env, parent_sel, resolver)?;
            let body_rules = eval_stmts(body, env, parent_sel, extends, resolver, module_cache)?;
            output.push(CssRule::AtRule {
                name: "media".to_string(),
                value: eval_query,
                body: body_rules,
            });
        }
        Stmt::AtRootRule(body) => {
            let span = tracing::debug_span!("eval_at_root", stage = "eval", module = "at-root");
            let _enter = span.enter();

            let rules = eval_stmts(body, env, &[], extends, resolver, module_cache)?;
            output.extend(rules);
        }
        Stmt::SupportsRule { condition, body } => {
            let span = tracing::debug_span!("eval_supports", stage = "eval", module = "supports");
            let _enter = span.enter();

            let eval_cond = eval_interpolation_in_str(condition, env, parent_sel, resolver)?;
            let body_rules = eval_stmts(body, env, parent_sel, extends, resolver, module_cache)?;
            output.push(CssRule::AtRule {
                name: "supports".to_string(),
                value: eval_cond,
                body: body_rules,
            });
        }
        Stmt::CssAtRule { name, value, body } => {
            let eval_val = eval_interpolation_in_str(value, env, parent_sel, resolver)?;
            let body_rules = if let Some(b) = body {
                eval_stmts(b, env, parent_sel, extends, resolver, module_cache)?
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

            if let Some(content_stmts) = env.content.take() {
                let cloned = content_stmts.clone();
                let rules = eval_stmts(&cloned, env, parent_sel, extends, resolver, module_cache)?;
                output.extend(rules);
                env.content = Some(content_stmts);
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
            eval_use_rule(url, namespace.as_deref(), config, env, output, resolver, module_cache)?;
        }
        Stmt::ForwardRule { url, show, hide } => {
            eval_forward_rule(url, show.as_deref(), hide.as_deref(), env, resolver, module_cache)?;
        }
        Stmt::ImportRule(url) => {
            atrule::eval_import_rule(url, env, parent_sel, output, extends, resolver, module_cache)?
        }
    }
    Ok(())
}

/// Evaluate `@use` rule — loads a module and registers it in the environment.
/// For built-in modules (`sass:math`, `sass:string`, etc.), registers the
/// module's functions and variables under a namespace.
/// For external files, resolves via the `ModuleResolver` trait.
///
/// Uses `ModuleCache` to ensure each module is evaluated at most once.
/// CSS output from a module is emitted only on the first `@use`; subsequent
/// `@use` of the same file reuse the cached public members without re-emitting
/// CSS.
fn eval_use_rule(
    url: &str,
    namespace: Option<&str>,
    config: &[(String, Expr)],
    env: &mut Env,
    output: &mut Vec<CssRule>,
    resolver: &mut dyn ModuleResolver,
    module_cache: &mut ModuleCache,
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

    let span = tracing::debug_span!("eval_use", stage = "eval", module = "use", url = %url);
    let _enter = span.enter();

    // Resolve relative to base_dir on the filesystem
    let base_dir = match env.get_base_dir() {
        Some(d) => d.clone(),
        None => {
            tracing::debug!(stage = "eval", module = "use", url = %url, "no base_dir, skipping");
            return Ok(());
        }
    };

    // Use the resolver to load the module
    let module = match resolver.resolve(url, &base_dir) {
        Ok(m) => m,
        Err(e) => {
            tracing::debug!(stage = "eval", module = "use", url = %url, error = %e, "module not found, skipping");
            return Ok(());
        }
    };

    if module.is_css {
        // CSS file — emit raw content
        if let Some(content) = module.raw_content {
            output.push(CssRule::Raw(content));
        }
        tracing::debug!(stage = "eval", module = "use", url = %url, "CSS module loaded as raw");
        return Ok(());
    }

    // --- SCSS module with ModuleCache support ---
    let module_path = module.source_path.clone();

    // Check if already cached
    if module_cache.contains(&module_path) {
        module_cache.log_hit(&module_path);

        // `with` on an already-loaded module is an error
        if !config.is_empty() {
            return Err(SassError::eval(
                format!(
                    "This module was already loaded, so it can't be configured using `with`.\n{}",
                    module_path.display()
                ),
                crate::error::SourcePos::default(),
            ));
        }

        let cached = module_cache.get(&module_path).unwrap();
        // Register namespace if specified (using cached members)
        if let Some(ns) = namespace {
            env.set_module(ns.to_string(), cached.to_module_env());
            tracing::debug!(stage = "eval", module = "use", url = %url, ns = %ns, "module registered from cache");
        }
        // CSS is NOT re-emitted on cache hit
        return Ok(());
    }

    // Not cached — evaluate the module now
    let sub_dir = module.source_path.parent().map(|p| p.to_path_buf());
    let prev_dir = env.base_dir.take();
    if let Some(d) = sub_dir {
        env.base_dir = Some(d);
    }

    let sub_rules = if !config.is_empty() {
        // @use with config — evaluate in a child scope with config injected
        let span = tracing::info_span!(
            "use_config_inject",
            stage = "eval",
            config_count = config.len(),
            module = %url,
        );
        let _enter = span.enter();

        let mut config_vars: Vec<(String, crate::value::Value)> = Vec::new();
        for (name, expr) in config {
            let val = expr::eval_expr(expr, env, &[], resolver)?;
            config_vars.push((name.clone(), val));
        }

        // Create child environment and inject config variables
        let parent = std::mem::replace(env, Env::new_global());
        let mut module_env = Env::new_child(parent);
        for (name, val) in config_vars {
            module_env.set_var(name, val, false, false);
        }

        let sub_rules = eval_stmts(&module.ast, &mut module_env, &[], &mut Vec::new(), resolver, module_cache)?;
        *env = *module_env.parent.take().unwrap();
        sub_rules
    } else {
        // No config — evaluate directly
        eval_stmts(&module.ast, env, &[], &mut Vec::new(), resolver, module_cache)?
    };

    env.base_dir = prev_dir;

    // Collect public members for caching and namespace registration
    let mut eval_mod = EvaluatedModule {
        variables: HashMap::new(),
        functions: HashMap::new(),
        mixins: HashMap::new(),
        css_output: sub_rules.clone(),
        configured: !config.is_empty(),
    };

    for (name, val) in env.export_vars() {
        if !name.starts_with('-') {
            eval_mod.variables.insert(name, val);
        }
    }
    for (name, func) in env.export_functions() {
        if !name.starts_with('-') {
            eval_mod.functions.insert(name, func);
        }
    }
    for (name, mixin) in env.export_mixins() {
        if !name.starts_with('-') {
            eval_mod.mixins.insert(name, mixin);
        }
    }

    // Emit CSS output (only on first load)
    output.extend(sub_rules);

    // Register namespace if specified
    if let Some(ns) = namespace {
        env.set_module(ns.to_string(), eval_mod.to_module_env());
        tracing::debug!(stage = "eval", module = "use", url = %url, ns = %ns, "module registered with namespace");
    }

    // Store in cache
    module_cache.insert(module_path, eval_mod);

    tracing::debug!(stage = "eval", module = "use", url = %url, "SCSS module loaded and cached");
    Ok(())
}

/// Evaluate `@forward "url" [show/hide ...]` — forwards a module's public
/// members into the current module's public interface.
///
/// `@forward` does NOT produce CSS output (unlike `@use`). It only re-exports
/// members so that downstream `@use` of the forwarding module can access them.
fn eval_forward_rule(
    url: &str,
    show: Option<&[String]>,
    hide: Option<&[String]>,
    env: &mut Env,
    resolver: &mut dyn ModuleResolver,
    module_cache: &mut ModuleCache,
) -> Result<(), SassError> {
    let span = tracing::debug_span!(
        "eval_forward",
        stage = "eval",
        module = "forward",
        url = %url,
        show_count = show.map(|s| s.len()).unwrap_or(0),
        hide_count = hide.map(|h| h.len()).unwrap_or(0),
    );
    let _enter = span.enter();

    // Built-in modules can't be forwarded
    if url.starts_with("sass:") {
        tracing::warn!(stage = "eval", module = "forward", url = %url, "cannot forward built-in module");
        return Ok(());
    }

    // Resolve relative to base_dir
    let base_dir = match env.get_base_dir() {
        Some(d) => d.clone(),
        None => {
            tracing::debug!(stage = "eval", module = "forward", url = %url, "no base_dir, skipping");
            return Ok(());
        }
    };

    let module = match resolver.resolve(url, &base_dir) {
        Ok(m) => m,
        Err(e) => {
            tracing::debug!(stage = "eval", module = "forward", url = %url, error = %e, "module not found, skipping");
            return Ok(());
        }
    };

    if module.is_css {
        tracing::debug!(stage = "eval", module = "forward", url = %url, "CSS module — nothing to forward");
        return Ok(());
    }

    let module_path = module.source_path.clone();

    // Check cache first
    if module_cache.contains(&module_path) {
        module_cache.log_hit(&module_path);
        let cached = module_cache.get(&module_path).unwrap();
        forward_members(env, cached, show, hide);
        tracing::debug!(stage = "eval", module = "forward", url = %url, "forwarded from cache");
        return Ok(());
    }

    // Not cached — evaluate the module
    let sub_dir = module.source_path.parent().map(|p| p.to_path_buf());
    let prev_dir = env.base_dir.take();
    if let Some(d) = sub_dir {
        env.base_dir = Some(d);
    }

    let sub_rules = eval_stmts(&module.ast, env, &[], &mut Vec::new(), resolver, module_cache)?;

    env.base_dir = prev_dir;

    // Collect public members
    let mut eval_mod = EvaluatedModule {
        variables: HashMap::new(),
        functions: HashMap::new(),
        mixins: HashMap::new(),
        css_output: sub_rules, // stored but NOT emitted for @forward
        configured: false,
    };

    for (name, val) in env.export_vars() {
        if !name.starts_with('-') {
            eval_mod.variables.insert(name, val);
        }
    }
    for (name, func) in env.export_functions() {
        if !name.starts_with('-') {
            eval_mod.functions.insert(name, func);
        }
    }
    for (name, mixin) in env.export_mixins() {
        if !name.starts_with('-') {
            eval_mod.mixins.insert(name, mixin);
        }
    }

    // Forward members into current scope (with show/hide filtering)
    forward_members(env, &eval_mod, show, hide);

    // Store in cache
    module_cache.insert(module_path, eval_mod);

    tracing::debug!(stage = "eval", module = "forward", url = %url, "module forwarded and cached");
    Ok(())
}

/// Forward public members from an `EvaluatedModule` into the current environment,
/// applying `show`/`hide` filters.
fn forward_members(
    env: &mut Env,
    module: &EvaluatedModule,
    show: Option<&[String]>,
    hide: Option<&[String]>,
) {
    let is_included = |name: &str| -> bool {
        if let Some(hide_list) = hide {
            if hide_list.iter().any(|h| h == name) {
                return false;
            }
        }
        if let Some(show_list) = show {
            if !show_list.iter().any(|s| s == name) {
                return false;
            }
        }
        true
    };

    for (name, val) in &module.variables {
        if is_included(name) {
            env.variables.insert(name.clone(), val.clone());
        }
    }
    for (name, func) in &module.functions {
        if is_included(name) {
            env.functions.insert(name.clone(), func.clone());
        }
    }
    for (name, mixin) in &module.mixins {
        if is_included(name) {
            env.mixins.insert(name.clone(), mixin.clone());
        }
    }
}
