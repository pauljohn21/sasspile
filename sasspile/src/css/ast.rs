//! CSS AST — intermediate representation for code generation.
//!
//! Separates the evaluated AST (Sass semantics) from the final
//! CSS output, enabling multiple output formats (expanded, compressed).

use crate::value::Value;

/// A CSS rule (selector + declarations).
#[derive(Debug, Clone)]
pub struct CssRule {
    /// Flattened selector string.
    pub selector: String,
    /// Property declarations.
    pub declarations: Vec<CssDeclaration>,
    /// Nested rules (for expanded output).
    pub children: Vec<CssRule>,
}

/// A CSS property declaration.
#[derive(Debug, Clone)]
pub struct CssDeclaration {
    /// Property name.
    pub name: String,
    /// Property value.
    pub value: String,
    /// `!important` flag.
    pub important: bool,
}

/// A CSS at-rule (@media, @supports, @import, @keyframes).
#[derive(Debug, Clone)]
pub struct CssAtRule {
    /// At-rule name (e.g., "media", "supports").
    pub name: String,
    /// Query/condition text.
    pub query: String,
    /// Nested rules.
    pub children: Vec<CssRule>,
    /// Nested at-rules.
    pub nested_atrules: Vec<CssAtRule>,
}

/// Top-level CSS document.
#[derive(Debug, Clone)]
pub struct CssDocument {
    /// Top-level rules.
    pub rules: Vec<CssRule>,
    /// Top-level at-rules.
    pub atrules: Vec<CssAtRule>,
}

impl CssDocument {
    /// Create an empty CSS document.
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            atrules: Vec::new(),
        }
    }

    /// Add a top-level rule.
    pub fn add_rule(&mut self, rule: CssRule) {
        self.rules.push(rule);
    }

    /// Add a top-level at-rule.
    pub fn add_atrule(&mut self, atrule: CssAtRule) {
        self.atrules.push(atrule);
    }
}

impl Default for CssDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl CssDeclaration {
    /// Create a new declaration.
    pub fn new(name: String, value: Value, important: bool) -> Self {
        Self {
            name,
            value: value.to_css_string(),
            important,
        }
    }

    /// Create with a raw string value.
    pub fn raw(name: String, value: String, important: bool) -> Self {
        Self {
            name,
            value,
            important,
        }
    }
}
