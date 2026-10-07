// ── Expression evaluation (leaf-level, naturally bounded depth) ─────────
//
// Pure functions: consume &AstNode, produce Value. No observable state.
// Depth is bounded by expression nesting (typically < 20 levels).

use crate::bus::CompilerBus;
use crate::runtime::EvalContext;
use crate::types::*;
use super::builtin;

pub fn eval_expr(expr: &AstNode, ctx: &EvalContext, bus: &CompilerBus) -> Value {
    match expr {
        AstNode::Literal(v) => v.clone(),
        AstNode::VariableRef { name, scope_id } => {
            if *scope_id != 0 {
                if let Some(direct) = ctx.bus().get_var_by_id(*scope_id, name) {
                    return direct;
                }
            }
            ctx.var(name).unwrap_or(Value::Null)
        }
        AstNode::BinOp { op, left, right } => {
            let l = eval_expr(left, ctx, bus);
            let r = eval_expr(right, ctx, bus);
            eval_binop(*op, &l, &r)
        }
        AstNode::FunctionCall { name, args } => {
            let evaled_args: Vec<Value> = args.iter().map(|a| eval_expr(a, ctx, bus)).collect();
            if let Some(result) = builtin::call_builtin(name, &evaled_args) {
                return result;
            }
            if let Some(func) = bus.lookup_fn(name) {
                let child_ctx = ctx.child_scope(9);
                for (p, a) in func.params.iter().zip(evaled_args.iter()) {
                    child_ctx.bind_var(&p.name, a.clone());
                }
                eval_function_body(&func.body, &child_ctx, bus)
            } else {
                Value::Null
            }
        }
        AstNode::UnaryOp { op, expr } => {
            let val = eval_expr(expr, ctx, bus);
            match op {
                UnaryOp::Neg => match val {
                    Value::Number(n, u) => Value::Number(-n, u),
                    _ => Value::Number(0.0, None),
                },
                UnaryOp::Not => Value::Bool(!truthy(&val)),
            }
        }
        AstNode::ListLiteral(items) => {
            Value::List(items.iter().map(|i| eval_expr(i, ctx, bus)).collect())
        }
        AstNode::MapLiteral(entries) => {
            Value::Map(entries.iter().map(|(k, v)| (k.clone(), eval_expr(v, ctx, bus))).collect())
        }
        AstNode::Interpolation(nodes) => {
            let s: String = nodes.iter().map(|n| eval_expr(n, ctx, bus).to_string()).collect();
            Value::String(s)
        }
        _ => Value::Null,
    }
}

fn eval_binop(op: BinOp, l: &Value, r: &Value) -> Value {
    match op {
        BinOp::Add => match (l, r) {
            (Value::Number(a, ua), Value::Number(b, ub)) => {
                let unit = if ua.is_some() { ua.clone() } else { ub.clone() };
                Value::Number(a + b, unit)
            }
            (Value::String(a), Value::String(b)) => Value::String(format!("{}{}", a, b)),
            (a, b) => Value::String(format!("{}{}", a, b)),
        },
        BinOp::Sub => match (l, r) {
            (Value::Number(a, ua), Value::Number(b, ub)) => {
                let unit = if ua.is_some() { ua.clone() } else { ub.clone() };
                Value::Number(a - b, unit)
            }
            (a, b) => Value::String(format!("{}-{}", a, b)),
        },
        BinOp::Mul => match (l, r) {
            (Value::Number(a, ua), Value::Number(b, _)) => Value::Number(a * b, ua.clone()),
            _ => Value::Null,
        },
        BinOp::Div => match (l, r) {
            (Value::Number(a, ua), Value::Number(b, _)) if *b != 0.0 => Value::Number(a / b, ua.clone()),
            _ => Value::Null,
        },
        BinOp::Eq => Value::Bool(values_equal(l, r)),
        BinOp::Ne => Value::Bool(!values_equal(l, r)),
        BinOp::Lt => match (l, r) {
            (Value::Number(a, _), Value::Number(b, _)) => Value::Bool(a < b),
            _ => Value::Bool(false),
        },
        BinOp::Gt => match (l, r) {
            (Value::Number(a, _), Value::Number(b, _)) => Value::Bool(a > b),
            _ => Value::Bool(false),
        },
        BinOp::Le => match (l, r) {
            (Value::Number(a, _), Value::Number(b, _)) => Value::Bool(a <= b),
            _ => Value::Bool(false),
        },
        BinOp::Ge => match (l, r) {
            (Value::Number(a, _), Value::Number(b, _)) => Value::Bool(a >= b),
            _ => Value::Bool(false),
        },
        BinOp::Mod => match (l, r) {
            (Value::Number(a, _), Value::Number(b, _)) if *b != 0.0 => Value::Number(a % b, None),
            _ => Value::Null,
        },
        BinOp::And => Value::Bool(truthy(l) && truthy(r)),
        BinOp::Or => Value::Bool(truthy(l) || truthy(r)),
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(a, _), Value::Number(b, _)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}

pub(crate) fn truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::Number(n, _) => *n != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::List(items) => !items.is_empty(),
        _ => true,
    }
}

