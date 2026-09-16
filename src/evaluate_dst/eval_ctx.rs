//! Eval-stage context helpers — @import resolution, mixin expansion,
//! @if/@for flow control, and post-process CSS-string → CssNode parsing.

use std::path::{Path, PathBuf};

use crate::ast::{CssNode, Node};
use crate::error::CompileError;
use crate::shared::context::CompilerContext;

// (evaluate_node is re-imported from the parent mod.rs — it's the central dispatch)
use super::evaluate_node;

// ═══════════════════════════════════════════════════════════════════════════════
// Project custom builtins (Element Plus specific)
// ═══════════════════════════════════════════════════════════════════════════════

pub(crate) fn builtin_get_css_var(args: &[String], ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return String::new();
    }
    let mut result = String::new();
    for arg in args {
        let name = arg.trim().trim_start_matches('$');
        let val = ctx.global_variables.borrow();
        let resolved = val.get(name).cloned().unwrap_or_default();
        if !result.is_empty() {
            result.push_str(", ");
        }
        result.push_str(&format!("--{resolved}"));
    }
    format!("var({result})")
}

pub(crate) fn builtin_get_css_var_name(args: &[String], ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return String::new();
    }
    let name = super::substitute_vars(ctx, &args[0]);
    format!("--{name}")
}

// ═══════════════════════════════════════════════════════════════════════════════
// Import / use resolution
// ═══════════════════════════════════════════════════════════════════════════════

