//! @extend validation — checks that extension targets exist.
//!
//! Verifies that every `@extend <selector>` references a selector
//! that exists in the stylesheet or an imported module.

use crate::diagnostics::{Diagnostic, Diagnostics};
use crate::{Node, Selector, Stylesheet};

/// Registry of all defined selectors in the stylesheet.
#[derive(Debug, Clone, Default)]
pub struct SelectorRegistry {
    /// Set of all selectors that can be extended.
    selectors: Vec<Selector>,
    /// Map from simple selector strings to their full selector entries.
    by_class: Vec<(String, Selector)>,
    by_id: Vec<(String, Selector)>,
    by_type: Vec<(String, Selector)>,
}

impl SelectorRegistry {
    /// Create an empty selector registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a registry from a parsed stylesheet.
    pub fn from_stylesheet(stylesheet: &Stylesheet) -> Self {
        let mut registry = Self::new();
        registry.collect_from_nodes(&stylesheet.nodes);
        registry
    }

    /// Collect all extendable selectors from a node list.
    fn collect_from_nodes(&mut self, nodes: &[Node]) {
        for node in nodes {
            match node {
                Node::Rule(rule) => {
                    self.register_selector(&rule.selector);
                    self.collect_from_nodes(&rule.nodes);
                }
                Node::AtRule(at_rule) => {
                    use crate::AtRule;
                    match at_rule {
                        AtRule::Media(media) => {
                            self.collect_from_nodes(&media.body);
                        }
                        AtRule::Supports(supports) => {
                            self.collect_from_nodes(&supports.body);
                        }
                        AtRule::AtRoot(root_nodes) => {
                            self.collect_from_nodes(root_nodes);
                        }
                        AtRule::If(if_stmt) => {
                            self.collect_from_nodes(&if_stmt.body);
                            if let Some(else_body) = &if_stmt.else_body {
                                self.collect_from_nodes(else_body);
                            }
                        }
                        AtRule::For(for_stmt) => {
                            self.collect_from_nodes(&for_stmt.body);
                        }
                        AtRule::Each(each_stmt) => {
                            self.collect_from_nodes(&each_stmt.body);
                        }
                        AtRule::While(while_stmt) => {
                            self.collect_from_nodes(&while_stmt.body);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    /// Register a single selector and build indexes.
    pub fn register_selector(&mut self, selector: &Selector) {
        // Extract simple selectors for fast lookup.
        match selector {
            Selector::Class(name) => {
                self.by_class.push((name.clone(), selector.clone()));
            }
            Selector::Id(name) => {
                self.by_id.push((name.clone(), selector.clone()));
            }
            Selector::Type(name) => {
                self.by_type.push((name.clone(), selector.clone()));
            }
            Selector::Compound(parts) => {
                for part in parts {
                    self.register_selector(part);
                }
            }
            Selector::ParentRef(inner) => {
                self.register_selector(inner);
            }
            _ => {}
        }
        self.selectors.push(selector.clone());
    }

    /// Check if a class selector with this name exists.
    pub fn has_class(&self, name: &str) -> bool {
        self.by_class.iter().any(|(n, _)| n == name)
    }

    /// Check if an ID selector with this name exists.
    /// (%placeholder selectors).
    pub fn has_id(&self, name: &str) -> bool {
        self.by_id.iter().any(|(n, _)| n == name)
    }

    /// Check if a type selector with this name exists.
    pub fn has_type(&self, name: &str) -> bool {
        self.by_type.iter().any(|(n, _)| n == name)
    }

    /// Check if a selector has any valid extend target, collecting diagnostics
    /// for any part that does not resolve.
    pub fn validate_extend(
        &self,
        selector: &Selector,
        diags: &mut Diagnostics,
    ) -> bool {
        match selector {
            Selector::Class(name) => {
                if self.has_class(name) {
                    true
                } else {
                    diags.push(
                        Diagnostic::error(
                            "EXT001",
                            format!(
                                "The target selector '{}' was never present in the document."
                                    , name
                            ),
                        )
                        .with_note(format!("@extend .{name} requires a matching .{name} selector")),
                    );
                    false
                }
            }
            Selector::Id(name) => {
                if self.has_id(name) {
                    true
                } else {
                    diags.push(
                        Diagnostic::error(
                            "EXT002",
                            format!(
                                "The target selector '#{}' was never present.",
                                name
                            ),
                        ),
                    );
                    false
                }
            }
            Selector::Type(name) => {
                if self.has_type(name) {
                    true
                } else {
                    diags.push(
                        Diagnostic::warn(
                            "EXT003",
                            format!(
                                "Extending type selector '{name}' — make sure it exists",
                            ),
                        ),
                    );
                    true
                }
            }
            Selector::Compound(parts) => {
                let mut all_valid = true;
                for part in parts {
                    if !self.validate_extend(part, diags) {
                        all_valid = false;
                    }
                }
                all_valid
            }
            Selector::ParentRef(inner) => self.validate_extend(inner, diags),
            _ => true, // Other selectors are considered valid
        }
    }

    /// Validate all @extend declarations in a stylesheet.
    pub fn validate_stylesheet(
        &self,
        stylesheet: &Stylesheet,
        diags: &mut Diagnostics,
    ) {
        self.validate_nodes(&stylesheet.nodes, diags);
    }

    fn validate_nodes(
        &self,
        nodes: &[Node],
        diags: &mut Diagnostics,
    ) {
        for node in nodes {
            match node {
                Node::AtRule(at_rule) => {
                    use crate::AtRule;
                    match at_rule {
                        AtRule::Extend(selector) => {
                            self.validate_extend(selector, diags);
                        }
                        AtRule::Media(media) => {
                            self.validate_nodes(&media.body, diags);
                        }
                        AtRule::Supports(supports) => {
                            self.validate_nodes(&supports.body, diags);
                        }
                        AtRule::If(if_stmt) => {
                            self.validate_nodes(&if_stmt.body, diags);
                            if let Some(else_body) = &if_stmt.else_body {
                                self.validate_nodes(else_body, diags);
                            }
                        }
                        AtRule::For(for_stmt) => {
                            self.validate_nodes(&for_stmt.body, diags);
                        }
                        AtRule::Each(each_stmt) => {
                            self.validate_nodes(&each_stmt.body, diags);
                        }
                        AtRule::While(while_stmt) => {
                            self.validate_nodes(&while_stmt.body, diags);
                        }
                        _ => {}
                    }
                }
                Node::Rule(rule) => {
                    self.validate_nodes(&rule.nodes, diags);
                }
                _ => {}
            }
        }
    }

    /// Total number of registered selectors.
    pub fn len(&self) -> usize {
        self.selectors.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.selectors.is_empty()
    }
}

/// Utility: collect all extend directives from a stylesheet.
pub fn collect_extends(stylesheet: &Stylesheet) -> Vec<&Selector> {
    let mut result = Vec::new();
    collect_extends_inner(&stylesheet.nodes, &mut result);
    result
}

fn collect_extends_inner<'a>(
    nodes: &'a [Node],
    acc: &mut Vec<&'a Selector>,
) {
    for node in nodes {
        match node {
            Node::AtRule(at_rule) => {
                use crate::AtRule;
                match at_rule {
                    AtRule::Extend(selector) => acc.push(selector),
                    AtRule::Media(media) => {
                        collect_extends_inner(&media.body, acc);
                    }
                    AtRule::Supports(supports) => {
                        collect_extends_inner(&supports.body, acc);
                    }
                    AtRule::If(if_stmt) => {
                        collect_extends_inner(&if_stmt.body, acc);
                        if let Some(else_body) = &if_stmt.else_body {
                            collect_extends_inner(else_body, acc);
                        }
                    }
                    AtRule::AtRoot(root_nodes) => {
                        collect_extends_inner(root_nodes, acc);
                    }
                    _ => {}
                }
            }
            Node::Rule(rule) => {
                collect_extends_inner(&rule.nodes, acc);
            }
            _ => {}
        }
    }
}
