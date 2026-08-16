//! Rule expansion — transforms nested SASS AST to flat CSS rules.
//!
//! Combines nested selectors using the SCSS parent reference (&) semantics,
//! evaluates expressions, and produces flat CSS output.

use tracing::instrument;

use crate::parser::{
    AtRule, Expr, Node, Rule, Selector, Stylesheet,
};
use crate::value::Value;

use super::ast::{CssDeclaration, CssDocument, CssRule};

/// Expand a full stylesheet into a flat CSS document.
#[instrument(skip(stylesheet, doc))]
pub fn expand_stylesheet(stylesheet: &Stylesheet, doc: &mut CssDocument) {
    expand_nodes(stylesheet.nodes.as_slice(), doc, "");
}

/// Expand a slice of nodes.
pub fn expand_nodes(nodes: &[Node], doc: &mut CssDocument, parent_sel: &str) {
    for node in nodes {
        expand_node(node, doc, parent_sel);
    }
}

fn expand_node(node: &Node, doc: &mut CssDocument, parent_sel: &str) {
    match node {
        Node::Rule(rule) => {
            expand_rule(rule, doc, parent_sel);
        }
        Node::Declaration(decl) => {
            // Top-level declarations are unusual but we handle by wrapping.
            // This case typically doesn't occur in well-formed SCSS.
            let value = eval_expr_simple(&decl.value);
            let css_decl = CssDeclaration::new(
                decl.name.clone(),
                value,
                decl.important,
            );
            let selector = if parent_sel.is_empty() {
                ":root".to_string()
            } else {
                parent_sel.to_string()
            };
            doc.add_rule(CssRule {
                selector,
                declarations: vec![css_decl],
                children: Vec::new(),
            });
        }
        Node::AtRule(at_rule) => {
            expand_atrule(at_rule, doc, parent_sel);
        }
        Node::Comment(_) => {
            // Comments are stripped in CSS output by default.
        }
    }
}

fn expand_rule(rule: &Rule, doc: &mut CssDocument, parent_sel: &str) {
    let combined = combine_selectors(parent_sel, &rule.selector);
    let mut css_rule = CssRule {
        selector: combined.clone(),
        declarations: Vec::new(),
        children: Vec::new(),
    };

    for node in &rule.nodes {
        match node {
            Node::Declaration(decl) => {
                let value = eval_expr_simple(&decl.value);
                css_rule.declarations.push(CssDeclaration::new(
                    decl.name.clone(),
                    value,
                    decl.important,
                ));
            }
            Node::Rule(nested_rule) => {
                expand_rule(nested_rule, doc, &combined);
            }
            Node::AtRule(at_rule) => {
                expand_atrule(at_rule, doc, &combined);
            }
            Node::Comment(_) => {}
        }
    }

    if !css_rule.declarations.is_empty() {
        doc.add_rule(css_rule);
    }
}

/// Combine parent selector with child selector following SCSS rules.
pub fn combine_selectors(parent: &str, child: &Selector) -> String {
    let child_str = selector_to_string(child);
    if parent.is_empty() {
        return child_str;
    }
    // If child starts with &, perform interpolation.
    if let Some(stripped) = child_str.strip_prefix('&') {
        format!("{parent}{}", stripped)
    } else if child_str.contains('&') {
        child_str.replace('&', parent)
    } else {
        // Descendant combinator.
        format!("{parent} {child_str}")
    }
}

/// Convert a Selector to a CSS string.
pub fn selector_to_string(selector: &Selector) -> String {
    match selector {
        Selector::Type(name) => name.clone(),
        Selector::Class(name) => format!(".{name}"),
        Selector::Id(name) => format!("#{name}"),
        Selector::Attribute(attr) => format!("[{attr}]"),
        Selector::Pseudo(pseudo) => format!(":{pseudo}"),
        Selector::ParentRef(inner) => format!("&{}", selector_to_string(inner)),
        Selector::Compound(parts) => {
            parts.iter().map(selector_to_string).collect::<Vec<_>>().join("")
        }
        Selector::Descendant(a, b) => {
            format!("{} {}", selector_to_string(a), selector_to_string(b))
        }
        Selector::Child(a, b) => {
            format!("{} > {}", selector_to_string(a), selector_to_string(b))
        }
        Selector::Adjacent(a, b) => {
            format!("{} + {}", selector_to_string(a), selector_to_string(b))
        }
        Selector::Sibling(a, b) => {
            format!("{} ~ {}", selector_to_string(a), selector_to_string(b))
        }
        Selector::Interpolation(text) => format!("#{{{text}}}"),
        Selector::Universal => "*".to_string(),
        Selector::Literal(text) => text.clone(),
    }
}

