//! Selector engine — parsing, matching, and extension of CSS selectors.
//!
//! Organized into:
//! - `mod.rs` — types and traits
//! - `parse.rs` — selector parsing from string
//! - `extend.rs` — @extend implementation

pub mod parse;
pub mod extend;

use std::fmt;

/// A simple selector (single component).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleSelector {
    /// Type selector: `div`, `span`, `*`
    Type(String),
    /// Universal selector: `*`
    Universal,
    /// Class selector: `.foo`
    Class(String),
    /// ID selector: `#bar`
    Id(String),
    /// Attribute selector: `[type="text"]`
    Attribute(String),
    /// Pseudo-class: `:hover`, `:nth-child(2n+1)`
    PseudoClass(String, Option<String>),
    /// Pseudo-element: `::before`
    PseudoElement(String, Option<String>),
    /// Placeholder: `%base`
    Placeholder(String),
}

impl fmt::Display for SimpleSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimpleSelector::Type(s) => write!(f, "{}", s),
            SimpleSelector::Universal => write!(f, "*"),
            SimpleSelector::Class(s) => write!(f, ".{}", s),
            SimpleSelector::Id(s) => write!(f, "#{}", s),
            SimpleSelector::Attribute(s) => write!(f, "[{}]", s),
            SimpleSelector::PseudoClass(name, None) => write!(f, ":{}", name),
            SimpleSelector::PseudoClass(name, Some(arg)) => write!(f, ":{}({})", name, arg),
            SimpleSelector::PseudoElement(name, None) => write!(f, "::{}", name),
            SimpleSelector::PseudoElement(name, Some(arg)) => write!(f, "::{}({})", name, arg),
            SimpleSelector::Placeholder(s) => write!(f, "%{}", s),
        }
    }
}

/// A compound selector (multiple simple selectors with no combinators).
/// e.g. `div.foo.bar:first-child`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundSelector {
    /// The simple selectors in this compound.
    /// The first element is typically the type selector (if any).
    pub components: Vec<SimpleSelector>,
}

impl CompoundSelector {
    /// Create an empty compound selector.
    pub fn new() -> Self {
        Self { components: Vec::new() }
    }

    /// Add a simple selector to this compound.
    pub fn add(&mut self, sel: SimpleSelector) {
        self.components.push(sel);
    }

    /// Check if this compound contains a placeholder.
    pub fn has_placeholder(&self) -> bool {
        self.components.iter().any(|s| matches!(s, SimpleSelector::Placeholder(_)))
    }

    /// Remove placeholder selectors from this compound.
    pub fn remove_placeholders(&mut self) {
        self.components.retain(|s| !matches!(s, SimpleSelector::Placeholder(_)));
    }
}

impl Default for CompoundSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CompoundSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.components {
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}

/// A combinator between compound selectors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Combinator {
    /// Descendant (space): `A B`
    Descendant,
    /// Child: `A > B`
    Child,
    /// Adjacent sibling: `A + B`
    AdjacentSibling,
    /// General sibling: `A ~ B`
    GeneralSibling,
}

impl fmt::Display for Combinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Combinator::Descendant => write!(f, " "),
            Combinator::Child => write!(f, " > "),
            Combinator::AdjacentSibling => write!(f, " + "),
            Combinator::GeneralSibling => write!(f, " ~ "),
        }
    }
}

/// A complex selector: compound selectors joined by combinators.
/// e.g. `div.foo > .bar + p`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexSelector {
    /// Leading compound, then (combinator, compound) pairs.
    /// e.g. `A > B + C` is stored as:
    ///   leading = A
    ///   rest = [(Child, B), (AdjacentSibling, C)]
    pub leading: CompoundSelector,
    pub rest: Vec<(Combinator, CompoundSelector)>,
}

impl ComplexSelector {
    pub fn new(leading: CompoundSelector) -> Self {
        Self { leading, rest: Vec::new() }
    }

    /// Add a compound selector with a combinator.
    pub fn add(&mut self, comb: Combinator, compound: CompoundSelector) {
        self.rest.push((comb, compound));
    }

    /// Check if this complex selector contains a placeholder.
    pub fn has_placeholder(&self) -> bool {
        if self.leading.has_placeholder() {
            return true;
        }
        self.rest.iter().any(|(_, c)| c.has_placeholder())
    }

