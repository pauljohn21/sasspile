//! Serializer — converts CSS output tree to string.

use crate::eval::{CssRule, CssTree, ExtendEntry};
use crate::raw_css::expand_css;
use tracing::instrument;

/// Output style for serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStyle {
    /// Multi-line, 2-space indent (default).
    Expanded,
    /// Single line, minimal whitespace.
    Compressed,
}

impl Default for OutputStyle {
    fn default() -> Self {
        Self::Expanded
    }
}

/// Serialize a CSS tree to a string (expanded style).
#[instrument(name = "serialize", skip_all, fields(stage = "serialize"))]
pub fn serialize(css: &CssTree) -> Result<String, crate::error::SassError> {
    serialize_with_style(css, OutputStyle::Expanded)
}

/// Serialize a CSS tree to a string with the given output style.
#[instrument(name = "serialize_with_style", skip_all, fields(stage = "serialize"))]
pub fn serialize_with_style(css: &CssTree, style: OutputStyle) -> Result<String, crate::error::SassError> {
    let span = tracing::info_span!(
        "serialize",
        stage = "serialize",
        rule_count = css.rules.len(),
        extend_count = css.extends.len(),
        style = ?style
    );
    let _enter = span.enter();

    // Apply extends to the CSS tree before serialization
    let rules = if css.extends.is_empty() {
        css.rules.clone()
    } else {
        apply_extends(&css.rules, &css.extends)
    };

    let mut output = String::new();
    for rule in &rules {
        serialize_rule(rule, 0, &mut output, style);
    }

    // Compressed: strip trailing newline
    if style == OutputStyle::Compressed {
        output = output.trim_end().to_string();
    }

    tracing::debug!(stage = "serialize", output_len = output.len(), "serialization complete");
    Ok(output)
}

/// Apply @extend entries to the CSS rules.
///
/// For each extend entry (extender → extendee), find all rules whose selector
/// contains the extendee and add the extender as an additional selector.
fn apply_extends(rules: &[CssRule], extends: &[ExtendEntry]) -> Vec<CssRule> {
    let mut result: Vec<CssRule> = Vec::new();
    for rule in rules {
        let extended = apply_extends_to_rule(rule, extends);
        result.push(extended);
    }
    result
}

/// Recursively apply extends to a single rule and its nested rules.
fn apply_extends_to_rule(rule: &CssRule, extends: &[ExtendEntry]) -> CssRule {
    match rule {
        CssRule::Style { selector, declarations, nested } => {
            // Check if any extend targets this selector
            let mut new_selector = selector.clone();
            for ext in extends {
                // Simple match: if selector contains the extendee as a substring
                if selector.contains(&ext.extendee) {
                    // Add the extender to the selector
                    // e.g. ".foo" → ".foo, .bar"
                    if !new_selector.contains(&ext.extender) {
                        new_selector = format!("{}, {}", new_selector, ext.extender);
                    }
                }
            }
            let new_nested: Vec<CssRule> = nested
                .iter()
                .map(|r| apply_extends_to_rule(r, extends))
                .collect();
            CssRule::Style {
                selector: new_selector,
                declarations: declarations.clone(),
                nested: new_nested,
            }
        }
        CssRule::AtRule { name, value, body } => {
            let new_body: Vec<CssRule> = body
                .iter()
                .map(|r| apply_extends_to_rule(r, extends))
                .collect();
            CssRule::AtRule {
                name: name.clone(),
                value: value.clone(),
                body: new_body,
            }
        }
        other => other.clone(),
    }
}

/// Serialize a single CSS rule.
fn serialize_rule(rule: &CssRule, indent: usize, output: &mut String, style: OutputStyle) {
    let pad = match style {
        OutputStyle::Expanded => "  ".repeat(indent),
        OutputStyle::Compressed => String::new(),
    };
    let indent_str = match style {
        OutputStyle::Expanded => "  ",
        OutputStyle::Compressed => "",
    };
    let line_end = match style {
        OutputStyle::Expanded => "\n",
        OutputStyle::Compressed => "",
    };
    let decl_sep = match style {
        OutputStyle::Expanded => ";",
        OutputStyle::Compressed => ";",
    };

    match rule {
        CssRule::Style { selector, declarations, nested } => {
            // Suppress empty rules (no declarations and no nested rules)
            if declarations.is_empty() && nested.is_empty() {
                return;
            }
            // Also suppress rules with empty selector and no content
            if selector.is_empty() && declarations.is_empty() {
                return;
            }

            let after_selector = match style {
                OutputStyle::Expanded => " {",
                OutputStyle::Compressed => "{",
            };
            let after_prop = match style {
                OutputStyle::Expanded => ": ",
                OutputStyle::Compressed => ":",
            };
            // In compressed mode, only add semicolons between declarations
            // (not after the last one)
            let decl_count = declarations.iter().filter(|(_, v)| !v.is_empty()).count();
            let mut current = 0;

            output.push_str(&pad);
            output.push_str(selector);
            output.push_str(after_selector);
            output.push_str(line_end);

            for (prop, val) in declarations {
                if !val.is_empty() {
                    current += 1;
                    output.push_str(&pad);
                    output.push_str(indent_str);
                    output.push_str(prop);
                    output.push_str(after_prop);
                    output.push_str(val);
                    // In compressed mode, skip semicolon on last declaration
                    if style == OutputStyle::Compressed && current == decl_count {
                        // no semicolon
                    } else {
                        output.push_str(decl_sep);
                    }
                    output.push_str(line_end);
                }
            }

            for nested_rule in nested {
                serialize_rule(nested_rule, indent + 1, output, style);
            }

            output.push_str(&pad);
            output.push('}');
            output.push_str(line_end);
        }
        CssRule::AtRule { name, value, body } => {
            // Suppress empty at-rules
            if body.is_empty() && value.is_empty() {
                // Allow @media with empty body? No — suppress
                return;
            }
            output.push_str(&pad);
            output.push('@');
            output.push_str(name);
            if !value.is_empty() {
                output.push(' ');
                output.push_str(value);
            }
            output.push_str(" {");
            output.push_str(line_end);
            for r in body {
                serialize_rule(r, indent + 1, output, style);
            }
            output.push_str(&pad);
            output.push('}');
            output.push_str(line_end);
        }
        CssRule::Comment(text) => {
            // Preserve block comments in expanded mode
            if style == OutputStyle::Expanded {
                output.push_str(&pad);
                output.push_str("/*");
                output.push_str(text);
                output.push_str("*/");
                output.push_str(line_end);
            }
            // In compressed mode, comments are omitted unless they start with `!`
            if style == OutputStyle::Compressed && text.starts_with('!') {
                output.push_str("/*");
                output.push_str(text);
                output.push_str("*/");
            }
        }
        CssRule::Raw(text) => {
            // Raw CSS text from @use of .css files
            if style == OutputStyle::Expanded {
                // Parse and re-format: expand `a {b: val}` into multi-line
                let formatted = expand_css(text, indent);
                output.push_str(&formatted);
            } else {
                // Compressed: just output the text as-is
                output.push_str(&text);
            }
        }
    }
}
