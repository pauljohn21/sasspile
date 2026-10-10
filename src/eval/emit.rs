use rxrust::prelude::*;
use rxrust::observer::Emitter;
use std::sync::Arc;
use crate::bus::{CompilerBus, FnDef, MixinDef};
use crate::runtime::EvalContext;
use crate::types::*;
use super::EvalEvent;
use super::expr::{combine_selectors, resolve_property, resolve_selector, truthy, value_to_number, value_to_string};
use super::eval_expr;
use super::prefixer;

/// @while 循环最大迭代次数 — 安全守卫防止无限循环
const MAX_WHILE_ITERATIONS: i64 = 10_000;

// ── RxRust 响应式核心：Shared::create 统一源 ────────────────────────────────
//
// 架构:
//   Shared::create(move |sub: &mut dyn Emitter| {
//       emit_ast_node(&node, parent_sel, &ctx, &bus, sub);
//       sub.complete();
//       ()  // 返回空 Subscription
//   })
//
// 关键:
//   - 一个 Shared::create 创建自定义源，完全消除 box_it
//   - emit_ast_node 递归遍历 AST，直接通过 sub.next() 发射事件
//   - Emitter trait 没有 is_closed()——下游 unsubscribe 通过 Subscription teardown 生效
//   - 闭包返回 ()：（空 tuple 已实现 Subscription trait）
//   - 返回具体类型 Shared<Create<...>>，仅在 eval/mod.rs 最终边界统一 box_it

/// AST 节点 → Observable<EvalEvent>
///
/// 使用 Shared::create 创建自定义源，递归函数 emit_ast_node 发射事件。
/// 在函数返回边界调用一次 box_it()，擦除为 SharedBoxedObservable。
/// 这是 box_it 的正确使用位置——flat_map 闭包返回统一类型。
pub(crate) fn emit_events(
    node: AstNode,
    parent_sel: Option<String>,
    ctx: Arc<EvalContext>,
    bus: CompilerBus,
) -> SharedBoxedObservable<'static, EvalEvent, CompileError> {
    Shared::create(move |sub: &mut dyn Emitter<EvalEvent, CompileError>| {
        emit_ast_node(&node, parent_sel, &ctx, &bus, sub);
        sub.complete();
        ()
    })
    .box_it()
}

