//! Recursive AST walker — dispatches to specialized transformers.
//!
//! Walks the parsed AST top-down, invoking variable replacement,
//! mixin expansion, and control flow expansion at each node.

use tracing::instrument;

use crate::parser::{
    AtRule, Declaration, Node, Rule, Selector,
};
use crate::Result;

use super::control_flow;
use super::mixins;
use super::variables;
use super::TransformCtx;

/// Expand a list of AST nodes.
#[instrument(skip(ctx, nodes))]
pub(crate) fn expand_nodes(
    ctx: &mut TransformCtx,
    nodes: &[Node],
) -> Result<Vec<Node>> {
    let mut result = Vec::new();

    for node in nodes {
        let expanded = expand_node(ctx, node)?;
        result.extend(expanded);
    }

    Ok(result)
}

/// Expand a single node, returning a list of resulting nodes.
fn expand_node(ctx: &mut TransformCtx, node: &Node) -> Result<Vec<Node>> {
    match node {
        Node::Rule(rule) => expand_rule(ctx, rule),
        Node::Declaration(decl) => expand_declaration(ctx, decl),
        Node::AtRule(at_rule) => expand_at_rule(ctx, at_rule),
        Node::Comment(comment) => Ok(vec![Node::Comment(comment.clone())]),
    }
}

/// Expand a style rule: handle selector interpolation and body expansion.
fn expand_rule(ctx: &mut TransformCtx, rule: &Rule) -> Result<Vec<Node>> {
    // Expand selector (resolve interpolation).
    let selector = expand_selector(&rule.selector, ctx)?;

    // Push local scope for the rule body.
    ctx.symbols.push_local();

    // Expand rule body (declarations + nested rules + at-rules).
    let nodes = expand_nodes(ctx, &rule.nodes)?;

    // Pop local scope.
    ctx.symbols.pop();

    Ok(vec![Node::Rule(Rule {
        selector,
        nodes,
    })])
}

/// Expand a top-level declaration: replace variables with constant values.
fn expand_declaration(ctx: &mut TransformCtx, decl: &Declaration) -> Result<Vec<Node>> {
    // Check if this is a variable declaration ($name: value).
    if decl.is_variable {
        let var_name = &decl.name;
        // Evaluate value and store in symbol table.
        let value = variables::eval_and_store(ctx, var_name, &decl.value)?;
        // Variable declarations don't produce CSS output.
        let _ = value;
        return Ok(vec![]);
    }

    // Regular property declaration — expand value expression.
    let value = variables::expand_expr(ctx, &decl.value)?;
    Ok(vec![Node::Declaration(Declaration {
        name: decl.name.clone(),
        value,
        important: decl.important,
        span: decl.span,
        is_variable: false,
    })])
}

/// Expand an at-rule: dispatch by variant.
fn expand_at_rule(ctx: &mut TransformCtx, at_rule: &AtRule) -> Result<Vec<Node>> {
    match at_rule {
        AtRule::Include(include) => {
            // Expand @include to the mixin body nodes.
            mixins::expand_include(ctx, &include.name, &include.args, &include.body)
        }
        AtRule::If(if_stmt) => {
            control_flow::expand_if(ctx, &if_stmt.condition, &if_stmt.body, if_stmt.else_body.as_deref())
        }
        AtRule::For(for_stmt) => {
            control_flow::expand_for(
                ctx,
                &for_stmt.var,
                &for_stmt.start,
                &for_stmt.end,
                for_stmt.inclusive,
                &for_stmt.body,
            )
        }
        AtRule::Each(each_stmt) => {
            control_flow::expand_each(ctx, &each_stmt.vars, &each_stmt.list, &each_stmt.body)
        }
        AtRule::While(while_stmt) => {
            control_flow::expand_while(ctx, &while_stmt.condition, &while_stmt.body)
        }
        AtRule::Mixin(def) => {
            ctx.register_mixin(def);
            Ok(vec![])
        }
        AtRule::Function(def) => {
            ctx.register_function(def);
            Ok(vec![])
        }
        AtRule::Media(media) => {
            // Expand media query string (resolve variables).
            let query = expand_media_query(&media.query, ctx)?;
            // Push scope for media body.
            ctx.symbols.push_local();
            let nodes = expand_nodes(ctx, &media.body)?;
            ctx.symbols.pop();

            let expanded_media = crate::parser::MediaRule {
                query,
                body: nodes,
            };
            Ok(vec![Node::AtRule(AtRule::Media(expanded_media))])
        }
        AtRule::Supports(supports) => {
            // Supports condition is evaluated at parse time (syntax only).
            ctx.symbols.push_local();
            let nodes = expand_nodes(ctx, &supports.body)?;
            ctx.symbols.pop();

            let expanded = crate::parser::SupportsRule {
                condition: supports.condition.clone(),
                body: nodes,
            };
            Ok(vec![Node::AtRule(AtRule::Supports(expanded))])
        }
        AtRule::Return(_) | AtRule::Content | AtRule::Debug(_) | AtRule::Warn(_) | AtRule::Error(_) => {
            // These don't produce CSS output in transform phase.
            Ok(vec![])
        }
        AtRule::AtRoot(body) => {
            // For now, just expand body (full @at-root needs root-level hoisting).
            expand_nodes(ctx, body)
        }
        AtRule::Extend(_) => {
            // @extend is no-op in CSS output (handled by selector transform).
            Ok(vec![])
        }
        AtRule::Use(_) | AtRule::Forward(_) => {
            // @use/@forward produce no CSS output.
            Ok(vec![])
        }
        AtRule::Else(_) | AtRule::Import(_) => {
            // Pass through (import produces CSS at codegen).
            Ok(vec![Node::AtRule(at_rule.clone())])
        }
    }
}

