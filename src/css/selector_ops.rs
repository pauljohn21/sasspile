//! —— 选择器代数运算 ——
//!
//! 概要：选择器操作的公共辅助函数（化合物冲突、组合器检测等）+ 公开 API 重导出。
//!
//! ## 核心概念
//! - `compounds_conflict`：检查 compound selector 是否冲突
//! - `extender_has_leading_combinator` / `has_leading_combinator`：前导组合器检测
//! - `is_semantic_subset`：语义子集判断
//! - `selector_simple_covers_ext`：selector 覆盖检测
//! - 重导出 `extend_selector`、`replace_selector`、`unify_selector`

pub use super::selector_unify::unify;
pub use super::selector_is_super::is_superselector;
pub use super::selector_extend::{extend_selector, extend_selector_with_mode, replace_selector};
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

/// 检查 `sub_simplex` 是否在 `super_set` 中被语义覆盖。
///
/// 对 Type 选择器进行命名空间感知比较：
/// - super_set 的 Type{Any, name} 覆盖 sub 的 Type{Any/Empty/None/Explicit, name}（如果名同名）
/// - super_set 的 Type{Empty, name} 只覆盖 sub 的 Type{Empty, name}
/// - super_set 的 Type{None, name} 只覆盖 sub 的 Type{None, name}
/// - super_set 的 Type{Explicit, name} 只覆盖 sub 的 Type{Explicit(same), name}
pub fn simple_contained_in(super_set: &CompoundSelector, sub_simplex: &SimpleSelector) -> bool {
    super_set.0.iter().any(|s| match (s, sub_simplex) {
        (SimpleSelector::Universal, _) => true,
        (SimpleSelector::Type { namespace: ns_s, name: name_s },
         SimpleSelector::Type { namespace: ns_sub, name: name_sub }) => {
            name_s == name_sub && (
                ns_s == ns_sub || matches!(ns_s, Namespace::Any)
            )
        }
        _ => s == sub_simplex,
    })
}

/// 检查 `selector_simple` 是否"覆盖" `extender_simple`（selector 更通用或相等）。
///
/// 用于 extend 的"remaining"计算：如果 selector_simple 覆盖 extender_simple，
/// 则 selector_simple 被消耗，不进入 remaining。
pub fn selector_simple_covers_ext(selector_simple: &SimpleSelector, ext_simple: &SimpleSelector) -> bool {
    match (selector_simple, ext_simple) {
        (SimpleSelector::Universal, _) => true,
        (SimpleSelector::Type { namespace: ns_s, name: name_s },
         SimpleSelector::Type { namespace: ns_e, name: name_e }) => {
            name_s == name_e && (
                ns_s == ns_e || matches!(ns_e, Namespace::Any) || matches!(ns_s, Namespace::Any)
            )
        }
        _ => selector_simple == ext_simple,
    }
}

/// 语义级别的 compound 子集检查——ext_compound 中的所有 simple 必须在 sel_compound 中有覆盖。
pub fn is_semantic_subset(ext_compound: &CompoundSelector, sel_compound: &CompoundSelector) -> bool {
    ext_compound.0.iter().all(|ext_s| simple_contained_in(sel_compound, ext_s))
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
    let rem_has_universal = remaining.iter().any(|s| {
        matches!(s, SimpleSelector::Universal) || matches!(s, SimpleSelector::Type { name, .. } if name == "*")
    });
    let ext_has_universal = ext_compound.0.iter().any(|s| {
        matches!(s, SimpleSelector::Universal) || matches!(s, SimpleSelector::Type { name, .. } if name == "*")
    });

    // 检查 Universal 命名空间冲突
    let universal_conflict = match (rem_has_universal, ext_has_universal) {
        (true, true) => {
            // 两者都有 Universal，检查命名空间是否兼容
            let get_universal_ns = |s: &SimpleSelector| -> Option<Namespace> {
                match s {
                    SimpleSelector::Universal => Some(Namespace::None),
                    SimpleSelector::Type { namespace, name } if name == "*" => Some(namespace.clone()),
                    _ => None,
                }
            };
            let rem_universal_ns = remaining.iter().find_map(get_universal_ns);
            let ext_universal_ns = ext_compound.0.iter().find_map(get_universal_ns);
            match (rem_universal_ns, ext_universal_ns) {
                (Some(ns_r), Some(ns_e)) => !namespaces_compatible(&ns_r, &ns_e),
                _ => false,
            }
        }
        _ => false,
    };

    let type_conflict = match (rem_type, ext_type) {
        (Some(SimpleSelector::Type { namespace: ns_r, name: name_r }),
         Some(SimpleSelector::Type { namespace: ns_e, name: name_e })) => {
            // Conflict if type names differ OR namespaces are incompatible
            (name_r != name_e) || !namespaces_compatible(ns_r, ns_e)
        }
        // remaining 有 Universal/Type{*, *} 且 extender 有 Type（特定命名空间）→ 冲突
        (None, Some(SimpleSelector::Type { name, .. })) if rem_has_universal && name != "*" => true,
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
    universal_conflict || type_conflict || id_conflict || pe_conflict
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
