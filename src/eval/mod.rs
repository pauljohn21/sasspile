use rxrust::prelude::*;
use std::convert::Infallible;
use std::sync::Arc;
use crate::bus::{CompilerBus, FnDef, MixinDef};
use crate::runtime::EvalContext;
use crate::types::*;

pub fn eval_stream(
    ast_stream: AstStream,
    ctx: Arc<EvalContext>,
) -> CssStream {
    let bus = ctx.bus().clone();
    let mut result_subject: SharedSubject<'static, CssStmt, Infallible> = Shared::subject();

    let nodes_arc = Arc::new(std::sync::Mutex::new(Vec::<AstNode>::new()));
    let nref = nodes_arc.clone();
    ast_stream.subscribe(move |node| {
        nref.lock().unwrap().push(node);
    });

    let nodes = nodes_arc.lock().unwrap().clone();
    let stmts: Vec<CssStmt> = nodes.iter().flat_map(|n| {
        let mut s = Vec::new();
        eval_node_collect(n, &ctx, &bus, &mut s);
        s
    }).collect();

    for stmt in &stmts {
        result_subject.next(stmt.clone());
    }

    result_subject.box_it()
}

pub fn eval_nodes_sync(
    nodes: &[AstNode],
    ctx: &EvalContext,
    bus: &CompilerBus,
) -> Vec<CssStmt> {
    let mut result = Vec::new();
    for node in nodes {
        eval_node_collect(node, ctx, bus, &mut result);
    }
    result
}

pub fn eval_ast_stream_sync(ast_stream: AstStream, ctx: Arc<EvalContext>) -> Vec<CssStmt> {
    let bus = ctx.bus().clone();
    let v_nodes = Arc::new(std::sync::Mutex::new(Vec::<AstNode>::new()));

    let v_ref = v_nodes.clone();
    ast_stream.subscribe(move |node| {
        v_ref.lock().unwrap().push(node);
    });

    let nodes = v_nodes.lock().unwrap().clone();
    eval_nodes_sync(&nodes, &ctx, &bus)
}