/// Expand selector interpolation — resolves `${var}` patterns in any selector.
#[tracing::instrument(skip(ctx, selector))]
pub(crate) fn expand_selector(selector: &Selector, ctx: &mut TransformCtx) -> Result<Selector> {
    match selector {
        // Standalone interpolation: .#{...} used as entire selector segment.
        Selector::Interpolation(expr_str) => expand_standalone_interpolation(expr_str, ctx),
        // Class selector may contain embedded `${var}` (e.g. `.el-${name}`).
        Selector::Class(name) => {
            let expanded = expand_selector_string(name, ctx)?;
            Ok(Selector::Class(expanded))
        }
        // ID selector may contain embedded `${var}` (e.g. `#${id}`).
        Selector::Id(name) => {
            let expanded = expand_selector_string(name, ctx)?;
            Ok(Selector::Id(expanded))
        }
        // Type/literal/pseudo may also contain embedded interpolations.
        Selector::Type(name) => {
            let expanded = expand_selector_string(name, ctx)?;
            Ok(Selector::Type(expanded))
        }
        Selector::Literal(name) => {
            let expanded = expand_selector_string(name, ctx)?;
            Ok(Selector::Literal(expanded))
        }
        Selector::Pseudo(name) => {
            let expanded = expand_selector_string(name, ctx)?;
            Ok(Selector::Pseudo(expanded))
        }
        Selector::Attribute(attr) => {
            let expanded = expand_selector_string(attr, ctx)?;
            Ok(Selector::Attribute(expanded))
        }
        Selector::Compound(parts) => {
            let expanded: Result<Vec<Selector>> =
                parts.iter().map(|p| expand_selector(p, ctx)).collect();
            Ok(Selector::Compound(expanded?))
        }
        Selector::Descendant(a, b) => {
            let a = Box::new(expand_selector(a, ctx)?);
            let b = Box::new(expand_selector(b, ctx)?);
            Ok(Selector::Descendant(a, b))
        }
        Selector::Child(a, b) => {
            let a = Box::new(expand_selector(a, ctx)?);
            let b = Box::new(expand_selector(b, ctx)?);
            Ok(Selector::Child(a, b))
        }
        Selector::Adjacent(a, b) => {
            let a = Box::new(expand_selector(a, ctx)?);
            let b = Box::new(expand_selector(b, ctx)?);
            Ok(Selector::Adjacent(a, b))
        }
        Selector::Sibling(a, b) => {
            let a = Box::new(expand_selector(a, ctx)?);
            let b = Box::new(expand_selector(b, ctx)?);
            Ok(Selector::Sibling(a, b))
        }
        Selector::ParentRef(inner) => {
            let inner = Box::new(expand_selector(inner, ctx)?);
            Ok(Selector::ParentRef(inner))
        }
        Selector::Universal => Ok(Selector::Universal),
    }
}