/// 从被导入模块的源码中抽取顶层 MixinDef / Variable,注册到父 ctx
pub(super) fn register_imported_definitions(ctx: &CompilerContext, source: &str) {
    let nodes = crate::parse_dst::parse_source(source);
    for node in &nodes {
        match node {
            Node::MixinDef { name, params, body } => {
                tracing::debug!(%name, params = ?params, "registering imported mixin");
                ctx.register_mixin(
                    name.clone(),
                    crate::shared::context::MixinDef {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
            }
            Node::Variable { name, value } => {
                tracing::debug!(%name, %value, "registering imported variable");
                ctx.global_variables
                    .borrow_mut()
                    .insert(name.clone(), value.clone());
            }
            _ => {}
        }
    }
}

pub(super) fn evaluate_import(ctx: &CompilerContext, path: &str) -> Result<Vec<CssNode>, CompileError> {
    let _span = tracing::info_span!("evaluate.import", path).entered();

    if path.starts_with("sass:") {
        tracing::debug!(path, "skipping sass built-in module import");
        return Ok(vec![]);
    }

    let base_dir = ctx
        .path_stack
        .last()
        .cloned()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let file_path = resolve_scss_path(&base_dir, path);

    if let Some(cached) = ctx.try_get_module(&file_path) {
        tracing::debug!(?file_path, "module cache hit");
        return nodes_to_css_nodes(&cached.nodes);
    }

    let content = std::fs::read_to_string(&file_path).map_err(|e| {
        tracing::warn!(error = %e, ?file_path, "import file read failure");
        CompileError::ModuleLoadFailure {
            path: path.to_string(),
            reason: e.to_string(),
        }
    })?;

    register_imported_definitions(ctx, &content);

    let nested_base = file_path.parent().unwrap_or_else(|| Path::new("."));
    let css = crate::compile_at(&content, nested_base).map_err(|e| {
        CompileError::ModuleLoadFailure {
            path: path.to_string(),
            reason: e.to_string(),
        }
    })?;

    parse_compiled_css(&css).map_err(|e| CompileError::ModuleLoadFailure {
        path: path.to_string(),
        reason: e.to_string(),
    })
}

fn resolve_scss_path(base: &PathBuf, raw: &str) -> PathBuf {
    let raw_path = Path::new(raw);
    let file_name = raw_path.file_name().and_then(|n| n.to_str()).unwrap_or(raw);
    let dir_part = raw_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.as_os_str());
    let build = |name: &str| -> PathBuf {
        match dir_part {
            Some(d) => base.join(d).join(name),
            None => base.join(name),
        }
    };
    let candidates = [
        base.join(raw),
        build(&format!("{file_name}.scss")),
        build(&format!("{file_name}.sass")),
        build(&format!("_{file_name}.scss")),
        build(&format!("_{file_name}.sass")),
        match dir_part {
            Some(d) => base.join(d).join(file_name).join("index.scss"),
            None => base.join(file_name).join("index.scss"),
        },
        match dir_part {
            Some(d) => base.join(d).join(file_name).join("_index.scss"),
            None => base.join(file_name).join("_index.scss"),
        },
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    candidates[0].clone()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Mixin call evaluation
// ═══════════════════════════════════════════════════════════════════════════════

pub(super) fn evaluate_mixin_call(
    ctx: &CompilerContext,
    name: &str,
    args: &[String],
) -> Result<Vec<CssNode>, CompileError> {
    let _span = tracing::info_span!("evaluate.mixin_call", name).entered();

    let def = ctx.get_mixin(name).ok_or_else(|| {
        tracing::warn!(%name, "mixin not found in registry");
        CompileError::UndefinedVariable {
            name: format!("@{name} (mixin)"),
        }
    })?;

    tracing::info!(
        params = ?def.params,
        args = ?args,
        body_len = def.body.len(),
        "mixin found, expanding"
    );

    let mut local_vars = Vec::new();
    for (i, param) in def.params.iter().enumerate() {
        if let Some(arg) = args.get(i) {
            let key = param.trim_start_matches('$').to_string();
            local_vars.push((key, arg.clone()));
        } else if let Some((name, default)) = param.split_once(':') {
            let key = name.trim().trim_start_matches('$').to_string();
            local_vars.push((key, default.trim().to_string()));
        }
    }

    let mut out = Vec::new();
    for node in &def.body {
        let expanded = evaluate_node_with_locals(ctx, node, &local_vars)?;
        out.extend(expanded);
    }
    Ok(out)
}

/// Evaluate a Node under local parameter bindings — implements mixin parameter passing
pub(super) fn evaluate_node_with_locals(
    ctx: &CompilerContext,
    node: &Node,
    locals: &[(String, String)],
) -> Result<Vec<CssNode>, CompileError> {
    let saved: Vec<(String, Option<String>)> = locals
        .iter()
        .map(|(name, _)| {
            let prev = ctx.global_variables.borrow().get(name).cloned();
            (name.clone(), prev)
        })
        .collect();

    {
        let mut vars = ctx.global_variables.borrow_mut();
        for (name, value) in locals {
            vars.insert(name.clone(), value.clone());
        }
    }

    let result = evaluate_node(ctx, node);

    {
        let mut vars = ctx.global_variables.borrow_mut();
        for (name, prev) in &saved {
            match prev {
                Some(v) => vars.insert(name.clone(), v.clone()),
                None => vars.remove(name),
            };
        }
    }

    result
}

// ═══════════════════════════════════════════════════════════════════════════════
// Flow-control evaluation
// ═══════════════════════════════════════════════════════════════════════════════

pub(super) fn evaluate_if(
    ctx: &CompilerContext,
    condition: &str,
    then_branch: &[Node],
    else_branch: &Option<Vec<Node>>,
) -> Result<Vec<CssNode>, CompileError> {
    let _span = tracing::info_span!("evaluate.if", condition).entered();
    let cond = condition.trim().to_lowercase();
    let cond_is_falsey = cond == "false" || cond == "null" || cond.is_empty();

    if !cond_is_falsey {
        let mut out = Vec::new();
        for node in then_branch {
            out.extend(evaluate_node(ctx, node)?);
        }
        Ok(out)
    } else if let Some(branch) = else_branch {
        let mut out = Vec::new();
        for node in branch {
            out.extend(evaluate_node(ctx, node)?);
        }
        Ok(out)
    } else {
        Ok(vec![])
    }
}

pub(super) fn evaluate_for(
    ctx: &CompilerContext,
    var: &str,
    from: &str,
    to: &str,
    body: &[Node],
) -> Result<Vec<CssNode>, CompileError> {
    let from_n: i64 = from.parse().map_err(|_| CompileError::InvalidInput {
        message: format!("@for from={from} is not an integer"),
    })?;
    let to_n: i64 = to.parse().map_err(|_| CompileError::InvalidInput {
        message: format!("@for to={to} is not an integer"),
    })?;

    if from_n > to_n {
        return Ok(vec![]);
    }

    let mut out = Vec::new();
    for i in from_n..=to_n {
        ctx.global_variables
            .borrow_mut()
            .insert(var.to_string(), i.to_string());
        for node in body {
            out.extend(evaluate_node(ctx, node)?);
        }
    }
    Ok(out)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Post-process helpers — convert raw CssNode to final form
// ═══════════════════════════════════════════════════════════════════════════════

pub(super) fn nodes_to_css_nodes(nodes: &[Node]) -> Result<Vec<CssNode>, CompileError> {
    let mut out = Vec::new();
    for n in nodes {
        let ctx = CompilerContext::new();
        out.extend(evaluate_node(&ctx, n)?);
    }
    Ok(out)
}

/// Lightweight CSS-string → CssNode parser for imported-module output.
pub(super) fn parse_compiled_css(css: &str) -> Result<Vec<CssNode>, CompileError> {
    let mut nodes = Vec::new();
    for line in css.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("/* COMPILE ERROR:") {
            return Err(CompileError::InvalidInput {
                message: "imported module emitted compile error".into(),
            });
        }
        if let Some(idx) = trimmed.find('{') {
            let sel = trimmed[..idx].trim();
            let inner = trimmed[idx + 1..].trim_end_matches('}').trim();
            let decls = parse_decl_inner(inner);
            nodes.push(CssNode::Rule {
                selector: sel.to_string(),
                body: decls,
            });
        } else if let Some(colon_idx) = trimmed.find(':') {
            let prop = trimmed[..colon_idx].trim().to_string();
            let value = trimmed[colon_idx + 1..]
                .trim()
                .trim_end_matches(';')
                .to_string();
            if !prop.is_empty() {
                nodes.push(CssNode::Declaration { prop, value });
            }
        }
    }
    Ok(nodes)
}

fn parse_decl_inner(inner: &str) -> Vec<CssNode> {
    inner
        .lines()
        .filter_map(|l| {
            let t = l.trim();
            if t.is_empty() {
                return None;
            }
            t.find(':').map(|c| {
                let prop = t[..c].trim().to_string();
                let value = t[c + 1..].trim().trim_end_matches(';').to_string();
                CssNode::Declaration { prop, value }
            })
        })
        .filter(|n| match n {
            CssNode::Declaration { prop, .. } => !prop.is_empty(),
            _ => true,
        })
        .collect()
}