/// 递归发射 AST 节点事件 —— 直接控制 sub.next()
fn emit_ast_node(
    node: &AstNode,
    parent_sel: Option<String>,
    ctx: &Arc<EvalContext>,
    bus: &CompilerBus,
    sub: &mut dyn Emitter<EvalEvent, CompileError>,
) {

    match node {
        // ── Rule: 嵌套规则分区后递归 ────────────────────────────────────────
        AstNode::Rule { selector, inner } => {
            let resolved = resolve_selector(selector, ctx);
            let combined = match &parent_sel {
                Some(p) => combine_selectors(p, &resolved),
                None => resolved.replace('&', ""),
            };
            tracing::trace!(raw = %selector, combined = %combined, scope_id = ctx.scope_id(), "Rule expanded");

            sub.next(EvalEvent::EnterRule(combined.clone()));

            let (nested_rules, others): (Vec<&AstNode>, Vec<&AstNode>) = inner.iter()
                .partition(|n| matches!(n, AstNode::Rule { .. }));

            let child_ctx = Arc::new(ctx.child_scope(1));
            let child_bus = bus.clone();

            for o in others {
                emit_ast_node(o, Some(combined.clone()), &child_ctx, &child_bus, sub);
            }

            for nr in nested_rules {
                emit_ast_node(nr, Some(combined.clone()), &child_ctx, &child_bus, sub);
            }

            sub.next(EvalEvent::LeaveRule);
        }

        // ── Media: 查询解析后递归 ────────────────────────────────────────────
        AstNode::Media { query, inner } => {
            let resolved_query = resolve_query(query, ctx);
            sub.next(EvalEvent::EnterMedia(resolved_query));

            let child_ctx = Arc::new(ctx.child_scope(2));
            let child_bus = bus.clone();

            for n in inner {
                emit_ast_node(n, parent_sel.clone(), &child_ctx, &child_bus, sub);
            }

            sub.next(EvalEvent::LeaveMedia);
        }

        // ── Supports: 查询解析后递归 ─────────────────────────────────────────
        AstNode::Supports { query, inner } => {
            let resolved_query = resolve_query(query, ctx);
            sub.next(EvalEvent::EnterSupports(resolved_query));

            let child_ctx = Arc::new(ctx.child_scope(3));
            let child_bus = bus.clone();

            for n in inner {
                emit_ast_node(n, parent_sel.clone(), &child_ctx, &child_bus, sub);
            }

            sub.next(EvalEvent::LeaveSupports);
        }

        // ── ContentBlock: 展开 @content 节点 ─────────────────────────────────
        AstNode::ContentBlock(inner_nodes) => {
            let scope = ctx.content_scope.clone().unwrap_or_else(|| ctx.clone());
            for n in inner_nodes {
                emit_ast_node(n, parent_sel.clone(), &scope, bus, sub);
            }
        }

        // ── Import: 展开导入节点 ─────────────────────────────────────────────
        AstNode::Import(inner_nodes) => {
            for n in inner_nodes {
                emit_ast_node(n, parent_sel.clone(), ctx, bus, sub);
            }
        }

        // ── MixinCall: 查找并展开 mixin body ─────────────────────────────────
        AstNode::MixinCall { name, args, content } => {
            let _span = tracing::debug_span!("mixin_call", mixin_name = %name, parent_sel = ?parent_sel).entered();

            match bus.lookup_mixin(name) {
                Some(mixin) => {
                    let child_ctx_raw = ctx.child_scope(8);
                    bind_mixin_args(&child_ctx_raw, &mixin.params, args, ctx, bus);
                    let expanded = expand_body_with_content(&mixin.body, content);
                    let mut mixin_ctx_inner = child_ctx_raw.clone();
                    mixin_ctx_inner.content_scope = Some(ctx.clone());
                    let mixin_ctx = Arc::new(mixin_ctx_inner);

                    for n in &expanded {
                        emit_ast_node(n, parent_sel.clone(), &mixin_ctx, bus, sub);
                    }
                }
                None => {
                    tracing::debug!("mixin not found: {}", name);
                }
            }
        }

        // ── @if: 条件分支选择 ────────────────────────────────────────────────
        AstNode::If { cond, then_branch, else_branch } => {
            let cond_val = eval_expr(cond, ctx, bus);
            let branch = if truthy(&cond_val) { then_branch } else { else_branch.as_ref().map(|b| b.as_slice()).unwrap_or(&[]) };

            for n in branch {
                emit_ast_node(n, parent_sel.clone(), ctx, bus, sub);
            }
        }

        // ── @for: 数值范围迭代 ───────────────────────────────────────────────
        AstNode::For { var, from, to, inclusive, body } => {
            let from_n = value_to_number(&eval_expr(from, ctx, bus)) as i64;
            let to_n = value_to_number(&eval_expr(to, ctx, bus)) as i64;
            let effective_to = if *inclusive { to_n } else { to_n - 1 };

            for i in from_n..=effective_to {
                let child_ctx = Arc::new(ctx.child_scope(5));
                child_ctx.bind_var(var, Value::Number(i as f64, None));

                for n in body {
                    emit_ast_node(n, parent_sel.clone(), &child_ctx, bus, sub);
                }
            }
        }

        // ── @each: 列表迭代 ──────────────────────────────────────────────────
        AstNode::Each { vars, list, body } => {
            let list_val = eval_expr(list, ctx, bus);
            let items = match list_val {
                Value::List(items, _) => items,
                Value::Map(entries) => entries.into_iter()
                    .map(|(k, v)| Value::List(vec![Value::String(k), v], ListSeparator::Comma))
                    .collect(),
                other => vec![other],
            };

            for item_val in &items {
                let child_ctx = ctx.clone();
                bind_each_vars(&child_ctx, vars, item_val);

                for n in body {
                    emit_ast_node(n, parent_sel.clone(), &child_ctx, bus, sub);
                }
            }
        }

        // ── @while: 运行时交替求值条件 + 执行 body ─────────────────────────────
        AstNode::While { cond, body } => {
            // 与 @for/@each 不同，@while 的变量修改必须在同一 scope 可见
            // 不能 child_scope——否则条件判断看不到 body 里的 $i: $i + 1
            let mut cnt = 0i64;
            loop {
                let cond_val = eval_expr(cond, ctx, bus);
                if !truthy(&cond_val) || cnt >= MAX_WHILE_ITERATIONS { break; }

                for n in body {
                    emit_ast_node(n, parent_sel.clone(), ctx, bus, sub);
                }
                cnt += 1;
            }
        }

        // ── 叶节点：直接求值发射 ─────────────────────────────────────────────
        leaf => {
            emit_leaf_node(leaf, parent_sel, ctx, bus, sub);
        }
    }
}