/// Combine parent and child selectors following Sass nested rules semantics.
pub(crate) fn combine_selectors(parent: &str, child: &str) -> String {
    if child.contains('&') {
        child.replace('&', parent)
    } else {
        format!("{} {}", parent, child)
    }
}

/// Resolve selector interpolation: replace $var references with their values.
/// Variable names are scanned as `[a-zA-Z0-9_-]+`. If the full name is undefined,
/// progressively shorten at `-` boundaries so `#{$key}-y` resolves `$key` then `-y`.
pub(crate) fn resolve_selector(selector: &str, ctx: &EvalContext) -> String {
    if !selector.contains('$') {
        return selector.to_string();
    }
    let mut result = String::new();
    let chars: Vec<char> = selector.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' {
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_' || chars[end] == '-') {
                end += 1;
            }
            if end > start {
                let full_name: String = chars[start..end].iter().collect();
                // Try the full name first; if undefined, shorten at `-` boundaries
                let val = ctx.var(&full_name)
                    .or_else(|| {
                        // Backward search: $key-y → try $key
                        full_name.rfind('-').and_then(|idx| {
                            if idx > 0 {
                                ctx.var(&full_name[..idx])
                            } else {
                                None
                            }
                        })
                    })
                    .unwrap_or(Value::Null);
                result.push_str(&val.to_string());
                // Only consume the chars that were resolved as a variable
                if ctx.var(&full_name).is_some() {
                    i = end;
                } else if full_name.rfind('-').is_some_and(|idx| idx > 0) && ctx.var(&full_name[..full_name.rfind('-').unwrap()]).is_some() {
                    i = start + full_name.rfind('-').unwrap();
                } else {
                    i = end;
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

pub(crate) fn value_to_string(v: &Value) -> String {
    v.to_string()
}

pub(crate) fn value_to_number(v: &Value) -> f64 {
    match v {
        Value::Number(n, _) => *n,
        Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
        Value::Bool(true) => 1.0,
        Value::Bool(false) => 0.0,
        _ => 0.0,
    }
}

/// Evaluate a user-defined function body.
/// Handles intermediate variable declarations (side effects) and returns the @return value.
fn eval_function_body(body: &[AstNode], ctx: &EvalContext, bus: &CompilerBus) -> Value {
    body.iter()
        .find_map(|node| match node {
            AstNode::Return(val) => Some(eval_expr(val, ctx, bus)),
            AstNode::VariableDecl { name, value, .. } => {
                let val = eval_expr(value, ctx, bus);
                ctx.bind_var(name, val);
                None
            }
            AstNode::If { cond, then_branch, else_branch } => {
                let cond_val = eval_expr(cond, ctx, bus);
                let branch = if truthy(&cond_val) { then_branch } else { else_branch.as_ref().map(|b| b.as_slice()).unwrap_or(&[]) };
                eval_function_body(branch, ctx, bus).into()
            }
            _ => None,
        })
        .unwrap_or(Value::Null)
}
