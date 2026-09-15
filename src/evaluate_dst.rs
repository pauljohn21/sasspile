//! Evaluate Stage (Stage 3)
//!
//! `Observable<Node, Infallible>`
//!     → `Observable<Result<Vec<CssNode>, CompileError>, Infallible>`
//!
//! Operator mapping:
//!   - `scan_map(CompilerContext::new(), reducer)` — threads mutable state
//!   - `flat_map(|outcomes| ...)` — 1 Node → 0..N CssNode findings
//!   - `distinct_until_changed()` — suppress duplicate error values
//!   - `tap(|item| ...)` — log errors as values (the only side-effect site)
//!
//! Why `scan_map`?
//!   `CompilerContext` (mutable state) propagates through the stream.
//!
//! Why `flat_map`?
//!   `@for`, `@if else`, `@import` all expand one Node into N CssNodes — the
//!   cardinality change is flat_map's purpose.
//!
//! Error-as-value contract:
//!   `CompileError` is returned inside `Result::Err`, **not** as Observable::Err.
//!   Observable Err type stays `Infallible`.

use rxrust::prelude::*;
use std::convert::Infallible;
use std::path::{Path, PathBuf};

use crate::ast::{CssNode, Node};
use crate::error::CompileError;
use crate::shared::context::{CompilerContext, MixinDef};

impl CompilerContext {
    /// 注册一个 mixin 定义
    pub fn register_mixin(&self, name: String, def: MixinDef) {
        self.mixins.borrow_mut().insert(name, def);
    }

    /// 查找 mixin
    pub fn get_mixin(&self, name: &str) -> Option<MixinDef> {
        self.mixins.borrow().get(name).cloned()
    }
}

/// Result alias — 1 Node maps to 0..N CssNode findings with possible errors.
pub type EvalOutcome = Result<Vec<CssNode>, CompileError>;

/// Build the evaluate stage — attaches operators to the upstream Node stream.
pub fn attach(
    upstream: LocalBoxedObservable<'static, Node, Infallible>,
) -> LocalBoxedObservable<'static, EvalOutcome, Infallible> {
    attach_with_ctx(upstream, CompilerContext::new())
}

/// Attach with a pre-seeded CompilerContext — used to inject the base path for
/// resolving relative @import/@use paths.
pub fn attach_with_ctx(
    upstream: LocalBoxedObservable<'static, Node, Infallible>,
    ctx: CompilerContext,
) -> LocalBoxedObservable<'static, EvalOutcome, Infallible> {
    let _span = tracing::info_span!("evaluate.attach").entered();

    upstream
        .scan_map(ctx, eval_and_accumulate_state)
        .flat_map(|outcomes| Local::from_iter(outcomes))
        .distinct_until_changed()
        .tap(|item| {
            if item.is_err() {
                tracing::warn!(?item, stage = "evaluate", "error as value");
            }
        })
        .box_it()
}

/// scan_map reducer: mutate CompilerContext (state) and emit 0..N EvalOutcome's.
fn eval_and_accumulate_state(ctx: &mut CompilerContext, node: Node) -> Vec<EvalOutcome> {
    let span = tracing::info_span!("evaluate.feed", node = ?node);
    let _enter = span.enter();

    match evaluate_node(ctx, &node) {
        Ok(css_nodes) if css_nodes.is_empty() => vec![],
        Ok(css_nodes) => vec![Ok(css_nodes)],
        Err(e) => {
            tracing::warn!(error = %e, ?node, "evaluation failed");
            vec![Err(e)]
        }
    }
}

