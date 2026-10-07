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
/// If child contains `&`, replace it with parent. Otherwise, determine the
/// combinator: pseudo-class (`:`), pseudo-element (`::`), class (`.`), ID (`#`),
/// attribute (`[`), or sibling (`+`/`~`) combinators attach directly (compound),
/// while type selectors and others use descendant combinator (space).
/// Supports comma-separated multi-selector parent/child combinations (cartesian product).
pub(crate) fn combine_selectors(parent: &str, child: &str) -> String {
    if child.contains('&') {
        return child.replace('&', parent);
    }
    // Compound selector: starts with pseudo/element/class/ID/attribute — these attach directly
    // Combinators (+, >, ~) need spaces around them: `.a + .btn` not `.a+ .btn`
    let is_compound = child.starts_with(':')
        || child.starts_with('.')
        || child.starts_with('#')
        || child.starts_with('[');
    // Split parents by comma and combine each with child (cartesian product)
    let parents: Vec<&str> = parent.split(',').map(str::trim).collect();
    let children: Vec<&str> = child.split(',').map(str::trim).collect();
    parents.iter().flat_map(|p| {
        children.iter().map(move |c| {
            // Handle & replacement within each child
            if c.contains('&') {
                return c.replace('&', p);
            }
            if is_compound {
                format!("{}{}", p, c)
            } else {
                format!("{} {}", p, c)
            }
        })
    }).collect::<Vec<_>>().join(", ")
}

/// Resolve selector interpolation: replace `#{$var}` and `$var` references with their values.
/// Handles both `#{$var}` (interpolation) and `$var` (variable reference) forms.
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
        // Handle `#{$var}` interpolation: skip the `#{` prefix
        if chars[i] == '#' && i + 1 < chars.len() && chars[i + 1] == '{' {
            // Find the closing `}`
            let close = chars[i + 2..].iter().position(|&c| c == '}').map(|p| i + 2 + p);
            if let Some(end) = close {
                let var_chars = &chars[i + 2..end];
                // Skip leading $ if present
                let var_start = if !var_chars.is_empty() && var_chars[0] == '$' { 1 } else { 0 };
                let var_name: String = var_chars[var_start..].iter().collect();
                if !var_name.is_empty() {
                    let val = ctx.var(&var_name).unwrap_or(Value::Null);
                    result.push_str(&val.to_string());
                }
                i = end + 1;
                continue;
            }
        }
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

/// Resolve a style declaration property from segments to a final string.
///
/// `Var` segments are looked up in the eval context — the variable's value
/// is Display-ed (numbers drop trailing `.0`, strings preserved verbatim).
/// Literal segments are passed through unchanged.
///
/// Uses iterator fold — grows a String by pushing each resolved segment.
#[tracing::instrument(skip(ctx), fields(segments = ?segments))]
pub(crate) fn resolve_property(segments: &[PropSegment], ctx: &EvalContext) -> String {
    segments.iter().fold(String::with_capacity(32), |mut acc, seg| {
        match seg {
            PropSegment::Literal(s) => acc.push_str(s),
            PropSegment::Var(name) => {
                let val = ctx.var(name).unwrap_or(Value::Null);
                acc.push_str(&val.to_string());
            }
        }
        acc
    })
}

pub(crate) fn value_to_string(v: &Value) -> String {
    match v {
        // CSS 列表中的 null 项应被过滤（例如 `solid null` → `solid`）
        Value::List(items) => items
            .iter()
            .filter(|item| **item != Value::Null)
            .map(|item| item.to_string())
            .collect::<Vec<_>>()
            .join(" "),
        _ => v.to_string(),
    }
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
