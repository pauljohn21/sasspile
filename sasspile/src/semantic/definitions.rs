//! Definition registry — tracks all declared mixins and functions.
//!
//! Provides duplicate detection and name resolution for @mixin
//! and @function definitions.

use std::collections::HashMap;

use crate::diagnostics::{Diagnostic, Diagnostics};
use crate::{
    AtRule, FunctionDef, MixinDef, Node, Stylesheet,
};

/// A registered function definition.
#[derive(Debug, Clone)]
pub struct FunctionEntry {
    /// Function name.
    pub name: String,
    /// Number of required parameters.
    pub required_params: usize,
    /// Total parameters (including optional).
    pub total_params: usize,
    /// Whether it accepts variable arguments (...).
    pub variadic: bool,
    /// The definition AST node.
    pub definition: FunctionDef,
}

/// A registered mixin definition.
#[derive(Debug, Clone)]
pub struct MixinEntry {
    /// Mixin name.
    pub name: String,
    /// Number of required parameters.
    pub required_params: usize,
    /// Total parameters (including optional).
    pub total_params: usize,
    /// Whether it accepts variable arguments (...).
    pub variadic: bool,
    /// The definition AST node.
    pub definition: MixinDef,
}

/// Registry of all function and mixin definitions.
#[derive(Debug, Clone, Default)]
pub struct DefinitionRegistry {
    /// Function definitions by name.
    functions: HashMap<String, FunctionEntry>,
    /// Mixin definitions by name.
    mixins: HashMap<String, MixinEntry>,
}

/// Duplicate definition information.
#[derive(Debug, Clone)]
pub struct DuplicateInfo {
    /// Name that was duplicated.
    pub name: String,
    /// Which definition kind was duplicated.
    pub kind: DefinitionKind,
}

/// Definition kind for error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionKind {
    Mixin,
    Function,
}

impl std::fmt::Display for DefinitionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefinitionKind::Mixin => write!(f, "mixin"),
            DefinitionKind::Function => write!(f, "function"),
        }
    }
}

impl DefinitionRegistry {
    /// Create an empty definition registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a function definition. Returns error diagnostic if duplicate.
    pub fn register_function(
        &mut self,
        def: &FunctionDef,
        diags: &mut Diagnostics,
    ) -> Result<(), DuplicateInfo> {
        let name = def.name.clone();
        let has_variadic = def
            .params
            .last()
            .map(|p| p.name.ends_with("..."))
            .unwrap_or(false);

        let total = def.params.len();
        let required = if has_variadic {
            total.saturating_sub(1)
        } else {
            def.params
                .iter()
                .filter(|p| p.default.is_none())
                .count()
        };

        if let Some(existing) = self.functions.get(&name) {
            // Sass allows function redefinition (last wins) but warns.
            diags.push(
                Diagnostic::warn(
                    "DEF001",
                    format!(
                        "Function '{name}' is being redefined.",
                    ),
                )
                .with_note(format!(
                    "Previous definition had {} params, new has {} params.",
                    existing.total_params, total,
                )),
            );
        }

        self.functions.insert(
            name.clone(),
            FunctionEntry {
                name: name.clone(),
                required_params: required,
                total_params: total,
                variadic: has_variadic,
                definition: def.clone(),
            },
        );
        Ok(())
    }

    /// Register a mixin definition. Returns diagnostic if duplicate.
    pub fn register_mixin(
        &mut self,
        def: &MixinDef,
        diags: &mut Diagnostics,
    ) -> Result<(), DuplicateInfo> {
        let name = def.name.clone();
        let has_variadic = def
            .params
            .last()
            .map(|p| p.name.ends_with("..."))
            .unwrap_or(false);

        let total = def.params.len();
        let required = if has_variadic {
            total.saturating_sub(1)
        } else {
            def.params
                .iter()
                .filter(|p| p.default.is_none())
                .count()
        };

        if self.mixins.contains_key(&name) {
            diags.push(
                Diagnostic::warn(
                    "DEF002",
                    format!("Mixin '{name}' is being redefined."),
                ),
            );
        }

        self.mixins.insert(
            name.clone(),
            MixinEntry {
                name: name.clone(),
                required_params: required,
                total_params: total,
                variadic: has_variadic,
                definition: def.clone(),
            },
        );
        Ok(())
    }

    /// Look up a function by name.
    pub fn get_function(&self, name: &str) -> Option<&FunctionEntry> {
        self.functions.get(name)
    }

    /// Look up a mixin by name.
    pub fn get_mixin(&self, name: &str) -> Option<&MixinEntry> {
        self.mixins.get(name)
    }

    /// Check if a function is defined.
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Check if a mixin is defined.
    pub fn has_mixin(&self, name: &str) -> bool {
        self.mixins.contains_key(name)
    }

    /// Check if any mixin or function with this name exists.
    pub fn has_definition(&self, name: &str) -> bool {
        self.has_function(name) || self.has_mixin(name)
    }

    /// All registered function names.
    pub fn function_names(&self) -> impl Iterator<Item = &str> {
        self.functions.keys().map(|s| s.as_str())
    }

    /// All registered mixin names.
    pub fn mixin_names(&self) -> impl Iterator<Item = &str> {
        self.mixins.keys().map(|s| s.as_str())
    }