    /// Remove all placeholder selectors.
    pub fn remove_placeholders(&mut self) {
        self.leading.remove_placeholders();
        for (_, c) in &mut self.rest {
            c.remove_placeholders();
        }
    }
}

impl fmt::Display for ComplexSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.leading)?;
        for (comb, compound) in &self.rest {
            write!(f, "{}{}", comb, compound)?;
        }
        Ok(())
    }
}

/// A selector list (comma-separated complex selectors).
/// e.g. `div, .foo, #bar`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorList {
    pub selectors: Vec<ComplexSelector>,
}

impl SelectorList {
    pub fn new() -> Self {
        Self { selectors: Vec::new() }
    }

    pub fn add(&mut self, sel: ComplexSelector) {
        self.selectors.push(sel);
    }

    /// Check if any selector in the list has a placeholder.
    pub fn has_placeholder(&self) -> bool {
        self.selectors.iter().any(|s| s.has_placeholder())
    }

    /// Remove all placeholder selectors from all complex selectors.
    pub fn remove_placeholders(&mut self) {
        for s in &mut self.selectors {
            s.remove_placeholders();
        }
    }

    /// Check if this selector is a superselector of another.
    /// A is a superselector of B if every element matched by B is also matched by A.
    pub fn is_superselector(&self, other: &SelectorList) -> bool {
        // Simple implementation: every selector in other must be matched
        // by some selector in self.
        for other_sel in &other.selectors {
            let matched = self.selectors.iter().any(|self_sel| {
                is_complex_superselector(self_sel, other_sel)
            });
            if !matched {
                return false;
            }
        }
        true
    }
}

impl Default for SelectorList {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SelectorList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.selectors.iter().map(|s| s.to_string()).collect();
        write!(f, "{}", parts.join(", "))
    }
}

/// Check if one complex selector is a superselector of another.
/// A is a superselector of B if A matches all elements B matches.
fn is_complex_superselector(parent: &ComplexSelector, child: &ComplexSelector) -> bool {
    // Simple implementation: check if parent's leading is a prefix of child
    // or if parent is a subset of child (as strings)
    let parent_str = parent.to_string();
    let child_str = child.to_string();

    // If parent == child, it's a superselector
    if parent_str == child_str {
        return true;
    }

    // If parent is a prefix of child (descendant)
    if child_str.starts_with(&parent_str) {
        return true;
    }

    // Check if all parent components are in child
    is_compound_subset(&parent.leading, &child.leading)
}

/// Check if all components of `a` are present in `b`.
fn is_compound_subset(a: &CompoundSelector, b: &CompoundSelector) -> bool {
    for comp in &a.components {
        if !b.components.contains(comp) {
            return false;
        }
    }
    true
}

/// Unify two compound selectors — produce a compound that matches elements
/// matching both.
pub fn unify(a: &CompoundSelector, b: &CompoundSelector) -> Option<CompoundSelector> {
    let mut result = CompoundSelector::new();

    // Type selector: only one allowed, prefer a if both have one
    let a_type = a.components.iter().find(|s| matches!(s, SimpleSelector::Type(_) | SimpleSelector::Universal));
    let b_type = b.components.iter().find(|s| matches!(s, SimpleSelector::Type(_) | SimpleSelector::Universal));

    match (a_type, b_type) {
        (Some(a_t), Some(b_t)) => {
            // If both are type selectors, they must match (or one is universal)
            match (a_t, b_t) {
                (SimpleSelector::Universal, _) => result.add(b_t.clone()),
                (_, SimpleSelector::Universal) => result.add(a_t.clone()),
                _ if a_t == b_t => result.add(a_t.clone()),
                _ => return None, // incompatible types
            }
        }
        (Some(a_t), None) => result.add(a_t.clone()),
        (None, Some(b_t)) => result.add(b_t.clone()),
        (None, None) => {}
    }

    // Add all non-type components from both
    for comp in a.components.iter()
        .filter(|s| !matches!(s, SimpleSelector::Type(_) | SimpleSelector::Universal))
    {
        if !result.components.contains(comp) {
            result.add(comp.clone());
        }
    }
    for comp in b.components.iter()
        .filter(|s| !matches!(s, SimpleSelector::Type(_) | SimpleSelector::Universal))
    {
        if !result.components.contains(comp) {
            result.add(comp.clone());
        }
    }

    Some(result)
}
