//! 选择器代数运算——公共辅助函数 + 公开 API 重导出。

pub use super::selector_unify::unify;
pub use super::selector_is_super::is_superselector;
pub use super::selector_extend::{extend_selector, replace_selector};
pub use super::selector_unify::unify_compound;
pub use super::selector_unify::unify_complex;

use super::selector_ast::{ComplexSelector, CompoundSelector, Namespace, Selector, SimpleSelector};

pub fn find_type(compound: &CompoundSelector) -> Option<&SimpleSelector> {
    compound.0.iter().find(|s| matches!(s, SimpleSelector::Type { .. }))
}

pub fn find_id(compound: &CompoundSelector) -> Option<&SimpleSelector> {
    compound.0.iter().find(|s| matches!(s, SimpleSelector::Id(_)))
}

pub fn find_pseudo_element(compound: &CompoundSelector) -> Option<&SimpleSelector> {
    compound.0.iter().find(|s| matches!(s, SimpleSelector::PseudoElement { .. }))
}

pub fn namespaces_compatible(a: &Namespace, b: &Namespace) -> bool {
    match (a, b) {
        (Namespace::None, Namespace::None) => true,
        (Namespace::Empty, Namespace::Empty) => true,
        (Namespace::Any, _) | (_, Namespace::Any) => true,
        (Namespace::Explicit(na), Namespace::Explicit(nb)) => na == nb,
        _ => false,
    }
}

pub fn unify_namespace(a: &Namespace, b: &Namespace) -> Option<Namespace> {
    match (a, b) {
        (Namespace::None, Namespace::None) => Some(Namespace::None),
        (Namespace::Empty, Namespace::Empty) => Some(Namespace::Empty),
        (Namespace::Any, other) | (other, Namespace::Any) => Some(other.clone()),
        (Namespace::Explicit(na), Namespace::Explicit(nb)) => {
            if na == nb { Some(Namespace::Explicit(na.clone())) } else { None }
        }
        (Namespace::None, Namespace::Empty) | (Namespace::Empty, Namespace::None) => None,
        (Namespace::None, Namespace::Explicit(_)) | (Namespace::Explicit(_), Namespace::None) => None,
        (Namespace::Empty, Namespace::Explicit(_)) | (Namespace::Explicit(_), Namespace::Empty) => None,
    }
}

pub fn pseudo_element_eq_normalized(a: &SimpleSelector, b: &SimpleSelector) -> bool {
    match (a, b) {
        (SimpleSelector::PseudoElement { name: na, arg: aa, .. }, SimpleSelector::PseudoElement { name: nb, arg: ab, .. }) => na == nb && aa == ab,
        _ => false,
    }
}

pub fn has_normalized_pseudo_element(compound: &CompoundSelector, target: &SimpleSelector) -> bool {
    compound.0.iter().any(|s| pseudo_element_eq_normalized(s, target))
}

pub fn normalize_pseudo_element(pe: &SimpleSelector) -> SimpleSelector {
    match pe {
        SimpleSelector::PseudoElement { name, arg, .. } => SimpleSelector::PseudoElement {
            name: name.clone(),
            arg: arg.clone(),
            is_class_syntax: true,
        },
        other => other.clone(),
    }
}

pub fn merge_pseudo_classes(a: &CompoundSelector, b: &CompoundSelector) -> Vec<SimpleSelector> {
    use std::collections::HashMap;
    let mut result: Vec<SimpleSelector> = Vec::new();
    let mut name_index: HashMap<String, usize> = HashMap::new();
    for s in &a.0 {
        if let SimpleSelector::PseudoClass { name, arg } = s {
            name_index.insert(name.clone(), result.len());
            result.push(SimpleSelector::PseudoClass { name: name.clone(), arg: arg.clone() });
        }
    }
    for s in &b.0 {
        if let SimpleSelector::PseudoClass { name, arg } = s {
            if let Some(&idx) = name_index.get(name) {
                result[idx] = SimpleSelector::PseudoClass { name: name.clone(), arg: arg.clone() };
            } else {
                name_index.insert(name.clone(), result.len());
                result.push(SimpleSelector::PseudoClass { name: name.clone(), arg: arg.clone() });
            }
        }
    }
    result
}

pub fn has_multiple_combinators(complex: &ComplexSelector) -> bool {
    complex.compounds.windows(2).any(|w| {
        let (comb1, comp1) = &w[0];
        let (comb2, _) = &w[1];
        comb1.is_some() && comp1.0.is_empty() && comb2.is_some()
    })
}

pub fn extender_has_multiple_combinators(extender: &Selector) -> bool {
    extender.0.iter().any(|c| has_multiple_combinators(c))
}

pub fn is_subset_compound(subset: &CompoundSelector, superset: &CompoundSelector) -> bool {
    subset.0.iter().all(|s| superset.0.contains(s))
}

pub fn compounds_conflict(remaining: &[SimpleSelector], ext_compound: &CompoundSelector) -> bool {
    let rem_type = remaining.iter().find(|s| matches!(s, SimpleSelector::Type { .. }));
    let ext_type = ext_compound.0.iter().find(|s| matches!(s, SimpleSelector::Type { .. }));
    let type_conflict = match (rem_type, ext_type) {
        (Some(SimpleSelector::Type { namespace: ns_r, name: name_r }),
         Some(SimpleSelector::Type { namespace: ns_e, name: name_e })) => {
            name_r == name_e && !namespaces_compatible(ns_r, ns_e)
        }
        _ => false,
    };

    let rem_id = remaining.iter().find_map(|s| match s { SimpleSelector::Id(i) => Some(i), _ => None });
    let ext_id = ext_compound.0.iter().find_map(|s| match s { SimpleSelector::Id(i) => Some(i), _ => None });
    let id_conflict = rem_id.zip(ext_id).is_some_and(|(r, e)| r != e);

    let rem_pe = remaining.iter().find(|s| matches!(s, SimpleSelector::PseudoElement { .. }));
    let ext_pe = ext_compound.0.iter().find(|s| matches!(s, SimpleSelector::PseudoElement { .. }));
    let pe_conflict = match (rem_pe, ext_pe) {
        (Some(r), Some(e)) => !pseudo_element_eq_normalized(r, e),
        _ => false,
    };
    type_conflict || id_conflict || pe_conflict
}

pub fn has_leading_combinator(complex: &ComplexSelector) -> bool {
    complex.compounds.first().is_some_and(|(comb, _)| comb.is_some())
}

pub fn has_trailing_combinator(complex: &ComplexSelector) -> bool {
    complex.compounds.last().is_some_and(|(comb, comp)| comb.is_some() && comp.0.is_empty())
}

pub fn extender_has_leading_combinator(extender: &Selector) -> bool {
    extender.0.iter().any(|c| has_leading_combinator(c))
}

pub fn extender_has_trailing_combinator(extender: &Selector) -> bool {
    extender.0.iter().any(|c| has_trailing_combinator(c))

}
