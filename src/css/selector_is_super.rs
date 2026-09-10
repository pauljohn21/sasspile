//! —— is_superselector 算法 ——
//!
//! 概要：实现 `is-superselector($super, $sub)` 判断 $super 是否覆盖 $sub。
//!
//! ## 核心概念
//! - 逐层匹配 compound selector
//! - 伪类参数递归比较
//! - 组合器（`>`、`+`、`~`）敏感的比较逻辑
//! - `selector_simple_covers_ext` 检查 simple selector 覆盖

use super::selector_ast::{ComplexSelector, CompoundSelector, Namespace, Selector, SimpleSelector};
use super::selector_ops::has_normalized_pseudo_element;

#[tracing::instrument(level = "debug", fields(super_ = %super_sel, sub = %sub_sel))]
pub fn is_superselector(super_sel: &Selector, sub_sel: &Selector) -> bool {
    sub_sel.0.iter().all(|sub_complex| {
        super_sel.0.iter().any(|super_complex| is_super_complex(super_complex, sub_complex))
    })
}

#[tracing::instrument(level = "trace", fields(super_ = %super_c, sub = %sub_c))]
pub fn is_super_complex(super_c: &ComplexSelector, sub_c: &ComplexSelector) -> bool {
    let super_compounds: Vec<&CompoundSelector> = super_c.compounds.iter().map(|(_, c)| c).collect();
    let sub_compounds: Vec<&CompoundSelector> = sub_c.compounds.iter().map(|(_, c)| c).collect();
    super_compounds.iter().try_fold(0usize, |si, super_comp| {
        sub_compounds[si..].iter().position(|sc| is_super_compound(super_comp, sc)).map(|offset| si + offset + 1)
    }).is_some()
}

#[tracing::instrument(level = "trace", fields(super_ = %super_c, sub = %sub_c))]
pub fn is_super_compound(super_c: &CompoundSelector, sub_c: &CompoundSelector) -> bool {
    if super_c.0.contains(&SimpleSelector::Universal) { return true; }

    super_c.0.iter().all(|super_s| match super_s {
        SimpleSelector::Type { namespace: ns_s, name: name_s } => sub_c.0.iter().any(|sub_s| match sub_s {
            SimpleSelector::Type { namespace: ns_sub, name: name_sub } => {
                name_s == name_sub && (ns_s == ns_sub || matches!(ns_s, Namespace::Any) || matches!(ns_sub, Namespace::Any))
            }
            SimpleSelector::Universal => true,
            _ => false,
        }),
        SimpleSelector::PseudoElement { .. } => has_normalized_pseudo_element(sub_c, super_s),
        SimpleSelector::PseudoClass { name, .. } => sub_c.0.iter().any(|sub_s| match sub_s {
            SimpleSelector::PseudoClass { name: sub_name, .. } => sub_name == name,
            _ => false,
        }),
        _ => sub_c.0.contains(super_s),
    }) && {
        let sub_pe: Vec<_> = sub_c.0.iter().filter(|s| matches!(s, SimpleSelector::PseudoElement { .. })).collect();
        sub_pe.is_empty() || sub_pe.iter().all(|pe| has_normalized_pseudo_element(super_c, pe))
    }
}