/// Evaluate a single Node against the current CompilerContext.
    fn evaluate_node(ctx: &CompilerContext, node: &Node) -> Result<Vec<CssNode>, CompileError> {
        match node {
            Node::Rule { selector, body } => evaluate_rule(ctx, selector, body),
            Node::Declaration { prop, value } => {
                if prop.is_empty() {
                    return Err(CompileError::MissingValue {
                        message: "empty property name".into(),
                    });
                }
                let p = substitute_vars(ctx, prop);
                let v = substitute_vars(ctx, value);
                Ok(vec![CssNode::Declaration { prop: p, value: v }])
            }
        Node::Text(t) => Ok(vec![CssNode::Text(substitute_vars(ctx, t))]),
        Node::Variable { name, value } => {
            tracing::info!(target: "sasspile.variable", %name, %value, "VARIABLE REGISTER");
            ctx.global_variables
                .borrow_mut()
                .insert(name.clone(), value.clone());
            Ok(vec![])
        }
        Node::Import { path } => evaluate_import(ctx, path),
        Node::Use { path } => evaluate_import(ctx, path), // @use 展开同 @import (模块加载语义)
        Node::Forward { path: _ } => {
            tracing::debug!("forward passthrough");
            Ok(vec![])
        }
        Node::MixinDef { name, params, body } => {
            tracing::info!(%name, params = ?params, body_len = body.len(), "mixin registered");
            let def = MixinDef {
                params: params.clone(),
                body: body.clone(),
            };
            ctx.register_mixin(name.clone(), def);
            Ok(vec![])
        }
        Node::MixinCall { name, args } => {
            tracing::info!(%name, args = ?args, "mixin expanding");
            evaluate_mixin_call(ctx, name, args)
        }
        Node::Extend { selector } => {
            tracing::debug!(%selector, "extend not yet implemented");
            Ok(vec![])
        }
        Node::If {
            condition,
            then_branch,
            else_branch,
        } => evaluate_if(ctx, condition, then_branch, else_branch),
        Node::For { var, from, to, body } => evaluate_for(ctx, var, from, to, body),
        Node::Directive { name, args } => {
            tracing::debug!(%name, %args, "directive passthrough");
            Ok(vec![])
        }
    }
}

fn evaluate_rule(
    ctx: &CompilerContext,
    selector: &str,
    body: &[Node],
) -> Result<Vec<CssNode>, CompileError> {
    let _span = tracing::info_span!("evaluate.rule", selector).entered();
    let mut css_body = Vec::new();
    for item in body {
        match evaluate_node(ctx, item) {
            Ok(nodes) => css_body.extend(nodes),
            Err(e) => return Err(e),
        }
    }
    Ok(vec![CssNode::Rule {
        selector: substitute_vars(ctx, selector),
        body: css_body,
    }])
}