/// 发射叶节点事件
fn emit_leaf_node(
    node: &AstNode,
    parent_sel: Option<String>,
    ctx: &Arc<EvalContext>,
    bus: &CompilerBus,
    sub: &mut dyn Emitter<EvalEvent, CompileError>,
) {
    match node {
        AstNode::StyleDecl { property, value, important } => {
            let val = eval_expr(value, ctx, bus);
            let prop_name = resolve_property(property, ctx);
            let val_str = value_to_string(&val);

            let final_val = if val_str == "null" || prop_name.contains("null") {
                if prop_name.starts_with("--") {
                    tracing::debug!(property = %prop_name, "null fallback to unset for CSS custom property");
                    "unset".to_string()
                } else {
                    tracing::debug!(property = %prop_name, "null value skipped for non-custom property");
                    return;
                }
            } else {
                val_str
            };

            let final_val = if *important {
                format!("{} !important", final_val)
            } else {
                final_val
            };

            let mut prefixes: Vec<String> = Vec::new();
            if !prefixer::is_prefixed(&prop_name)
                && let Some(vp) = prefixer::get_vendor_prefixes(&prop_name)
            {
                prefixes = vp;
            }

            for pref in &prefixes {
                sub.next(EvalEvent::Terminal(CssStmt::Decl {
                    property: pref.clone(),
                    value: final_val.clone(),
                }));
            }

            sub.next(EvalEvent::Terminal(CssStmt::Decl {
                property: prop_name,
                value: final_val,
            }));
        }
        AstNode::Css(stmt) => {
            sub.next(EvalEvent::Terminal(stmt.clone()));
        }
        AstNode::Extend(target) => {
            match &parent_sel {
                Some(current) => {
                    let resolved_target = resolve_selector(target, ctx);
                    sub.next(EvalEvent::AddSelector {
                        target: resolved_target,
                        source: current.clone(),
                    });
                }
                None => {
                    tracing::debug!(extend_target = %target, "extend without parent_sel — skipped");
                }
            }
        }
        AstNode::VariableDecl { name, value, .. } => {
            let val = eval_expr(value, ctx, bus);
            ctx.bind_var(name, val);
        }
        AstNode::MixinDecl { name, params, body } => {
            bus.register_mixin(MixinDef {
                name: name.clone(),
                params: params.clone(),
                body: body.clone(),
            });
        }
        AstNode::FunctionDecl { name, params, body } => {
            bus.register_fn(FnDef {
                name: name.clone(),
                params: params.clone(),
                body: body.clone(),
            });
        }
        AstNode::Content | AstNode::Return(_) => {
            // 无事件发射
        }
        AstNode::Warn(val) => {
            let _ = eval_expr(val, ctx, bus);
        }
        AstNode::Debug(val) => {
            let _ = eval_expr(val, ctx, bus);
        }
        _ => {
            // 未知节点类型，忽略或记录
        }
    }
}

