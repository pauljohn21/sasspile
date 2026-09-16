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

use crate::ast::{CssNode, Node};
use crate::error::CompileError;
use crate::shared::context::{CompilerContext, MixinDef};

mod builtins;
mod builtins_math;
mod eval_ctx;

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
pub(crate) fn evaluate_node(ctx: &CompilerContext, node: &Node) -> Result<Vec<CssNode>, CompileError> {
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
        Node::Import { path } => eval_ctx::evaluate_import(ctx, path),
        Node::Use { path } => eval_ctx::evaluate_import(ctx, path),
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
            eval_ctx::evaluate_mixin_call(ctx, name, args)
        }
        Node::Extend { selector } => {
            tracing::debug!(%selector, "extend not yet implemented");
            Ok(vec![])
        }
        Node::If {
            condition,
            then_branch,
            else_branch,
        } => eval_ctx::evaluate_if(ctx, condition, then_branch, else_branch),
        Node::For { var, from, to, body } => eval_ctx::evaluate_for(ctx, var, from, to, body),
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

/// Variable substitution — 扫描 $var 标识符,在 global_variables 中查找替换。
///
/// 处理 `$var-rest` 模式: 从最短到最长尝试分割 `$ident`(以 `-` 为分隔符),
/// 最先在 variables 中找到的即采用。
///
/// 同时处理内置函数调用 `fn-name(args)` — 解析参数并在 builtin 注册表中查找求值,
/// 把整个 `fn-name(args)` 替换为返回值。支持点分模块名 (`map.get`, `list.nth`)。
pub(crate) fn substitute_vars(ctx: &CompilerContext, s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars_vec: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars_vec.len() {
        let ch = chars_vec[i];

        if ch == '$' {
            // 变量替换 — 先在局部作用域链查找, 未命中再查全局
            i += 1;
            let mut ident = String::new();
            while i < chars_vec.len() {
                let c = chars_vec[i];
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    ident.push(c);
                    i += 1;
                } else {
                    break;
                }
            }
            if ident.is_empty() {
                out.push('$');
                continue;
            }
            // 优先: 局部作用域栈 (mixin 参数等)
            if let Some(value) = ctx.lookup_local(&ident) {
                out.push_str(&value);
                continue;
            }
            // 其次: 全局变量 (clone 后立即释放 borrow)
            if let Some(value) = ctx.global_variables.borrow().get(ident.as_str()).cloned() {
                out.push_str(&value);
            } else if ident.contains('-') {
                let mut matched = false;
                for (j, c) in ident.char_indices() {
                    if c == '-' {
                        let (head, tail) = ident.split_at(j);
                        if let Some(value) = ctx.global_variables.borrow().get(head).cloned() {
                            out.push_str(&value);
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
        } else if ch.is_alphabetic() || ch == '_' {
            // 可能是函数调用: ident(args) 或 module.fn(args)
            let mut ident = String::new();
            while i < chars_vec.len() {
                let c = chars_vec[i];
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    ident.push(c);
                    i += 1;
                } else {
                    break;
                }
            }
            // 检查是否是函数调用 (允许点分模块名)
            if i < chars_vec.len() && chars_vec[i] == '(' {
                // 跳过可选的 module. 前缀,提取纯函数名
                let fn_name = ident.rsplitn(2, '.').next().unwrap().to_string();
                // 解析匹配的 (...) 参数
                i += 1; // skip '('
                let args_start = i;
                let mut depth = 1u32;
                while i < chars_vec.len() && depth > 0 {
                    match chars_vec[i] {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    if depth > 0 {
                        i += 1;
                    }
                }
                let args_str = &s[args_start..i];
                // i 现在指向 ')'
                if i < chars_vec.len() {
                    i += 1; // skip ')'
                }
                // 解析逗号分隔参数
                let args = split_args(args_str);
                tracing::debug!(%fn_name, ?args, "builtin function call detected");
                let result = eval_builtin(&fn_name, &args, ctx);
                out.push_str(&result);
            } else {
                // 普通标识符,不替换
                out.push_str(&ident);
            }
        } else {
            out.push(ch);
            i += 1;
        }
    }
    out
}

/// 按顶级逗号分割参数 (忽略嵌套括号内的逗号)
pub(crate) fn split_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut depth = 0u32;
    let mut current = String::new();
    for ch in s.chars() {
        match ch {
            '(' | '[' => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' => {
                if depth > 0 {
                    depth -= 1;
                }
                current.push(ch);
            }
            ',' if depth == 0 => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    args.push(trimmed);
                }
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        args.push(trimmed);
    }
    args
}

/// 仅解析变量 (不触发函数求值),避免 substitute_vars 递归调用自己
pub(crate) fn resolve_vars_only(
    vars: &std::cell::Ref<'_, std::collections::HashMap<String, String>>,
    s: &str,
) -> String {
    let mut out = String::with_capacity(s.len());
    let chars_vec: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars_vec.len() {
        let ch = chars_vec[i];
        if ch == '$' {
            i += 1;
            let mut ident = String::new();
            while i < chars_vec.len() {
                let c = chars_vec[i];
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    ident.push(c);
                    i += 1;
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
                let mut matched = false;
                for (j, c) in ident.char_indices() {
                    if c == '-' {
                        let (head, tail) = ident.split_at(j);
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
            i += 1;
        }
    }
    out
}

/// 内置函数求值
fn eval_builtin(name: &str, args: &[String], ctx: &CompilerContext) -> String {
    let _span = tracing::info_span!("eval.builtin", name, args = ?args).entered();
    match name {
        // --- Map ---
        "map-get" | "get" => builtins::builtin_map_get(args, ctx),
        "map-has-key" | "has-key" => builtins::builtin_map_has_key(args, ctx),
        "map-keys" | "keys" => builtins::builtin_map_keys(args, ctx),
        "map-values" | "values" => builtins::builtin_map_values(args, ctx),
        "map-merge" | "merge" => builtins::builtin_map_merge(args, ctx),
        // --- List ---
        "nth" => builtins::builtin_nth(args, ctx),
        "length" => builtins::builtin_length(args, ctx),
        "append" => builtins::builtin_append(args, ctx),
        "join" => builtins::builtin_join(args, ctx),
        "index" => builtins::builtin_index(args, ctx),
        // --- String ---
        "unquote" => builtins::builtin_unquote(args, ctx),
        "quote" => builtins::builtin_quote(args, ctx),
        "str-length" => builtins::builtin_str_length(args, ctx),
        "str-index" => builtins::builtin_str_index(args, ctx),
        // --- Color ---
        "mix" => builtins::builtin_mix(args, ctx),
        "darken" => builtins::builtin_darken(args, ctx),
        "lighten" => builtins::builtin_lighten(args, ctx),
        "alpha" | "opacity" => builtins::builtin_alpha(args, ctx),
        "red" | "green" | "blue" => builtins::builtin_color_component(name, args, ctx),
        "grayscale" => builtins::builtin_grayscale(args, ctx),
        "invert" => builtins::builtin_invert(args, ctx),
        // --- Math ---
        "percentage" => builtins_math::builtin_percentage(args, ctx),
        "abs" | "ceil" | "floor" | "round" => builtins_math::builtin_math(name, args, ctx),
        "min" | "max" => builtins_math::builtin_min_max(name, args, ctx),
        "unit" => builtins_math::builtin_unit(args, ctx),
        "unitless" => builtins_math::builtin_unitless(args, ctx),
        "comparable" => builtins_math::builtin_comparable(args, ctx),
        // --- Type / Meta ---
        "type-of" => builtins_math::builtin_type_of(args, ctx),
        "variable-exists" => builtins_math::builtin_variable_exists(args, ctx),
        "global-variable-exists" => builtins_math::builtin_global_variable_exists(args, ctx),
        "inspect" => builtins_math::builtin_inspect(args, ctx),
        "not" => builtins_math::builtin_not(args, ctx),
        // --- Module / Selector ---
        "selector-nest" => builtins_math::builtin_selector_nest(args, ctx),
        "selector-append" => builtins_math::builtin_selector_append(args, ctx),
        "call" => builtins_math::builtin_call(args, ctx),
        "get-function" => builtins_math::builtin_get_function(args, ctx),
        "if" => builtins_math::builtin_if_function(args, ctx),
        // --- Custom Project ---
        "getCssVar" => eval_ctx::builtin_get_css_var(args, ctx),
        "getCssVarName" => eval_ctx::builtin_get_css_var_name(args, ctx),
        _ => {
            tracing::warn!(name, "unknown builtin function");
            String::new()
        }
    }
}