/// Simple expression evaluator for CSS generation time (backwards compatibility).
/// For full evaluation, the pipeline should evaluate the AST first.
pub fn eval_expr_simple(expr: &Expr) -> Value {
    match expr {
        Expr::Number(val, unit) => {
            let u = unit.as_deref().and_then(crate::value::Unit::parse)
                .unwrap_or(crate::value::Unit::None);
            Value::Number(crate::value::Number::new(*val, u))
        }
        Expr::String(s) => Value::String(s.clone(), crate::value::Quoted::Quoted),
        Expr::Boolean(b) => Value::Boolean(*b),
        Expr::Null => Value::Null,
        Expr::Color(c) => Value::Color(crate::value::SassColor::from_hex(*c)),
        Expr::Url(u) => Value::String(u.clone(), crate::value::Quoted::Unquoted),
        Expr::List(items) => {
            let evaluated: Vec<Value> = items.iter().map(eval_expr_simple).collect();
            Value::List(evaluated, crate::value::Separator::Comma)
        }
        Expr::SpaceList(items) => {
            let evaluated: Vec<Value> = items.iter().map(eval_expr_simple).collect();
            Value::List(evaluated, crate::value::Separator::Space)
        }
        Expr::SlashList(items) => {
            let evaluated: Vec<Value> = items.iter().map(eval_expr_simple).collect();
            Value::List(evaluated, crate::value::Separator::Slash)
        }
        Expr::Parens(inner) => eval_expr_simple(inner),
        Expr::Interpolation(_) => Value::String("#{...}".into(), crate::value::Quoted::Unquoted),
        Expr::Variable(name) => {
            // Variables not evaluated — output as css custom property fallback.
            // In full pipeline, these would be resolved by EvalContext.
            Value::String(format!("${name}"), crate::value::Quoted::Unquoted)
        }
        Expr::Call(name, args) => {
            let arg_values: Vec<String> = args.iter().map(|a| eval_expr_simple(a).to_css_string()).collect();
            Value::String(format!("{}({})", name, arg_values.join(", ")), crate::value::Quoted::Unquoted)
        }
            Expr::Binary(op, lhs, rhs) => {
            let l = eval_expr_simple(lhs);
            let r = eval_expr_simple(rhs);
            format_binary(op, &l, &r)
        }
        Expr::Unary(op, operand) => {
            let v = eval_expr_simple(operand);
            format_unary(op, &v)
        }
        Expr::Map(entries) => {
            let map: Vec<(Value, Value)> = entries
                .iter()
                .map(|(k, v)| (eval_expr_simple(k), eval_expr_simple(v)))
                .collect();
            Value::Map(map)
        }
        Expr::NamedArg(name, value) => {
            let v = eval_expr_simple(value);
            Value::String(format!("{name}: {}", v.to_css_string()), crate::value::Quoted::Unquoted)
        }
        Expr::Spread(inner) => {
            let v = eval_expr_simple(inner);
            Value::String(format!("{}...", v.to_css_string()), crate::value::Quoted::Unquoted)
        }
    }
}

fn format_binary(op: &crate::parser::BinaryOp, l: &Value, r: &Value) -> Value {
    let op_str = match op {
        crate::parser::BinaryOp::Add => "+",
        crate::parser::BinaryOp::Sub => "-",
        crate::parser::BinaryOp::Mul => "*",
        crate::parser::BinaryOp::Div => "/",
        crate::parser::BinaryOp::Mod => "%",
        _ => " ",
    };
    Value::String(
        format!("{} {op_str} {}", l.to_css_string(), r.to_css_string()),
        crate::value::Quoted::Unquoted,
    )
}

fn format_unary(op: &crate::parser::UnaryOp, v: &Value) -> Value {
    match op {
        crate::parser::UnaryOp::Neg => Value::String(format!("-{}", v.to_css_string()), crate::value::Quoted::Unquoted),
        crate::parser::UnaryOp::Not => Value::String(format!("not {}", v.to_css_string()), crate::value::Quoted::Unquoted),
    }
}

/// Expand an at-rule.
fn expand_atrule(at_rule: &AtRule, doc: &mut CssDocument, parent_sel: &str) {
    super::atrules::expand_atrule(at_rule, doc, parent_sel);
}