fn eval_node(
    node: &AstNode,
    ctx: &EvalContext,
    subj: &mut SharedSubject<'static, CssStmt, Infallible>,
    bus: &CompilerBus,
) {
    match node {
        AstNode::Literal(_) => {}
        AstNode::VariableRef { name, .. } => { let _ = ctx.var(name); }
        AstNode::VariableDecl { name, value, .. } => {
            let val = eval_expr(value, ctx, bus);
            ctx.bind_var(name, val);
        }
        AstNode::StyleDecl { property, value } => {
            let val = eval_expr(value, ctx, bus);
            subj.next(CssStmt::Decl {
                property: property.clone(),
                value: value_to_string(&val),
            });
        }
        AstNode::Rule { selector, inner } => {
            let child_ctx = ctx.child_scope(1);
            let mut inner_stmts = Vec::new();
            for n in inner {
                if let Some(css) = eval_rule_inner(n, &child_ctx, bus) {
                    inner_stmts.push(css);
                }
            }
            subj.next(CssStmt::Rule {
                selector: selector.clone(),
                inner: inner_stmts,
            });
        }
        AstNode::Media { query, inner } => {
            let child_ctx = ctx.child_scope(2);
            let mut inner_stmts = Vec::new();
            for n in inner {
                if let Some(css) = eval_rule_inner(n, &child_ctx, bus) {
                    inner_stmts.push(css);
                }
            }
            subj.next(CssStmt::Media {
                query: query.clone(),
                inner: inner_stmts,
            });
        }
        AstNode::Supports { query, inner } => {
            let child_ctx = ctx.child_scope(3);
            let mut inner_stmts = Vec::new();
            for n in inner {
                if let Some(css) = eval_rule_inner(n, &child_ctx, bus) {
                    inner_stmts.push(css);
                }
            }
            subj.next(CssStmt::Supports {
                query: query.clone(),
                inner: inner_stmts,
            });
        }
        AstNode::If { cond, then_branch, else_branch } => {
            let cond_val = eval_expr(cond, ctx, bus);
            let branch = if truthy(&cond_val) { then_branch.as_slice() } else if let Some(eb) = else_branch { eb.as_slice() } else { &[] };
            let child_ctx = ctx.child_scope(4);
            for n in branch {
                if let Some(s) = eval_rule_inner(n, &child_ctx, bus) {
                    subj.next(s);
                }
            }
        }
        AstNode::For { var, from, to, inclusive, body } => {
            let from_val = eval_expr(from, ctx, bus);
            let to_val = eval_expr(to, ctx, bus);
            let from_n = value_to_number(&from_val) as i64;
            let to_n = value_to_number(&to_val) as i64;
            let child_ctx = ctx.child_scope(5);
            let mut i = from_n;
            while if *inclusive { i <= to_n } else { i < to_n } {
                child_ctx.bind_var(var, Value::Number(i as f64));
                for n in body {
                    if let Some(s) = eval_rule_inner(n, &child_ctx, bus) {
                        subj.next(s);
                    }
                }
                i += 1;
            }
        }
        AstNode::Each { vars, list, body } => {
            let list_val = eval_expr(list, ctx, bus);
            let items = match &list_val {
                Value::List(items) => items.clone(),
                v => vec![v.clone()],
            };
            let child_ctx = ctx.child_scope(6);
            for item in items {
                if vars.len() == 1 {
                    child_ctx.bind_var(&vars[0], item);
                }
                for n in body {
                    if let Some(s) = eval_rule_inner(n, &child_ctx, bus) {
                        subj.next(s);
                    }
                }
            }
        }
        AstNode::While { cond, body } => {
            let child_ctx = ctx.child_scope(7);
            let mut iterations = 0;
            loop {
                let cond_val = eval_expr(cond, ctx, bus);
                if !truthy(&cond_val) { break; }
                for n in body {
                    if let Some(s) = eval_rule_inner(n, &child_ctx, bus) {
                        subj.next(s);
                    }
                }
                iterations += 1;
                if iterations > 10000 { break; }
            }
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
        AstNode::MixinCall { name, args } => {
            if let Some(mixin) = bus.lookup_mixin(name) {
                let child_ctx = ctx.child_scope(8);
                for (p, a) in mixin.params.iter().zip(args.iter()) {
                    let val = eval_expr(a, ctx, bus);
                    child_ctx.bind_var(&p.name, val);
                }
                for n in &mixin.body {
                    if let Some(s) = eval_rule_inner(n, &child_ctx, bus) {
                        subj.next(s);
                    }
                }
            }
        }
        AstNode::FunctionCall { name, args } => {
            if let Some(func) = bus.lookup_fn(name) {
                let child_ctx = ctx.child_scope(9);
                for (p, a) in func.params.iter().zip(args.iter()) {
                    let val = eval_expr(a, ctx, bus);
                    child_ctx.bind_var(&p.name, val);
                }
                let mut ret_val = Value::Null;
                for n in &func.body {
                    if let AstNode::Return(val) = n {
                        ret_val = eval_expr(val, &child_ctx, bus);
                        break;
                    }
                }
                let _ = ret_val;
            }
        }
        AstNode::Return(_) => return,
        AstNode::Warn(val) => {
            let warn_val = eval_expr(val, ctx, bus);
            tracing::warn!(value = %warn_value_to_string(&warn_val), "@warn directive");
        }
        AstNode::Debug(val) => {
            let debug_val = eval_expr(val, ctx, bus);
            tracing::debug!(value = %warn_value_to_string(&debug_val), "@debug directive");
        }
        AstNode::Css(stmt) => {
            subj.next(stmt.clone());
        }
        _ => {}
    }
}

fn eval_node_collect(
    node: &AstNode,
    ctx: &EvalContext,
    bus: &CompilerBus,
    stmts: &mut Vec<CssStmt>,
) {
    match node {
        AstNode::Literal(_) => {}
        AstNode::VariableRef { name, .. } => { let _ = ctx.var(name); }
        AstNode::VariableDecl { name, value, .. } => {
            let val = eval_expr(value, ctx, bus);
            ctx.bind_var(name, val);
        }
        AstNode::StyleDecl { property, value } => {
            let val = eval_expr(value, ctx, bus);
            stmts.push(CssStmt::Decl {
                property: property.clone(),
                value: value_to_string(&val),
            });
        }
        AstNode::Rule { selector, inner } => {
            let child_ctx = ctx.child_scope(1);
            let mut inner_stmts = Vec::new();
            for n in inner {
                if let Some(css) = eval_rule_inner(n, &child_ctx, bus) {
                    inner_stmts.push(css);
                }
            }
            stmts.push(CssStmt::Rule {
                selector: selector.clone(),
                inner: inner_stmts,
            });
        }
        AstNode::Media { query, inner } => {
            let child_ctx = ctx.child_scope(2);
            let mut inner_stmts = Vec::new();
            for n in inner {
                if let Some(css) = eval_rule_inner(n, &child_ctx, bus) {
                    inner_stmts.push(css);
                }
            }
            stmts.push(CssStmt::Media {
                query: query.clone(),
                inner: inner_stmts,
            });
        }
        AstNode::Supports { query, inner } => {
            let child_ctx = ctx.child_scope(3);
            let mut inner_stmts = Vec::new();
            for n in inner {
                if let Some(css) = eval_rule_inner(n, &child_ctx, bus) {
                    inner_stmts.push(css);
                }
            }
            stmts.push(CssStmt::Supports {
                query: query.clone(),
                inner: inner_stmts,
            });
        }
        AstNode::If { cond, then_branch, else_branch } => {
            let cond_val = eval_expr(cond, ctx, bus);
            let branch = if truthy(&cond_val) { then_branch.as_slice() } else if let Some(eb) = else_branch { eb.as_slice() } else { &[] };
            let child_ctx = ctx.child_scope(4);
            for n in branch {
                if let Some(s) = eval_rule_inner(n, &child_ctx, bus) {
                    stmts.push(s);
                }
            }
        }
        AstNode::For { var, from, to, inclusive, body } => {
            let from_val = eval_expr(from, ctx, bus);
            let to_val = eval_expr(to, ctx, bus);
            let from_n = value_to_number(&from_val) as i64;
            let to_n = value_to_number(&to_val) as i64;
            let child_ctx = ctx.child_scope(5);
            let mut i = from_n;
            while if *inclusive { i <= to_n } else { i < to_n } {
                child_ctx.bind_var(var, Value::Number(i as f64));
                for n in body {
                    if let Some(s) = eval_rule_inner(n, &child_ctx, bus) {
                        stmts.push(s);
                    }
                }
                i += 1;
            }
        }
        AstNode::Each { vars, list, body } => {
            let list_val = eval_expr(list, ctx, bus);
            let items = match &list_val {
                Value::List(items) => items.clone(),
                v => vec![v.clone()],
            };
            let child_ctx = ctx.child_scope(6);
            for item in items {
                if vars.len() == 1 {
                    child_ctx.bind_var(&vars[0], item);
                }
                for n in body {
                    if let Some(s) = eval_rule_inner(n, &child_ctx, bus) {
                        stmts.push(s);
                    }
                }
            }
        }
        AstNode::While { cond, body } => {
            let child_ctx = ctx.child_scope(7);
            let mut iterations = 0;
            loop {
                let cond_val = eval_expr(cond, ctx, bus);
                if !truthy(&cond_val) { break; }
                for n in body {
                    if let Some(s) = eval_rule_inner(n, &child_ctx, bus) {
                        stmts.push(s);
                    }
                }
                iterations += 1;
                if iterations > 10000 { break; }
            }
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
        AstNode::MixinCall { name, args } => {
            if let Some(mixin) = bus.lookup_mixin(name) {
                let child_ctx = ctx.child_scope(8);
                for (p, a) in mixin.params.iter().zip(args.iter()) {
                    let val = eval_expr(a, ctx, bus);
                    child_ctx.bind_var(&p.name, val);
                }
                for n in &mixin.body {
                    if let Some(s) = eval_rule_inner(n, &child_ctx, bus) {
                        stmts.push(s);
                    }
                }
            }
        }
        AstNode::FunctionCall { name, args } => {
            if let Some(func) = bus.lookup_fn(name) {
                let child_ctx = ctx.child_scope(9);
                for (p, a) in func.params.iter().zip(args.iter()) {
                    let val = eval_expr(a, ctx, bus);
                    child_ctx.bind_var(&p.name, val);
                }
                for n in &func.body {
                    if let AstNode::Return(val) = n {
                        break;
                    }
                }
            }
        }
        AstNode::Return(_) => return,
        AstNode::Warn(val) => {
            let _ = eval_expr(val, ctx, bus);
        }
        AstNode::Debug(val) => {
            let _ = eval_expr(val, ctx, bus);
        }
        AstNode::Css(stmt) => {
            stmts.push(stmt.clone());
        }
        _ => {}
    }
}

fn eval_rule_inner(
    node: &AstNode,
    ctx: &EvalContext,
    bus: &CompilerBus,
) -> Option<CssStmt> {
    match node {
        AstNode::StyleDecl { property, value } => {
            let val = eval_expr(value, ctx, bus);
            Some(CssStmt::Decl {
                property: property.clone(),
                value: value_to_string(&val),
            })
        }
        AstNode::Rule { selector, inner } => {
            let child_ctx = ctx.child_scope(1);
            let mut inner_stmts = Vec::new();
            for n in inner {
                if let Some(css) = eval_rule_inner(n, &child_ctx, bus) {
                    inner_stmts.push(css);
                }
            }
            Some(CssStmt::Rule {
                selector: selector.clone(),
                inner: inner_stmts,
            })
        }
        AstNode::Media { query, inner } => {
            let child_ctx = ctx.child_scope(2);
            let mut inner_stmts = Vec::new();
            for n in inner {
                if let Some(css) = eval_rule_inner(n, &child_ctx, bus) {
                    inner_stmts.push(css);
                }
            }
            Some(CssStmt::Media {
                query: query.clone(),
                inner: inner_stmts,
            })
        }
        AstNode::Supports { query, inner } => {
            let child_ctx = ctx.child_scope(3);
            let mut inner_stmts = Vec::new();
            for n in inner {
                if let Some(css) = eval_rule_inner(n, &child_ctx, bus) {
                    inner_stmts.push(css);
                }
            }
            Some(CssStmt::Supports {
                query: query.clone(),
                inner: inner_stmts,
            })
        }
        AstNode::VariableDecl { name, value, .. } => {
            let val = eval_expr(value, ctx, bus);
            ctx.bind_var(name, val);
            None
        }
        AstNode::MixinCall { name, args } => {
            if let Some(mixin) = bus.lookup_mixin(name) {
                let child_ctx = ctx.child_scope(8);
                for (p, a) in mixin.params.iter().zip(args.iter()) {
                    let val = eval_expr(a, ctx, bus);
                    child_ctx.bind_var(&p.name, val);
                }
                let mut inner_stmts = Vec::new();
                for n in &mixin.body {
                    if let Some(s) = eval_rule_inner(n, &child_ctx, bus) {
                        inner_stmts.push(s);
                    }
                }
                if !inner_stmts.is_empty() {
                    Some(CssStmt::Rule {
                        selector: format!(":*mixin-call:{}", name),
                        inner: inner_stmts,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
        AstNode::If { cond, then_branch, else_branch } => {
            let cond_val = eval_expr(cond, ctx, bus);
            let branch = if truthy(&cond_val) { then_branch.as_slice() } else if let Some(eb) = else_branch { eb.as_slice() } else { &[] };
            let child_ctx = ctx.child_scope(4);
            let mut results = Vec::new();
            for n in branch {
                if let Some(s) = eval_rule_inner(n, &child_ctx, bus) {
                    results.push(s);
                }
            }
            if results.len() == 1 {
                Some(results.into_iter().next().unwrap())
            } else if !results.is_empty() {
                Some(CssStmt::Rule {
                    selector: ":*if-group*".into(),
                    inner: results,
                })
            } else {
                None
            }
        }
        AstNode::For { var, from, to, inclusive, body } => {
            let from_val = eval_expr(from, ctx, bus);
            let to_val = eval_expr(to, ctx, bus);
            let from_n = value_to_number(&from_val) as i64;
            let to_n = value_to_number(&to_val) as i64;
            let child_ctx = ctx.child_scope(5);
            let mut i = from_n;
            let mut results = Vec::new();
            while if *inclusive { i <= to_n } else { i < to_n } {
                child_ctx.bind_var(var, Value::Number(i as f64));
                for n in body {
                    if let Some(s) = eval_rule_inner(n, &child_ctx, bus) {
                        results.push(s);
                    }
                }
                i += 1;
            }
            if results.len() == 1 {
                Some(results.into_iter().next().unwrap())
            } else if !results.is_empty() {
                Some(CssStmt::Rule {
                    selector: ":*for-group*".into(),
                    inner: results,
                })
            } else {
                None
            }
        }
        AstNode::Warn(val) => {
            let _ = eval_expr(val, ctx, bus);
            None
        }
        AstNode::Debug(val) => {
            let _ = eval_expr(val, ctx, bus);
            None
        }
        _ => None,
    }
}

fn eval_expr(expr: &AstNode, ctx: &EvalContext, bus: &CompilerBus) -> Value {
    match expr {
        AstNode::Literal(v) => v.clone(),
        AstNode::VariableRef { name, scope_id } => {
            let look_scope = if *scope_id == 0 { ctx.scope_id() } else { *scope_id };
            ctx.bus().get_var(look_scope, name).unwrap_or_else(|| ctx.var(name).unwrap_or(Value::Null))
        }
        AstNode::BinOp { op, left, right } => {
            let l = eval_expr(left, ctx, bus);
            let r = eval_expr(right, ctx, bus);
            eval_binop(*op, &l, &r)
        }
        AstNode::FunctionCall { name, args } => {
            if let Some(func) = bus.lookup_fn(name) {
                let child_ctx = ctx.child_scope(9);
                for (p, a) in func.params.iter().zip(args.iter()) {
                    let val = eval_expr(a, ctx, bus);
                    child_ctx.bind_var(&p.name, val);
                }
                let mut ret_val = Value::Null;
                for n in &func.body {
                    if let AstNode::Return(val) = n {
                        ret_val = eval_expr(val, &child_ctx, bus);
                        break;
                    }
                }
                ret_val
            } else {
                Value::Null
            }
        }
        AstNode::ListLiteral(items) => {
            Value::List(items.iter().map(|i| eval_expr(i, ctx, bus)).collect())
        }
        AstNode::MapLiteral(entries) => {
            Value::Map(entries.iter().map(|(k, v)| (k.clone(), eval_expr(v, ctx, bus))).collect())
        }
        _ => Value::Null,
    }
}

fn eval_binop(op: BinOp, l: &Value, r: &Value) -> Value {
    match op {
        BinOp::Add => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::String(a), Value::String(b)) => Value::String(format!("{}{}", a, b)),
            (a, b) => Value::String(format!("{}{}", a, b)),
        },
        BinOp::Sub => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            (a, b) => Value::String(format!("{}-{}", a, b)),
        },
        BinOp::Mul => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            _ => Value::Null,
        },
        BinOp::Div => match (l, r) {
            (Value::Number(a), Value::Number(b)) if *b != 0.0 => Value::Number(a / b),
            _ => Value::Null,
        },
        BinOp::Eq => Value::Bool(values_equal(l, r)),
        BinOp::Ne => Value::Bool(!values_equal(l, r)),
        BinOp::Lt => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Value::Bool(a < b),
            _ => Value::Bool(false),
        },
        BinOp::Gt => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Value::Bool(a > b),
            _ => Value::Bool(false),
        },
        BinOp::Le => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Value::Bool(a <= b),
            _ => Value::Bool(false),
        },
        BinOp::Ge => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Value::Bool(a >= b),
            _ => Value::Bool(false),
        },
        BinOp::Mod => match (l, r) {
            (Value::Number(a), Value::Number(b)) if *b != 0.0 => Value::Number(a % b),
            _ => Value::Null,
        },
        BinOp::And => Value::Bool(truthy(l) && truthy(r)),
        BinOp::Or => Value::Bool(truthy(l) || truthy(r)),
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::Number(n) => *n != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::List(items) => !items.is_empty(),
        _ => true,
    }
}

fn value_to_string(v: &Value) -> String {
    v.to_string()
}

fn value_to_number(v: &Value) -> f64 {
    match v {
        Value::Number(n) => *n,
        Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
        Value::Bool(true) => 1.0,
        Value::Bool(false) => 0.0,
        _ => 0.0,
    }
}

fn warn_value_to_string(v: &Value) -> String {
    value_to_string(v)
}