/// Variable substitution — 扫描 $var 标识符，在 global_variables 中查找替换。
///
/// 处理 `$var-rest` 模式: 从最短到最长尝试分割 `$ident`(以 `-` 为分隔符),
/// 最先在 variables 中找到的即采用。
fn substitute_vars(ctx: &CompilerContext, s: &str) -> String {
    let vars = ctx.global_variables.borrow();
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' {
            let mut ident = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    ident.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if ident.is_empty() {
                out.push('$');
                continue;
            }
            if let Some(value) = vars.get(&ident) {
                out.push_str(value);
            } else if ident.contains('-') {
                // 找到第一个能匹配的最短前缀
                let mut matched = false;
                for (i, c) in ident.char_indices() {
                    if c == '-' {
                        let (head, tail) = ident.split_at(i);
                        if let Some(value) = vars.get(head) {
                            out.push_str(value);
                            out.push_str(tail);
                            matched = true;
                            break;
                        }
                    }
                }
                if !matched {
                    out.push('$');
                    out.push_str(&ident);
                }
            } else {
                out.push('$');
                out.push_str(&ident);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn evaluate_import(ctx: &CompilerContext, path: &str) -> Result<Vec<CssNode>, CompileError> {
    let _span = tracing::info_span!("evaluate.import", path).entered();

    // 跳过 sass 内置模块 (sass:meta, sass:string, sass:math, sass:color 等)
    // 这些不是真实文件路径,而是内建函数库的命名空间
    if path.starts_with("sass:") {
        tracing::debug!(path, "skipping sass built-in module import");
        return Ok(vec![]);
    }

    // Resolve path relative to ctx.path_stack.last() — already a directory (not a file path).
    // pipeline.rs 推送的 bp = entry.path.parent() = 正确的 base 目录
    let base_dir = ctx
        .path_stack
        .last()
        .cloned()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let file_path = resolve_scss_path(&base_dir, path);

    // Hot-path: cached module → return stored nodes converted to CssNode.
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

    // Recursive compile — 使用 file_path.parent() 作为 base_dir,
    // 确保嵌套 @import/@use 能正确从被导入模块的目录开始解析
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
    // 拆分 raw 的目录和文件名部分,前缀 _ 只加在文件名前
    let raw_path = Path::new(raw);
    let file_name = raw_path.file_name().and_then(|n| n.to_str()).unwrap_or(raw);
    let dir_part = raw_path.parent().filter(|p| !p.as_os_str().is_empty()).map(|p| p.as_os_str());
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

/// 展开 mixin 调用 — 从 ctx.mixins 查找定义，展开 body 并替换参数
fn evaluate_mixin_call(
    ctx: &CompilerContext,
    name: &str,
    args: &[String],
) -> Result<Vec<CssNode>, CompileError> {
    let _span = tracing::info_span!("evaluate.mixin_call", name).entered();

    // 查找 mixin 定义
    let def = ctx.get_mixin(name).ok_or_else(|| {
        tracing::warn!(%name, "mixin not found in registry");
        CompileError::UndefinedVariable {
            name: format!("@{name} (mixin)"),
        }
    })?;

    tracing::info!(params = ?def.params, args = ?args, body_len = def.body.len(), "mixin found, expanding");

    // 建立参数映射: param_name → arg_value
    // substitute_vars 查找时用不带 $ 的 ident (如 color),所以 locals key 也必须不带 $
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

    // 在局部作用域展开 body
    let mut out = Vec::new();
    for node in &def.body {
        let expanded = evaluate_node_with_locals(ctx, node, &local_vars)?;
        out.extend(expanded);
    }
    Ok(out)
}

/// 在局部参数绑定下 evaluate 一组 Node — 实现 mixin 参数传递
fn evaluate_node_with_locals(
    ctx: &CompilerContext,
    node: &Node,
    locals: &[(String, String)],
) -> Result<Vec<CssNode>, CompileError> {
    // 注入局部变量到 context, 然后 evaluate, 让参数优先于全局变量
    // 简化策略: 先临时覆盖 global_variables，再恢复
    let saved: Vec<(String, Option<String>)> = locals
        .iter()
        .map(|(name, _)| {
            let prev = ctx.global_variables.borrow().get(name).cloned();
            (name.clone(), prev)
        })
        .collect();

    // 注入新值
    {
        let mut vars = ctx.global_variables.borrow_mut();
        for (name, value) in locals {
            vars.insert(name.clone(), value.clone());
        }
    }

    let result = evaluate_node(ctx, node);

    // 恢复原始值
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

fn evaluate_if(
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

fn evaluate_for(
    ctx: &CompilerContext,
    var: &str,
    from: &str,
    to: &str,
    body: &[Node],
) -> Result<Vec<CssNode>, CompileError> {
    let from_n: i64 = from
        .parse()
        .map_err(|_| CompileError::InvalidInput {
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

fn nodes_to_css_nodes(nodes: &[Node]) -> Result<Vec<CssNode>, CompileError> {
    let mut out = Vec::new();
    for n in nodes {
        let ctx = CompilerContext::new();
        out.extend(evaluate_node(&ctx, n)?);
    }
    Ok(out)
}

/// Lightweight CSS-string → CssNode parser for imported-module output.
fn parse_compiled_css(css: &str) -> Result<Vec<CssNode>, CompileError> {
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