/// Interpolate a selector string, resolving all `${var}` patterns.
fn expand_selector_string(s: &str, ctx: &mut TransformCtx) -> Result<String> {
    let mut result = s.to_string();
    // Iteratively resolve all ${var} patterns.
    while let Some(start) = result.rfind("${") {
        let rest = &result[start..];
        if let Some(end) = rest.find('}') {
            let var_name = &rest[2..end];
            if let Some(entry) = ctx.symbols.lookup(var_name) {
                let value_str = match &entry.value {
                    Some(val) => val.to_string_value(),
                    None => String::new(),
                };
                result.replace_range(start..start + end + 1, &value_str);
            } else {
                return Err(crate::SassError::Compile(format!(
                    "undefined selector variable: ${var_name}"
                )));
            }
        } else {
            break;
        }
    }
    Ok(result)
}

/// Expand a standalone interpolation like `.#{...}`.
fn expand_standalone_interpolation(expr_str: &str, ctx: &mut TransformCtx) -> Result<Selector> {
    // Interpolation format: "${var_name}" (variable) or "#{...}" (complex).
    if let Some(var_name) = expr_str.strip_prefix("${").and_then(|s| s.strip_suffix('}')) {
        // Simple variable reference — resolve through symbol table.
        match ctx.symbols.lookup(var_name) {
            Some(entry) => {
                let value_str = match &entry.value {
                    Some(val) => val.to_string_value(),
                    None => String::new(),
                };
                Ok(Selector::Class(value_str))
            }
            None => Err(crate::SassError::Compile(format!(
                "undefined variable: ${var_name}"
            ))),
        }
    } else {
        // Complex interpolation — keep as literal for now.
        Ok(Selector::Interpolation(expr_str.to_string()))
    }
}

/// Convert a Value to a string suitable for interpolation into a @media or
/// @supports query. Preserves CSS units (e.g. `768px`, `48em`) so that the
/// expanded query is syntactically valid CSS.
fn val_to_media_string(val: &crate::value::Value) -> String {
    use crate::value::{Unit, Value};
    match val {
        Value::Number(n) => {
            let num = if n.value.fract().abs() < f64::EPSILON {
                format!("{}", n.value as i64)
            } else {
                format!("{}", n.value)
            };
            let suffix = match &n.unit {
                Unit::None => "",
                Unit::Em => "em",
                Unit::Rem => "rem",
                Unit::Px => "px",
                Unit::Pt => "pt",
                Unit::Pc => "pc",
                Unit::In => "in",
                Unit::Cm => "cm",
                Unit::Mm => "mm",
                Unit::Q => "q",
                Unit::Deg => "deg",
                Unit::Rad => "rad",
                Unit::Grad => "grad",
                Unit::Turn => "turn",
                Unit::S => "s",
                Unit::Ms => "ms",
                Unit::Hz => "hz",
                Unit::Khz => "khz",
                Unit::Dpi => "dpi",
                Unit::Dpcm => "dpcm",
                Unit::Dppx => "dppx",
                Unit::Percent => "%",
                Unit::Compound(units) => {
                    if let Some(first) = units.first() {
                        // Recurse to render the first unit in the compound.
                        let probe =
                            Value::Number(crate::value::Number::new(0.0, first.clone()));
                        return val_to_media_string(&probe);
                    }
                    ""
                }
            };
            format!("{num}{suffix}")
        }
        _ => val.to_string_value(),
    }
}

/// Expand media query string (resolve variables).
fn expand_media_query(query: &str, ctx: &mut TransformCtx) -> Result<String> {
    // Simple variable substitution in query string.
    let mut result = query.to_string();

    // Find all $variable references and replace them.
    let mut search_start = 0;
    while let Some(pos) = result[search_start..].find('$').map(|p| search_start + p) {
        let rest = &result[pos..].get(1..).unwrap_or("");
        let end = rest
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(rest.len());

        if end == 0 {
            // `$` followed by punctuation — skip.
            search_start = pos + 1;
            continue;
        }

        let var_name = &rest[..end];
        let match_end = pos + 1 + end;

        if let Some(entry) = ctx.symbols.lookup(var_name) {
            if let Some(val) = &entry.value {
                // Use CSS representation to preserve units in query (e.g., 768px).
                let value_str = val_to_media_string(val);
                result.replace_range(pos..match_end, &value_str);
                search_start = pos + value_str.len();
            } else {
                search_start = match_end;
            }
        } else {
            search_start = match_end;
        }
    }

    Ok(result)
}
