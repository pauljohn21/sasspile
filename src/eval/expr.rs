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
            if *scope_id != 0
                && let Some(direct) = ctx.bus().get_var_by_id(*scope_id, name)
            {
                return direct;
            }
            // 变量解析: 先尝试作用域链查找；未找到时 fallback 到 `var(--name)` 保留 CSS 级联引用
            // （而非 Value::Null）。这确保 Bootstrap 3-5 级变量链（如 --bs-dark-text-emphasis → $gray-700 → #292b2c）
            // 中间层缺失时不破坏整个链。
            ctx.var(name).unwrap_or_else(|| {
                // 仅对 CSS 自定义属性格式的变量生成 var() 引用
                if name.starts_with("--") {
                    tracing::debug!(var_name = %name, "unresolved var fallback to var() reference");
                    Value::String(format!("var({})", name))
                } else {
                    Value::Null
                }
            })
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
            Value::List(items.iter().map(|i| eval_expr(i, ctx, bus)).collect(), ListSeparator::Comma)
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
        Value::List(items, _) => !items.is_empty(),
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
    // Compound selector: only pseudo-classes/elements (e.g., :hover, ::before, :not())
    // attach directly to parent without space. All other selectors (.class, #id, [attr], tag)
    // produce descendant combinator when nested (Sass spec: `.card { .title }` → `.card .title`)
    let is_compound = child.starts_with(':');
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
///
/// Recursively resolves variable values that themselves contain `$var` or `#{$var}` references.
pub(crate) fn resolve_selector(selector: &str, ctx: &EvalContext) -> String {
    resolve_selector_recursive(selector, ctx, 0)
}

/// Recursive helper with depth limit to prevent infinite loops from circular references.
fn resolve_selector_recursive(selector: &str, ctx: &EvalContext, depth: usize) -> String {
    const MAX_DEPTH: usize = 5;
    if !selector.contains('$') || depth >= MAX_DEPTH {
        return selector.to_string();
    }
    let mut result = String::new();
    let chars: Vec<char> = selector.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        // Handle `#{...}` interpolation in selector
        if chars[i] == '#' && i + 1 < chars.len() && chars[i + 1] == '{' {
            // Find the closing `}` respecting nested braces
            let mut depth_brace = 1i32;
            let mut end = None;
            for j in (i + 2)..chars.len() {
                if chars[j] == '#' && j + 1 < chars.len() && chars[j + 1] == '{' {
                    depth_brace += 1;
                } else if chars[j] == '}' {
                    depth_brace -= 1;
                    if depth_brace == 0 {
                        end = Some(j);
                        break;
                    }
                }
            }
            if let Some(end) = end {
                let inner: String = chars[i + 2..end].iter().collect();
                // Evaluate the interpolation expression:
                // Replace all $var references with their values, then
                // handle string concatenation (+) and nested interpolation.
                let resolved = eval_interp_expression(&inner, ctx, depth + 1);
                result.push_str(&resolved);
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
                let val_str = val.to_string();
                // Recursively resolve the value if it contains variable references
                if val_str.contains('$') {
                    result.push_str(&resolve_selector_recursive(&val_str, ctx, depth + 1));
                } else {
                    result.push_str(&val_str);
                }
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

/// Evaluate an interpolation expression like `$a + $b + $c` or `$class`.
/// Handles: simple $var, $var + $var (string concat), nested #{...}, and literal strings.
fn eval_interp_expression(expr: &str, ctx: &EvalContext, depth: usize) -> String {
    // Split on + (string concatenation) at top level
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut brace_depth = 0i32;
    let mut pound_brace = false;
    for ch in expr.chars() {
        if pound_brace {
            pound_brace = false;
            if ch == '{' {
                brace_depth += 1;
                current.push('#');
                current.push('{');
                continue;
            } else {
                current.push('#');
            }
        }
        if ch == '#' && brace_depth == 0 {
            pound_brace = true;
            continue;
        }
        if ch == '{' && brace_depth > 0 {
            brace_depth += 1;
            current.push(ch);
            continue;
        }
        if ch == '}' && brace_depth > 0 {
            brace_depth -= 1;
            current.push(ch);
            continue;
        }
        if ch == '+' && brace_depth == 0 {
            parts.push(current.trim().to_string());
            current = String::new();
            continue;
        }
        current.push(ch);
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    if parts.is_empty() {
        return String::new();
    }
    // Evaluate each part and concatenate
    parts.iter()
        .map(|part| eval_interp_part(part, ctx, depth))
        .collect()
}

/// Evaluate a single part of an interpolation expression.
/// Handles: $var, "literal", #{...}, function calls, and nested expressions.
fn eval_interp_part(part: &str, ctx: &EvalContext, depth: usize) -> String {
    let trimmed = part.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // Literal string
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    // Recursive #{...}
    if trimmed.starts_with("#{") && trimmed.ends_with('}') {
        let inner = &trimmed[2..trimmed.len() - 1];
        return eval_interp_expression(inner, ctx, depth + 1);
    }
    // Function call: funcname(arg1, arg2, ...)
    if let Some(rest) = trimmed.strip_suffix(')')
        && let Some(paren_pos) = rest.find('(')
    {
        let func_name = rest[..paren_pos].trim();
        let args_str = rest[paren_pos + 1..].trim();
        if !func_name.is_empty() && is_identifier(func_name) {
            return eval_interp_function_call(func_name, args_str, ctx, depth);
        }
    }
    // $var reference: handle the case where $var might appear standalone
    if let Some(var_name) = trimmed.strip_prefix('$')
        && !var_name.contains(' ') && !var_name.contains('#')
    {
        let val = ctx.var(var_name).unwrap_or(Value::Null);
        let val_str = val.to_string();
        if val_str.contains('$') {
            return resolve_selector_recursive(&val_str, ctx, depth);
        }
        return val_str;
    }
    // Fallback: try resolving the whole thing as a selector (handles inline $var+... cases)
    resolve_selector_recursive(trimmed, ctx, depth)
}

/// Check if a string is a valid identifier (for function names).
fn is_identifier(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Evaluate a function call in interpolation context: funcname(arg1, arg2, ...)
fn eval_interp_function_call(func_name: &str, args_str: &str, ctx: &EvalContext, depth: usize) -> String {
    // Split arguments by comma (respecting nested parens would need a real parser)
    let arg_parts: Vec<&str> = args_str.split(',').map(str::trim).collect();
    let evaled_args: Vec<Value> = arg_parts.iter().map(|arg| {
        // Each argument can be a $var, a nested function call, or a literal
        let arg = arg.trim();
        if let Some(var_name) = arg.strip_prefix('$') {
            ctx.var(var_name).unwrap_or(Value::Null)
        } else if arg.ends_with(')') && arg.contains('(') {
            // Nested function call — evaluate and convert result to value
            let result = eval_interp_part(arg, ctx, depth);
            Value::String(result)
        } else {
            // Try as number or string literal
            if let Ok(n) = arg.parse::<f64>() {
                Value::Number(n, None)
            } else {
                Value::String(arg.to_string())
            }
        }
    }).collect();

    if let Some(result) = builtin::call_builtin(func_name, &evaled_args) {
        result.to_string()
    } else {
        String::new()
    }
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
        Value::List(items, _) => items
            .iter()
            .filter(|item| **item != Value::Null)
            .map(|item| item.to_string())
            .collect::<Vec<_>>()
            .join(" "),
        // Maps used directly as CSS values serialize as null (Sass behavior)
        Value::Map(_) => "null".to_string(),
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
