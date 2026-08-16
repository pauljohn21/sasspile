//! At-rule output generation.
//!
//! Handles @media, @supports, @import, @keyframes, and unknown at-rules.

use crate::parser::{
    AtRule, ForStmt, IfStmt, MediaRule, SupportsRule, WhileStmt, EachStmt,
};
use crate::css::ast::{CssAtRule, CssDocument, CssRule};

/// Expand an at-rule into the CSS document.
pub fn expand_atrule(at_rule: &AtRule, doc: &mut CssDocument, parent_sel: &str) {
    match at_rule {
        AtRule::Media(media) => {
            expand_media_rule(media, doc, parent_sel);
        }
        AtRule::Supports(supports) => {
            expand_supports_rule(supports, doc, parent_sel);
        }
        AtRule::Import(import_rule) => {
            // Generate @import statements directly.
            for url in &import_rule.urls {
                let atrule = CssAtRule {
                    name: "import".into(),
                    query: format!("\"{url}\""),
                    children: Vec::new(),
                    nested_atrules: Vec::new(),
                };
                doc.add_atrule(atrule);
            }
            let _ = parent_sel; // unused
        }
        AtRule::Use(_) | AtRule::Forward(_) => {
            // @use and @forward don't produce CSS output directly.
        }
        AtRule::If(if_stmt) => {
            expand_if_rule(if_stmt, doc, parent_sel);
        }
        AtRule::Else(body) => {
            // Treat @else body as nested expansion.
            crate::css::rules::expand_nodes(body, doc, parent_sel);
        }
        AtRule::For(for_stmt) => {
            expand_for_rule(for_stmt, doc, parent_sel);
        }
        AtRule::Each(each_stmt) => {
            expand_each_rule(each_stmt, doc, parent_sel);
        }
        AtRule::While(while_stmt) => {
            expand_while_rule(while_stmt, doc, parent_sel);
        }
        AtRule::AtRoot(body) => {
            // @at-root emits nodes at the document root level.
            crate::css::rules::expand_nodes(body, doc, "");
        }
        AtRule::Extend(_) => {
            // @extend modifies selectors; handled in full pipeline.
            // For now, skip (no CSS output from @extend alone).
        }
        AtRule::Content => {
            // @content is resolved at mixin include time.
        }
        AtRule::Debug(expr) | AtRule::Warn(expr) | AtRule::Error(expr) => {
            // In CSS output, these are typically stripped.
            let _ = expr;
        }
        AtRule::Mixin(_) | AtRule::Function(_) | AtRule::Return(_) => {
            // Definitions don't produce CSS.
        }
        AtRule::Include(include) => {
            // @include without body — typically handled in eval phase.
            // Output comments for now (placeholder for full expansion).
            if !include.body.is_empty() {
                crate::css::rules::expand_nodes(&include.body, doc, parent_sel);
            }
        }
    }
}

fn expand_media_rule(media: &MediaRule, doc: &mut CssDocument, parent_sel: &str) {
    let mut atrule = CssAtRule {
        name: "media".into(),
        query: media.query.clone(),
        children: Vec::new(),
        nested_atrules: Vec::new(),
    };
    // Recursively expand nested rules.
    for node in &media.body {
        if let crate::parser::Node::Rule(rule) = node {
            let combined = crate::css::rules::combine_selectors(parent_sel, &rule.selector);
            let mut css_rule = CssRule {
                selector: combined,
                declarations: Vec::new(),
                children: Vec::new(),
            };
            expand_rule_body(rule, &mut css_rule, parent_sel);
            if !css_rule.declarations.is_empty() {
                atrule.children.push(css_rule);
            }
        }
    }
    if !atrule.children.is_empty() {
        doc.add_atrule(atrule);
    }
}

fn expand_supports_rule(supports: &SupportsRule, doc: &mut CssDocument, parent_sel: &str) {
    let mut atrule = CssAtRule {
        name: "supports".into(),
        query: supports.condition.clone(),
        children: Vec::new(),
        nested_atrules: Vec::new(),
    };
    for node in &supports.body {
        if let crate::parser::Node::Rule(rule) = node {
            let combined = crate::css::rules::combine_selectors(parent_sel, &rule.selector);
            let mut css_rule = CssRule {
                selector: combined,
                declarations: Vec::new(),
                children: Vec::new(),
            };
            expand_rule_body(rule, &mut css_rule, parent_sel);
            if !css_rule.declarations.is_empty() {
                atrule.children.push(css_rule);
            }
        }
    }
    if !atrule.children.is_empty() {
        doc.add_atrule(atrule);
    }
}

fn expand_rule_body(
    rule: &crate::parser::Rule,
    css_rule: &mut CssRule,
    _parent_sel: &str,
) {
    for node in &rule.nodes {
        match node {
            crate::parser::Node::Declaration(decl) => {
                let value = crate::css::rules::eval_expr_simple(&decl.value);
                css_rule.declarations.push(
                    crate::css::ast::CssDeclaration::new(
                        decl.name.clone(),
                        value,
                        decl.important,
                    ),
                );
            }
            crate::parser::Node::Rule(nested_rule) => {
                let combined = crate::css::rules::combine_selectors(&css_rule.selector, &nested_rule.selector);
                let mut child_rule = CssRule {
                    selector: combined,
                    declarations: Vec::new(),
                    children: Vec::new(),
                };
                expand_rule_body(nested_rule, &mut child_rule, &css_rule.selector);
                if !child_rule.declarations.is_empty() {
                    css_rule.children.push(child_rule);
                }
            }
            _ => {}
        }
    }
}

fn expand_if_rule(if_stmt: &IfStmt, doc: &mut CssDocument, parent_sel: &str) {
    // Simple heuristic: always emit the if body (not the else).
    // Full implementation would evaluate the condition.
    crate::css::rules::expand_nodes(&if_stmt.body, doc, parent_sel);
    if let Some(else_body) = &if_stmt.else_body {
        crate::css::rules::expand_nodes(else_body, doc, parent_sel);
    }
}

fn expand_for_rule(_for_stmt: &ForStmt, _doc: &mut CssDocument, _parent_sel: &str) {
    // @for requires evaluation to iterate.
    // In this simplified version, @for output is a placeholder.
    // Full pipeline expands @for after evaluation.
    tracing::debug!("@for expansion requires evaluator integration");
}

fn expand_each_rule(_each_stmt: &EachStmt, _doc: &mut CssDocument, _parent_sel: &str) {
    tracing::debug!("@each expansion requires evaluator integration");
}

fn expand_while_rule(_while_stmt: &WhileStmt, _doc: &mut CssDocument, _parent_sel: &str) {
    tracing::debug!("@while expansion requires evaluator integration");
}

// Re-export internal function so rules.rs can delegate.
#[allow(dead_code)]
pub(crate) fn expand_atrule_internal(
    at_rule: &AtRule,
    doc: &mut CssDocument,
    parent_sel: &str,
) {
    expand_atrule(at_rule, doc, parent_sel);
}