    /// Validate that a called function exists with arity check.
    pub fn validate_function_call(
        &self,
        name: &str,
        arg_count: usize,
        diags: &mut Diagnostics,
    ) -> bool {
        match self.get_function(name) {
            Some(func) => {
                if arg_count < func.required_params {
                    diags.push(
                        Diagnostic::error(
                            "DEF010",
                            format!(
                                "Function '{name}' requires at least {} arguments, got {}.",
                                func.required_params,
                                arg_count,
                            ),
                        ),
                    );
                    false
                } else if !func.variadic && arg_count > func.total_params {
                    diags.push(
                        Diagnostic::error(
                            "DEF011",
                            format!(
                                "Function '{name}' takes at most {} arguments, got {}.",
                                func.total_params,
                                arg_count,
                            ),
                        ),
                    );
                    false
                } else {
                    true
                }
            }
            None => {
                diags.push(
                    Diagnostic::error(
                        "DEF012",
                        format!("Undefined function '{name}'."),
                    ),
                );
                false
            }
        }
    }

    /// Validate that a called mixin exists with arity check.
    pub fn validate_mixin_call(
        &self,
        name: &str,
        arg_count: usize,
        diags: &mut Diagnostics,
    ) -> bool {
        match self.get_mixin(name) {
            Some(mixin) => {
                if arg_count < mixin.required_params {
                    diags.push(
                        Diagnostic::error(
                            "DEF020",
                            format!(
                                "Mixin '{name}' requires at least {} arguments, got {}.",
                                mixin.required_params,
                                arg_count,
                            ),
                        ),
                    );
                    false
                } else if !mixin.variadic && arg_count > mixin.total_params {
                    diags.push(
                        Diagnostic::error(
                            "DEF021",
                            format!(
                                "Mixin '{name}' takes at most {} arguments, got {}.",
                                mixin.total_params,
                                arg_count,
                            ),
                        ),
                    );
                    false
                } else {
                    true
                }
            }
            None => {
                diags.push(
                    Diagnostic::error(
                        "DEF022",
                        format!("Undefined mixin '{name}'."),
                    ),
                );
                false
            }
        }
    }

    /// Collect all definitions from a stylesheet's top-level nodes.
    pub fn collect_from_stylesheet(
        &mut self,
        stylesheet: &Stylesheet,
        diags: &mut Diagnostics,
    ) {
        self.collect_from_nodes(&stylesheet.nodes, diags);
    }

    fn collect_from_nodes(
        &mut self,
        nodes: &[Node],
        diags: &mut Diagnostics,
    ) {
        for node in nodes {
            match node {
                Node::AtRule(AtRule::Function(def)) => {
                    let _ = self.register_function(def, diags);
                }
                Node::AtRule(AtRule::Mixin(def)) => {
                    let _ = self.register_mixin(def, diags);
                }
                Node::AtRule(AtRule::Media(media)) => {
                    self.collect_from_nodes(&media.body, diags);
                }
                Node::AtRule(AtRule::Supports(supports)) => {
                    self.collect_from_nodes(&supports.body, diags);
                }
                Node::Rule(rule) => {
                    self.collect_from_nodes(&rule.nodes, diags);
                }
                _ => {}
            }
        }
    }

    /// Validate all @include and @function calls in stylesheet.
    pub fn validate_calls(
        &self,
        stylesheet: &Stylesheet,
        diags: &mut Diagnostics,
    ) {
        self.validate_calls_in_nodes(&stylesheet.nodes, diags);
    }

    fn validate_calls_in_nodes(
        &self,
        nodes: &[Node],
        diags: &mut Diagnostics,
    ) {
        for node in nodes {
            match node {
                Node::AtRule(AtRule::Include(include)) => {
                    self.validate_mixin_call(
                        &include.name,
                        include.args.len(),
                        diags,
                    );
                }
                Node::AtRule(AtRule::Media(media)) => {
                    self.validate_calls_in_nodes(&media.body, diags);
                }
                Node::AtRule(AtRule::Supports(supports)) => {
                    self.validate_calls_in_nodes(&supports.body, diags);
                }
                Node::AtRule(AtRule::If(stmt)) => {
                    self.validate_calls_in_nodes(&stmt.body, diags);
                    if let Some(else_body) = &stmt.else_body {
                        self.validate_calls_in_nodes(else_body, diags);
                    }
                }
                Node::AtRule(AtRule::For(stmt)) => {
                    self.validate_calls_in_nodes(&stmt.body, diags);
                }
                Node::AtRule(AtRule::Each(stmt)) => {
                    self.validate_calls_in_nodes(&stmt.body, diags);
                }
                Node::AtRule(AtRule::While(stmt)) => {
                    self.validate_calls_in_nodes(&stmt.body, diags);
                }
                Node::Rule(rule) => {
                    self.validate_calls_in_nodes(&rule.nodes, diags);
                }
                _ => {}
            }
        }
    }

    /// Number of registered functions.
    pub fn function_count(&self) -> usize {
        self.functions.len()
    }

    /// Number of registered mixins.
    pub fn mixin_count(&self) -> usize {
        self.mixins.len()
    }

    /// Total number of definitions.
    pub fn len(&self) -> usize {
        self.functions.len() + self.mixins.len()
    }

    /// Whether empty.
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty() && self.mixins.is_empty()
    }
}