// ── 辅助函数 ─────────────────────────────────────────────────────────────────

fn bind_each_vars(ctx: &EvalContext, vars: &[String], item_val: &Value) {
    match vars.len() {
        1 => ctx.bind_var(&vars[0], item_val.clone()),
        2 => {
            if let Value::List(pair, _) = item_val {
                if let Some(key) = pair.first() {
                    tracing::trace!(var = %vars[0], val = %key, scope = ctx.scope_id(), "bind_each_vars");
                    ctx.bind_var(&vars[0], key.clone());
                }
                if let Some(val) = pair.get(1) {
                    tracing::trace!(var = %vars[1], val = %val, scope = ctx.scope_id(), "bind_each_vars");
                    ctx.bind_var(&vars[1], val.clone());
                }
            } else {
                ctx.bind_var(&vars[0], item_val.clone());
            }
        }
        _ => ctx.bind_var(&vars[0], item_val.clone()),
    }
}

fn bind_mixin_args(
    child_ctx: &EvalContext,
    params: &[Param],
    args: &[AstNode],
    ctx: &EvalContext,
    bus: &CompilerBus,
) {
    for (i, p) in params.iter().enumerate() {
        if p.is_rest {
            let rest_vals: Vec<Value> = args[i..].iter()
                .map(|a| eval_expr(a, ctx, bus))
                .collect();
            child_ctx.bind_var(&p.name, Value::List(rest_vals, ListSeparator::Comma));
            break;
        }
        let val = if let Some(a) = args.get(i) {
            eval_expr(a, ctx, bus)
        } else if let Some(default) = &p.default_value {
            eval_expr(default, ctx, bus)
        } else {
            Value::Null
        };
        child_ctx.bind_var(&p.name, val);
    }
}

fn resolve_query(query: &str, ctx: &EvalContext) -> String {
    if !query.contains('$') {
        return query.to_string();
    }
    let mut result = String::new();
    let mut chars = query.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' {
            let mut var_name = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    var_name.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if !var_name.is_empty() {
                let val = ctx.var(&var_name).unwrap_or(Value::Null);
                result.push_str(&value_to_string(&val));
            } else {
                result.push('$');
            }
        } else {
            result.push(ch);
        }
    }
    result
}

fn expand_body_with_content(nodes: &[AstNode], content: &[AstNode]) -> Vec<AstNode> {
    nodes.iter()
        .flat_map(|node| expand_node_with_content(node, content))
        .collect()
}

fn expand_node_with_content(node: &AstNode, content: &[AstNode]) -> Vec<AstNode> {
    if matches!(node, AstNode::Content) {
        return vec![AstNode::ContentBlock(content.to_vec())];
    }
    match node {
        AstNode::Rule { selector, inner } => {
            let expanded = expand_body_with_content(inner, content);
            vec![AstNode::Rule { selector: selector.clone(), inner: expanded }]
        }
        AstNode::Media { query, inner } => {
            let expanded = expand_body_with_content(inner, content);
            vec![AstNode::Media { query: query.clone(), inner: expanded }]
        }
        AstNode::Supports { query, inner } => {
            let expanded = expand_body_with_content(inner, content);
            vec![AstNode::Supports { query: query.clone(), inner: expanded }]
        }
        AstNode::If { cond, then_branch, else_branch } => {
            let then_expanded = expand_body_with_content(then_branch, content);
            let else_expanded = else_branch.as_ref().map(|eb| expand_body_with_content(eb, content));
            vec![AstNode::If { cond: cond.clone(), then_branch: then_expanded, else_branch: else_expanded }]
        }
        _ => vec![node.clone()],
    }
}
