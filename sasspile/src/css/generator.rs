//! CSS formatter — controls whitespace and formatting output.

use super::ast::{CssAtRule, CssDocument, CssRule};

/// Output formatting style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStyle {
    /// Nested indented with 2-space indent (Sass original).
    Nested,
    /// Expanded readable CSS (4-space indent, one decl per line).
    Expanded,
    /// Compact (single line per rule).
    Compact,
    /// Compressed (no whitespace, minimal output).
    Compressed,
}

/// Set indent width.
const INDENT: usize = 2;

/// Format a CSS document to a string.
pub fn format(doc: &CssDocument, style: OutputStyle) -> String {
    match style {
        OutputStyle::Compressed => format_compressed(doc),
        OutputStyle::Expanded => format_expanded(doc, INDENT),
        OutputStyle::Compact => format_compact(doc),
        OutputStyle::Nested => format_expanded(doc, INDENT),
    }
}

fn format_expanded(doc: &CssDocument, indent: usize) -> String {
    let mut out = String::new();
    for rule in &doc.rules {
        format_rule(rule, &mut out, 0, indent);
    }
    for atrule in &doc.atrules {
        format_atrule(atrule, &mut out, 0, indent);
    }
    // Ensure trailing newline.
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn format_compact(doc: &CssDocument) -> String {
    let mut out = String::new();
    for rule in &doc.rules {
        format_rule_compact(rule, &mut out, 0);
    }
    for atrule in &doc.atrules {
        format_atrule_compact(atrule, &mut out, 0);
    }
    out
}

fn format_compressed(doc: &CssDocument) -> String {
    let mut out = String::new();
    for rule in &doc.rules {
        format_rule_compressed(rule, &mut out, 0);
    }
    for atrule in &doc.atrules {
        format_atrule_compressed(atrule, &mut out, 0);
    }
    out
}

fn format_rule(rule: &CssRule, out: &mut String, depth: usize, indent: usize) {
    let prefix = " ".repeat(depth * indent);
    // Inline children declarations into parent selector if nested.
    if !rule.declarations.is_empty() {
        out.push_str(&prefix);
        out.push_str(&rule.selector);
        out.push_str(" {\n");
        for decl in &rule.declarations {
            out.push_str(&" ".repeat((depth + 1) * indent));
            out.push_str(&decl.name);
            out.push_str(": ");
            out.push_str(&decl.value);
            if decl.important {
                out.push_str(" !important");
            }
            out.push_str(";\n");
        }
        out.push_str(&prefix);
        out.push_str("}\n");
    }
    // Output nested rules.
    for child in &rule.children {
        format_rule(child, out, depth + 1, indent);
    }
}

fn format_rule_compact(rule: &CssRule, out: &mut String, depth: usize) {
    let prefix = " ".repeat(depth * 2);
    if !rule.declarations.is_empty() {
        out.push_str(&prefix);
        out.push_str(&rule.selector);
        out.push_str(" { ");
        for (i, decl) in rule.declarations.iter().enumerate() {
            if i > 0 {
                out.push_str("; ");
            }
            out.push_str(&decl.name);
            out.push_str(": ");
            out.push_str(&decl.value);
            if decl.important {
                out.push_str(" !important");
            }
        }
        out.push_str("; }\n");
    }
    for child in &rule.children {
        format_rule_compact(child, out, depth + 1);
    }
}

fn format_rule_compressed(rule: &CssRule, out: &mut String, _depth: usize) {
    if !rule.declarations.is_empty() {
        out.push_str(&rule.selector);
        out.push('{');
        for decl in &rule.declarations {
            out.push_str(&decl.name);
            out.push(':');
            out.push_str(&decl.value);
            if decl.important {
                out.push_str("!important");
            }
            out.push(';');
        }
        out.push('}');
    }
    for child in &rule.children {
        format_rule_compressed(child, out, 0);
    }
}

fn format_atrule(atrule: &CssAtRule, out: &mut String, depth: usize, indent: usize) {
    let prefix = " ".repeat(depth * indent);
    out.push_str(&prefix);
    out.push('@');
    out.push_str(&atrule.name);
    out.push(' ');
    out.push_str(&atrule.query);
    out.push_str(" {\n");
    for rule in &atrule.children {
        format_rule(rule, out, depth + 1, indent);
    }
    out.push_str(&prefix);
    out.push_str("}\n");
}

fn format_atrule_compact(atrule: &CssAtRule, out: &mut String, depth: usize) {
    let prefix = " ".repeat(depth * 2);
    out.push_str(&prefix);
    out.push('@');
    out.push_str(&atrule.name);
    out.push(' ');
    out.push_str(&atrule.query);
    out.push_str(" {\n");
    for rule in &atrule.children {
        format_rule_compact(rule, out, depth + 1);
    }
    out.push_str(&prefix);
    out.push_str("}\n");
}

fn format_atrule_compressed(atrule: &CssAtRule, out: &mut String, _depth: usize) {
    out.push('@');
    out.push_str(&atrule.name);
    out.push(' ');
    out.push_str(&atrule.query);
    out.push('{');
    for rule in &atrule.children {
        format_rule_compressed(rule, out, 0);
    }
    out.push('}');
}
