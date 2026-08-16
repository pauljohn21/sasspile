//! At-rule output generation.
//!
//! Handles @media, @supports, @import, and unknown at-rules.
//! Control flow (@if, @for, @each, @while) is expanded by the transform stage,
//! so it should not appear here.

use crate::parser::{
    AtRule, MediaRule, SupportsRule,
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
        AtRule::AtRoot(body) => {
            // @at-root emits nodes at the document root level.
            crate::css::rules::expand_nodes(body, doc, "");
        }
        AtRule::Extend(_) => {
            // @extend modifies selectors; handled in transform stage.
        }
        AtRule::Else(_body) => {
            // @else is handled as part of @if in transform stage.
            // If it reaches here, expand as fallback.
        }
        AtRule::Include(include) => {
            // @include is expanded in transform stage.
            // If it reaches here, include body nodes directly.
            if !include.body.is_empty() {
                crate::css::rules::expand_nodes(&include.body, doc, parent_sel);
            }
        }
        // Control flow should have been expanded by transform stage.
        // Log a warning if any reach here.
        AtRule::If(_) | AtRule::For(_) | AtRule::Each(_) | AtRule::While(_) => {
            tracing::warn!("unexpected control-flow at-rule in CSS generation — should have been expanded by transform stage");
        }
        // Definitions and non-CSS at-rules produce no output.
        AtRule::Mixin(_) | AtRule::Function(_) | AtRule::Return(_)
        | AtRule::Content | AtRule::Debug(_) | AtRule::Warn(_) | AtRule::Error(_) => {}
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
