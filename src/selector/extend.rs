//! @extend implementation — applies extension requests to selector lists.

use super::*;

/// An extension request: `extender` extends `extendee`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendRequest {
    pub extender: String,
    pub extendee: String,
    pub optional: bool,
}

/// ExtendTable — collects all @extend requests during evaluation.
#[derive(Debug, Clone, Default)]
pub struct ExtendTable {
    pub entries: Vec<ExtendRequest>,
}

impl ExtendTable {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(&mut self, extender: String, extendee: String, optional: bool) {
        self.entries.push(ExtendRequest { extender, extendee, optional });
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Apply extensions to a selector string.
///
/// Given a selector string and the extend table, produce a new selector string
/// with extenders added.
///
/// # Example
/// ```
/// use sasspile::selector::extend::{ExtendTable, apply_extends_to_selector};
///
/// let mut table = ExtendTable::new();
/// table.add(".bar".to_string(), ".foo".to_string(), false);
/// let result = apply_extends_to_selector(".foo", &table);
/// assert_eq!(result, ".foo, .bar");
/// ```
pub fn apply_extends_to_selector(selector: &str, table: &ExtendTable) -> String {
    if table.is_empty() {
        return selector.to_string();
    }

    let list = super::parse::parse_selector_list(selector);
    let new_list = apply_extends_to_list(&list, table);
    new_list.to_string()
}

/// Apply extensions to a SelectorList.
pub fn apply_extends_to_list(list: &SelectorList, table: &ExtendTable) -> SelectorList {
    let mut result = SelectorList::new();

    for complex in &list.selectors {
        // Start with the original selector
        result.add(complex.clone());

        // Check each extend request
        for entry in &table.entries {
            // Parse the extendee to check if it matches
            let extendee_list = super::parse::parse_selector_list(&entry.extendee);
            let extender_list = super::parse::parse_selector_list(&entry.extender);

            // Check if any selector in this complex matches the extendee
            for extendee_complex in &extendee_list.selectors {
                if selector_contains_complex(complex, extendee_complex) {
                    // Add each extender selector to the result
                    for extender_complex in &extender_list.selectors {
                        let merged = merge_selectors(complex, extender_complex);
                        if !result.selectors.iter().any(|s| s == &merged) {
                            result.add(merged);
                        }
                    }
                }
            }
        }
    }

    result
}

/// Check if a complex selector contains another (as a substring match).
fn selector_contains_complex(haystack: &ComplexSelector, needle: &ComplexSelector) -> bool {
    // Simple string-based check
    let haystack_str = haystack.to_string();
    let needle_str = needle.to_string();
    haystack_str.contains(&needle_str)
}

/// Merge two selectors — replace matching parts of `original` with `extender`.
fn merge_selectors(_original: &ComplexSelector, extender: &ComplexSelector) -> ComplexSelector {
    // Simple implementation: just use the extender as-is
    // (This handles the common case: .bar extends .foo → .bar is added)
    extender.clone()
}
