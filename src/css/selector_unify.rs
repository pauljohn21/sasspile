//! —— 选择器 unify 算法 ——
//!
//! 概要：实现 `selector-unify($selector1, $selector2)` 合并两个选择器。
//!
//! ## 核心概念
//! - 按位合并 compound selector
//! - 处理 type 选择器的命名空间冲突
//! - `:is()` / `:where()` / `:not()` 伪类合并
//! - 返回 `None` 表示无法统一

use super::selector_ast::{Combinator, ComplexSelector, CompoundSelector, Namespace, Selector, SimpleSelector};
use super::selector_ops::{find_id, find_pseudo_element, find_type, merge_pseudo_classes, normalize_pseudo_element, pseudo_element_eq_normalized, unify_namespace};

#[tracing::instrument(level = "debug", fields(a = %a, b = %b))]
pub fn unify(a: &Selector, b: &Selector) -> Option<Selector> {
    let results: Vec<ComplexSelector> = a
        .0
        .iter()
        .flat_map(|ca| b.0.iter().filter_map(move |cb| unify_complex(ca, cb)))
        .collect();
    (!results.is_empty()).then_some(Selector(results))
}

pub(super) fn unify_extendee_list(extendee: &Selector) -> Option<ComplexSelector> {
    match extendee.0.as_slice() {
        [] => None,
        [single] => Some(single.clone()),
        [first, rest @ ..] => {
            // 仅当所有 selector 之间有严格 super/sub 关系才能统一
            // 否则（如 .c 和 .d）无法合并为一个 extendee
            rest.iter().try_fold(first.clone(), |acc, next| {
                use super::selector_is_super::is_super_complex;
                if is_super_complex(next, &acc) {
                    // acc 更通用，下一代继续进行 super 检查
                    Some(acc)
                } else if is_super_complex(&acc, next) {
                    // next 更通用，下一代以 next 为基准
                    Some(next.clone())
                } else {
                    // 无法统一 → extendee 不兼容
                    None
                }
            })
        }
    }
}

#[tracing::instrument(level = "trace", fields(a = %a, b = %b))]
pub fn unify_complex(a: &ComplexSelector, b: &ComplexSelector) -> Option<ComplexSelector> {
    use super::selector_is_super::is_super_complex;
    if is_super_complex(a, b) { return Some(b.clone()); }
    if is_super_complex(b, a) { return Some(a.clone()); }

    let a_len = a.compounds.len();
    let b_len = b.compounds.len();
    let min_len = a_len.min(b_len);

    let mut unified_from_right: Vec<(Option<Combinator>, CompoundSelector)> = (0..min_len)
        .try_fold(Vec::with_capacity(min_len), |mut acc, i| {
            let a_idx = a_len - 1 - i;
            let b_idx = b_len - 1 - i;
            let (a_comb, a_comp) = &a.compounds[a_idx];
            let (_, b_comp) = &b.compounds[b_idx];
            let merged = unify_compound(a_comp, b_comp)?;
            acc.push((*a_comb, merged));
            Some(acc)
        })?;

    let tail = match a_len.cmp(&b_len) {
        std::cmp::Ordering::Greater => &a.compounds[..a_len - min_len],
        std::cmp::Ordering::Less => &b.compounds[..b_len - min_len],
        std::cmp::Ordering::Equal => &[],
    };

    unified_from_right.reverse();
    let compounds: Vec<(Option<Combinator>, CompoundSelector)> = tail.iter().cloned().chain(unified_from_right).collect();
    Some(ComplexSelector { compounds })
}

#[tracing::instrument(level = "trace", fields(a = %a, b = %b))]
pub fn unify_compound(a: &CompoundSelector, b: &CompoundSelector) -> Option<CompoundSelector> {
    let a_type = find_type(a);
    let b_type = find_type(b);
    let unified_type = match (a_type, b_type) {
        (Some(SimpleSelector::Type { namespace: ns_a, name: name_a }),
         Some(SimpleSelector::Type { namespace: ns_b, name: name_b })) => {
            let unified_ns = unify_namespace(ns_a, ns_b)?;
            let unified_name = match (name_a.as_str(), name_b.as_str()) {
                ("*", other) | (other, "*") => other.to_string(),
                (na, nb) if na == nb => na.to_string(),
                _ => return None,
            };
            Some(SimpleSelector::Type { namespace: unified_ns, name: unified_name })
        }
        (Some(SimpleSelector::Type { .. }), None) | (None, Some(SimpleSelector::Type { .. })) => {
            if let Some(t) = a_type { Some(t.clone()) } else { b_type.cloned() }
        }
        (None, None) => None,
        _ => None,
    };

    let a_id = find_id(a);
    let b_id = find_id(b);
    match (a_id, b_id) {
        (Some(SimpleSelector::Id(i1)), Some(SimpleSelector::Id(i2))) if i1 != i2 => return None,
        _ => {}
    }
    let unified_id = b_id.or(a_id).cloned();

    let a_has_universal = a.0.contains(&SimpleSelector::Universal);
    let b_has_universal = b.0.contains(&SimpleSelector::Universal);
    if a_has_universal || b_has_universal {
        let explicit_ns_type_only = match (&a_type, &b_type) {
            (Some(SimpleSelector::Type { namespace, .. }), None)
            | (None, Some(SimpleSelector::Type { namespace, .. })) => {
                matches!(namespace, Namespace::Explicit(_))
            }
            _ => false,
        };
        if explicit_ns_type_only && unified_type.is_none() { return None; }
    }
    let has_universal = unified_type.is_none() && unified_id.is_none()
        && (a_has_universal || b_has_universal)
        && !match (&a_type, &b_type) {
            (Some(SimpleSelector::Type { namespace, .. }), None)
            | (None, Some(SimpleSelector::Type { namespace, .. })) => {
                matches!(namespace, Namespace::Explicit(_))
            }
            _ => false,
        };

    let a_pe = find_pseudo_element(a);
    let b_pe = find_pseudo_element(b);
    let unified_pe = match (a_pe, b_pe) {
        (Some(pa), Some(pb)) => {
            if pseudo_element_eq_normalized(pa, pb) {
                Some(normalize_pseudo_element(pa))
            } else {
                return None;
            }
        }
        (Some(p), None) | (None, Some(p)) => Some(p.clone()),
        (None, None) => None,
    };

    let unified_pseudo_classes = merge_pseudo_classes(a, b);

    let rest: Vec<SimpleSelector> = a.0.iter().chain(b.0.iter())
        .filter(|s| !matches!(s, SimpleSelector::Type { .. } | SimpleSelector::Universal | SimpleSelector::Id(_) | SimpleSelector::PseudoClass { .. } | SimpleSelector::PseudoElement { .. }))
        .cloned()
        .fold(Vec::new(), |mut acc, s| { if !acc.contains(&s) { acc.push(s); } acc });

    let merged: Vec<SimpleSelector> = unified_type.into_iter()
        .chain(has_universal.then_some(SimpleSelector::Universal))
        .chain(unified_id.into_iter())
        .chain(unified_pseudo_classes.into_iter())
        .chain(unified_pe.into_iter())
        .chain(rest.into_iter())
        .collect();

    (!merged.is_empty()).then_some(CompoundSelector(merged))
}
