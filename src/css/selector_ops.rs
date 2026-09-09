//! 选择器代数运算——unify（统一）、is_superselector（超选择器判断）、extend（扩展）。
//!
//! 这些算法基于 AST 结构操作，而非字符串匹配。
#![allow(clippy::nonminimal_bool, clippy::missing_panics_doc, clippy::expect_used)]

use super::selector_ast::{
    Combinator, ComplexSelector, CompoundSelector, Namespace, Selector, SimpleSelector,
};

// ─── 辅助函数 ──────────────────────────────────────────────────

/// 检查 complex selector 是否有多个连续组合器（中间、前导或尾随）。
fn has_multiple_combinators(complex: &ComplexSelector) -> bool {
    complex
        .compounds
        .windows(2)
        .any(|w| {
            let (comb1, comp1) = &w[0];
            let (comb2, _) = &w[1];
            comb1.is_some() && comp1.0.is_empty() && comb2.is_some()
        })
}

/// 检查 extender Selector 是否有多个连续组合器。
fn extender_has_multiple_combinators(extender: &Selector) -> bool {
    extender.0.iter().any(|c| has_multiple_combinators(c))
}

/// 检查 subset 是否是 superset 的 simple selector 子集。
fn is_subset_compound(subset: &CompoundSelector, superset: &CompoundSelector) -> bool {
    subset.0.iter().all(|s| superset.0.contains(s))
}

/// 从 compound 中提取 Type 选择器引用。
fn find_type(compound: &CompoundSelector) -> Option<&SimpleSelector> {
    compound.0.iter().find(|s| matches!(s, SimpleSelector::Type { .. }))
}

/// 从 compound 中提取 Id 选择器引用。
fn find_id(compound: &CompoundSelector) -> Option<&SimpleSelector> {
    compound.0.iter().find(|s| matches!(s, SimpleSelector::Id(_)))
}

/// 从 compound 中提取 PseudoElement 选择器引用。
fn find_pseudo_element(compound: &CompoundSelector) -> Option<&SimpleSelector> {
    compound
        .0
        .iter()
        .find(|s| matches!(s, SimpleSelector::PseudoElement { .. }))
}

/// 从 compound 中提取所有 PseudoClass 选择器引用。
fn find_pseudo_classes(compound: &CompoundSelector) -> Vec<&SimpleSelector> {
    compound
        .0
        .iter()
        .filter(|s| matches!(s, SimpleSelector::PseudoClass { .. }))
        .collect()
}

/// 判断两个命名空间是否兼容（可统一）。
fn namespaces_compatible(a: &Namespace, b: &Namespace) -> bool {
    match (a, b) {
        (Namespace::None, Namespace::None) => true,
        (Namespace::Empty, Namespace::Empty) => true,
        (Namespace::Any, _) | (_, Namespace::Any) => true,
        (Namespace::Explicit(na), Namespace::Explicit(nb)) => na == nb,
        _ => false,
    }
}

/// 根据命名空间规则矩阵统一两个 Type 选择器的命名空间。
/// 返回统一后的 Namespace（name 单独处理）。
fn unify_namespace(a: &Namespace, b: &Namespace) -> Option<Namespace> {
    match (a, b) {
        // None + None → None
        (Namespace::None, Namespace::None) => Some(Namespace::None),
        // Empty + Empty → Empty
        (Namespace::Empty, Namespace::Empty) => Some(Namespace::Empty),
        // 任何一方是 Any → 取另一方（drop any）
        (Namespace::Any, other) | (other, Namespace::Any) => Some(other.clone()),
        // Explicit + Explicit → 相同 ns 保留，不同 ns 冲突
        (Namespace::Explicit(na), Namespace::Explicit(nb)) => {
            if na == nb {
                Some(Namespace::Explicit(na.clone()))
            } else {
                None
            }
        }
        // None + Empty → 冲突
        (Namespace::None, Namespace::Empty) | (Namespace::Empty, Namespace::None) => None,
        // None + Explicit → 冲突
        (Namespace::None, Namespace::Explicit(_)) | (Namespace::Explicit(_), Namespace::None) => {
            None
        }
        // Empty + Explicit → 冲突
        (Namespace::Empty, Namespace::Explicit(_))
        | (Namespace::Explicit(_), Namespace::Empty) => None,
    }
}

/// 归一化伪元素比较：单双冒号语法视为等价。
/// 比较 name 和 arg，忽略 is_class_syntax 字段。
fn pseudo_element_eq_normalized(a: &SimpleSelector, b: &SimpleSelector) -> bool {
    match (a, b) {
        (
            SimpleSelector::PseudoElement {
                name: na,
                arg: aa,
                ..
            },
            SimpleSelector::PseudoElement {
                name: nb,
                arg: ab,
                ..
            },
        ) => na == nb && aa == ab,
        _ => false,
    }
}

/// 检查 compound 是否包含归一化后等价的伪元素。
fn has_normalized_pseudo_element(
    compound: &CompoundSelector,
    target: &SimpleSelector,
) -> bool {
    compound
        .0
        .iter()
        .any(|s| pseudo_element_eq_normalized(s, target))
}

// ─── unify 算法 ──────────────────────────────────────────────────

/// 统一两个选择器列表（笛卡尔积）。
#[tracing::instrument(level = "debug", fields(a = %a, b = %b))]
pub fn unify(a: &Selector, b: &Selector) -> Option<Selector> {
    let results: Vec<ComplexSelector> = a
        .0
        .iter()
        .flat_map(|ca| b.0.iter().filter_map(move |cb| unify_complex(ca, cb)))
        .collect();
    (!results.is_empty()).then_some(Selector(results))
}

/// 将 extendee 列表统一为单个 complex。
fn unify_extendee_list(extendee: &Selector) -> Option<ComplexSelector> {
    match extendee.0.as_slice() {
        [] => None,
        [single] => Some(single.clone()),
        [first, rest @ ..] => rest.iter().try_fold(first.clone(), |acc, next| {
            unify_complex(&acc, next)
        }),
    }
}

/// 统一两个复杂选择器——从右向左逐位置合并复合选择器。
#[tracing::instrument(level = "trace", fields(a = %a, b = %b))]
pub fn unify_complex(a: &ComplexSelector, b: &ComplexSelector) -> Option<ComplexSelector> {
    if is_super_complex(a, b) {
        return Some(b.clone());
    }
    if is_super_complex(b, a) {
        return Some(a.clone());
    }

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

    let compounds: Vec<(Option<Combinator>, CompoundSelector)> = tail
        .iter()
        .cloned()
        .chain(unified_from_right)
        .collect();

    Some(ComplexSelector { compounds })
}

/// 统一两个复合选择器——合并简单选择器。
///
/// 算法：
/// 1. Type 冲突检测：按命名空间规则矩阵判断
/// 2. Id 冲突检测：不同 → None
/// 3. PseudoElement 冲突检测：归一化后比较
/// 4. PseudoClass 链式合并：同 name 覆盖，不同 name 链式
/// 5. 其余：并集去重
#[tracing::instrument(level = "trace", fields(a = %a, b = %b))]
pub fn unify_compound(a: &CompoundSelector, b: &CompoundSelector) -> Option<CompoundSelector> {
    // ── 1. Type 冲突检测（命名空间感知）──
    let a_type = find_type(a);
    let b_type = find_type(b);
    let unified_type = match (a_type, b_type) {
        (
            Some(SimpleSelector::Type {
                namespace: ns_a,
                name: name_a,
            }),
            Some(SimpleSelector::Type {
                namespace: ns_b,
                name: name_b,
            }),
        ) => {
            let unified_ns = unify_namespace(ns_a, ns_b)?;
            // 确定 name：如果一方是 "*"，取另一方
            let unified_name = match (name_a.as_str(), name_b.as_str()) {
                ("*", other) | (other, "*") => other.to_string(),
                (na, nb) if na == nb => na.to_string(),
                _ => return None, // 不同 name → 冲突
            };
            Some(SimpleSelector::Type {
                namespace: unified_ns,
                name: unified_name,
            })
        }
        (Some(SimpleSelector::Type { .. }), None)
        | (None, Some(SimpleSelector::Type { .. })) => {
            // 只一方有 Type，直接保留
            if let Some(t) = a_type {
                Some(t.clone())
            } else {
                b_type.cloned()
            }
        }
        (Some(_), None) | (None, Some(_)) | (Some(_), Some(_)) => {
            // 非 Type 选择器（不应发生，find_type 已过滤）或双方非 Type
            None
        }
        (None, None) => None,
    };

    // ── 2. Id 冲突检测 ──
    let a_id = find_id(a);
    let b_id = find_id(b);
    match (a_id, b_id) {
        (Some(SimpleSelector::Id(i1)), Some(SimpleSelector::Id(i2))) if i1 != i2 => return None,
        _ => {}
    }
    let unified_id = b_id.or(a_id).cloned();

    // ── 3. Universal + Type 兼容检查 ──
    let a_has_universal = a.0.contains(&SimpleSelector::Universal);
    let b_has_universal = b.0.contains(&SimpleSelector::Universal);
    // Universal + Explicit namespace Type（单方 Type）→ 冲突
    if a_has_universal || b_has_universal {
        let explicit_ns_type_only = match (&a_type, &b_type) {
            (Some(SimpleSelector::Type { namespace, .. }), None)
            | (None, Some(SimpleSelector::Type { namespace, .. })) => {
                matches!(namespace, Namespace::Explicit(_))
            }
            _ => false,
        };
        if explicit_ns_type_only && unified_type.is_none() {
            return None;
        }
    }
    let has_universal = unified_type.is_none() && unified_id.is_none()
        && (a_has_universal || b_has_universal)
        // Universal + Explicit ns Type（单方 Type）→ 不含 Universal
        && !match (&a_type, &b_type) {
            (Some(SimpleSelector::Type { namespace, .. }), None)
            | (None, Some(SimpleSelector::Type { namespace, .. })) => {
                matches!(namespace, Namespace::Explicit(_))
            }
            _ => false,
        };

    // ── 4. PseudoElement 冲突检测（归一化）──
    let a_pe = find_pseudo_element(a);
    let b_pe = find_pseudo_element(b);
    let unified_pe = match (a_pe, b_pe) {
        (Some(pa), Some(pb)) => {
            if pseudo_element_eq_normalized(pa, pb) {
                // 归一化后相同，取 class syntax 版本
                Some(normalize_pseudo_element(pa))
            } else {
                return None;
            }
        }
        (Some(p), None) | (None, Some(p)) => Some(p.clone()),
        (None, None) => None,
    };

    // ── 5. PseudoClass 链式合并 ──
    let unified_pseudo_classes = merge_pseudo_classes(a, b);

    // ── 6. 其余：并集去重 ──
    let rest: Vec<SimpleSelector> = a
        .0
        .iter()
        .chain(b.0.iter())
        .filter(|s| {
            !matches!(
                s,
                SimpleSelector::Type { .. }
                    | SimpleSelector::Universal
                    | SimpleSelector::Id(_)
                    | SimpleSelector::PseudoClass { .. }
                    | SimpleSelector::PseudoElement { .. }
            )
        })
        .cloned()
        .fold(Vec::new(), |mut acc, s| {
            if !acc.contains(&s) {
                acc.push(s);
            }
            acc
        });

    // ── 7. 组装：Type → Universal → Id → PseudoClass → PseudoElement → rest ──
    let merged: Vec<SimpleSelector> = unified_type
        .into_iter()
        .chain(has_universal.then_some(SimpleSelector::Universal))
        .chain(unified_id.into_iter())
        .chain(unified_pseudo_classes.into_iter())
        .chain(unified_pe.into_iter())
        .chain(rest.into_iter())
        .collect();

    (!merged.is_empty()).then_some(CompoundSelector(merged))
}

/// 归一化伪元素为 class syntax（单冒号）。
fn normalize_pseudo_element(pe: &SimpleSelector) -> SimpleSelector {
    match pe {
        SimpleSelector::PseudoElement {
            name,
            arg,
            is_class_syntax: _,
        } => SimpleSelector::PseudoElement {
            name: name.clone(),
            arg: arg.clone(),
            is_class_syntax: true,
        },
        other => other.clone(),
    }
}

/// 合并伪类：同名覆盖，不同名链式。
/// 合并策略：先收集 a 中所有伪类，再用 b 中伪类覆盖同名项。
fn merge_pseudo_classes(a: &CompoundSelector, b: &CompoundSelector) -> Vec<SimpleSelector> {
    let mut result: Vec<SimpleSelector> = Vec::new();
    let mut name_index: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    // 先加入 a 中的伪类
    for s in &a.0 {
        if let SimpleSelector::PseudoClass { name, arg } = s {
            name_index.insert(name.clone(), result.len());
            result.push(SimpleSelector::PseudoClass {
                name: name.clone(),
                arg: arg.clone(),
            });
        }
    }

    // 用 b 中伪类覆盖同名项
    for s in &b.0 {
        if let SimpleSelector::PseudoClass { name, arg } = s {
            if let Some(&idx) = name_index.get(name) {
                result[idx] = SimpleSelector::PseudoClass {
                    name: name.clone(),
                    arg: arg.clone(),
                };
            } else {
                name_index.insert(name.clone(), result.len());
                result.push(SimpleSelector::PseudoClass {
                    name: name.clone(),
                    arg: arg.clone(),
                });
            }
        }
    }

    result
}

// ─── is_superselector 算法 ────────────────────────────────────────

/// 判断 `super_sel` 是否是 `sub_sel` 的超选择器。
#[tracing::instrument(level = "debug", fields(super_ = %super_sel, sub = %sub_sel))]
pub fn is_superselector(super_sel: &Selector, sub_sel: &Selector) -> bool {
    sub_sel.0.iter().all(|sub_complex| {
        super_sel
            .0
            .iter()
            .any(|super_complex| is_super_complex(super_complex, sub_complex))
    })
}

/// 判断 `super_c` 是否是 `sub_c` 的超复杂选择器。
#[tracing::instrument(level = "trace", fields(super_ = %super_c, sub = %sub_c))]
pub fn is_super_complex(super_c: &ComplexSelector, sub_c: &ComplexSelector) -> bool {
    let super_compounds: Vec<&CompoundSelector> = super_c.compounds.iter().map(|(_, c)| c).collect();
    let sub_compounds: Vec<&CompoundSelector> = sub_c.compounds.iter().map(|(_, c)| c).collect();

    super_compounds
        .iter()
        .try_fold(0usize, |si, super_comp| {
            sub_compounds[si..]
                .iter()
                .position(|sc| is_super_compound(super_comp, sc))
                .map(|offset| si + offset + 1)
        })
        .is_some()
}

/// 判断 `super_c` 是否是 `sub_c` 的超复合选择器。
///
/// spec 规则：
/// - super 中每个 Type 必须在 sub 中存在兼容的 Type 或 Universal
/// - super 中每个伪元素必须在 sub 中存在（归一化比较）
/// - super 中每个伪类必须在 sub 中存在（name 相同）
/// - super 中其他 simples 必须在 sub 中存在
/// - sub 中多出的伪元素使 super 不是超选择器
#[tracing::instrument(level = "trace", fields(super_ = %super_c, sub = %sub_c))]
pub fn is_super_compound(super_c: &CompoundSelector, sub_c: &CompoundSelector) -> bool {
    // `*` 是任何复合选择器的超选择器
    if super_c.0.contains(&SimpleSelector::Universal) {
        return true;
    }

    super_c.0.iter().all(|super_s| match super_s {
        SimpleSelector::Type {
            namespace: ns_s,
            name: name_s,
        } => sub_c.0.iter().any(|sub_s| match sub_s {
            SimpleSelector::Type {
                namespace: ns_sub,
                name: name_sub,
            } => {
                // 同名 + 命名空间兼容
                name_s == name_sub
                    && (ns_s == ns_sub
                        || matches!(ns_s, Namespace::Any)
                        || matches!(ns_sub, Namespace::Any))
            }
            SimpleSelector::Universal => true,
            _ => false,
        }),
        SimpleSelector::PseudoElement { .. } => {
            // super 中的伪元素必须在 sub 中存在（归一化比较）
            has_normalized_pseudo_element(sub_c, super_s)
        }
        SimpleSelector::PseudoClass { name, .. } => {
            // super 中的伪类必须在 sub 中存在（name 相同）
            sub_c.0.iter().any(|sub_s| match sub_s {
                SimpleSelector::PseudoClass { name: sub_name, .. } => sub_name == name,
                _ => false,
            })
        }
        _ => sub_c.0.contains(super_s),
    }) && {
        // 额外条件：sub 中的伪元素必须在 super 中存在（归一化比较）
        let sub_pe = find_pseudo_classes(sub_c); // 这里错了，应该是 PseudoElement
        let sub_pe: Vec<_> = sub_c
            .0
            .iter()
            .filter(|s| matches!(s, SimpleSelector::PseudoElement { .. }))
            .collect();
        sub_pe.is_empty()
            || sub_pe
                .iter()
                .all(|pe| has_normalized_pseudo_element(super_c, pe))
    }
}

// ─── extend/replace 算法 ──────────────────────────────────────────

/// 扩展选择器：在 `selector` 中查找匹配 `extendee` 的部分，用 `extender` 追加。
#[tracing::instrument(level = "info", fields(extendee = %extendee, extender = %extender))]
pub fn extend_selector(selector: &Selector, extendee: &Selector, extender: &Selector) -> Selector {
    let unified_extendee = unify_extendee_list(extendee);
    tracing::debug!(unified = ?unified_extendee, "extend_selector: unified extendee");

    let is_no_op = unified_extendee.is_none() || is_more_specific_than(extender, extendee);

    if is_no_op {
        tracing::debug!("extend_selector: NO-OP triggered");
        return selector.clone();
    }

    let unified_extendee = unified_extendee.expect("checked above");

    let results: Vec<ComplexSelector> = selector
        .0
        .iter()
        .flat_map(|complex| {
            let original = std::iter::once(complex.clone());
            let extended = extend_complex(complex, &unified_extendee, extender)
                .map(|s| s.0.into_iter())
                .unwrap_or_default();
            original.chain(extended)
        })
        .fold(Vec::new(), |mut acc, c| {
            if !acc.contains(&c) {
                acc.push(c);
            }
            acc
        });

    let result = Selector(results);
    tracing::debug!(%result, "extend_selector: final result");
    result
}

/// 检查 extender 是否比 extendee 更具体（用于 NO-OP 检测）。
fn is_more_specific_than(extender: &Selector, extendee: &Selector) -> bool {
    extender.0.iter().all(|ext_c| {
        extendee.0.iter().any(|ee_c| {
            ext_c.compounds.len() >= ee_c.compounds.len()
                && ee_c.compounds.iter().enumerate().all(|(i, (_, ee_comp))| {
                    ext_c.compounds.get(i).is_some_and(|(_, ext_comp)| {
                        ee_comp.0.iter().all(|s| match s {
                            // Type: extender 必须与 extendee 同名（命名空间兼容）
                            SimpleSelector::Type {
                                namespace: ns_ee,
                                name: name_ee,
                            } => ext_comp.0.iter().any(|es| match es {
                                SimpleSelector::Type {
                                    namespace: ns_ext,
                                    name: name_ext,
                                } => {
                                    name_ee == name_ext
                                        && (ns_ee == ns_ext
                                            || matches!(ns_ee, Namespace::Any)
                                            || matches!(ns_ext, Namespace::Any))
                                }
                                _ => false,
                            }),
                            _ => ext_comp.0.contains(s),
                        })
                    })
                })
        })
    })
}

/// 在单个复杂选择器上执行 extend。
fn extend_complex(
    selector: &ComplexSelector,
    extendee: &ComplexSelector,
    extender: &Selector,
) -> Option<Selector> {
    if has_multiple_combinators(selector) || extender_has_multiple_combinators(extender) {
        return None;
    }

    let sel_leading = has_leading_combinator(selector);
    let sel_trailing = has_trailing_combinator(selector);
    let ext_leading = extender_has_leading_combinator(extender);
    let ext_trailing = extender_has_trailing_combinator(extender);
    if sel_leading && ext_leading {
        return None;
    }
    if sel_trailing && ext_trailing {
        return None;
    }

    if extendee.compounds.len() != 1 {
        return try_exact_complex_match(selector, extendee, extender);
    }

    let ext_compound = &extendee.compounds[0].1;

    let match_positions: Vec<usize> = selector
        .compounds
        .iter()
        .enumerate()
        .filter(|(_, (_, sel_compound))| is_subset_compound(ext_compound, sel_compound))
        .map(|(i, _)| i)
        .collect();

    if match_positions.is_empty() {
        return None;
    }

    let unique_results: Vec<ComplexSelector> = match_positions
        .iter()
        .filter_map(|&match_pos| {
            let (_, sel_compound) = &selector.compounds[match_pos];
            let prefix = &selector.compounds[..match_pos];
            let suffix = &selector.compounds[match_pos + 1..];
            let is_last = match_pos == selector.compounds.len() - 1;
            let has_prefix = !prefix.is_empty();
            let extender_has_multiple = extender.0.iter().any(|c| c.compounds.len() > 1);

            let is_no_op = is_last && has_prefix && !extender_has_multiple;

            let remaining: Vec<SimpleSelector> = sel_compound
                .0
                .iter()
                .filter(|s| !ext_compound.0.contains(s))
                .cloned()
                .collect();

            let has_conflict = !remaining.is_empty()
                && extender
                    .0
                    .first()
                    .and_then(|c| c.compounds.first())
                    .map(|(_, ext_comp)| compounds_conflict(&remaining, ext_comp))
                    .unwrap_or(false);

            if is_no_op {
                tracing::debug!("extend_complex: NO-OP triggered");
            }

            (!is_no_op && !has_conflict).then(|| {
                extender
                    .0
                    .iter()
                    .flat_map(|ext_complex| {
                        build_extended_complex(
                            selector,
                            ext_complex,
                            match_pos,
                            &remaining,
                            sel_leading,
                            sel_trailing,
                        )
                    })
                    .collect::<Vec<_>>()
            })
        })
        .flatten()
        .fold(Vec::new(), |mut acc, c| {
            if !acc.contains(&c) {
                acc.push(c);
            }
            acc
        });

    (!unique_results.is_empty()).then_some(Selector(unique_results))
}

/// 检查 remaining simples 是否与 extender compound 冲突。
/// 命名空间感知版本。
fn compounds_conflict(remaining: &[SimpleSelector], ext_compound: &CompoundSelector) -> bool {
    let rem_type = remaining.iter().find(|s| matches!(s, SimpleSelector::Type { .. }));
    let ext_type = ext_compound
        .0
        .iter()
        .find(|s| matches!(s, SimpleSelector::Type { .. }));

    let type_conflict = match (rem_type, ext_type) {
        (
            Some(SimpleSelector::Type {
                namespace: ns_r,
                name: name_r,
            }),
            Some(SimpleSelector::Type {
                namespace: ns_e,
                name: name_e,
            }),
        ) => {
            // 同名 Type 但命名空间不兼容 → 冲突
            name_r == name_e && !namespaces_compatible(ns_r, ns_e)
        }
        _ => false,
    };

    let rem_id = remaining.iter().find_map(|s| match s {
        SimpleSelector::Id(i) => Some(i),
        _ => None,
    });
    let ext_id = ext_compound.0.iter().find_map(|s| match s {
        SimpleSelector::Id(i) => Some(i),
        _ => None,
    });
    let id_conflict = rem_id.zip(ext_id).is_some_and(|(r, e)| r != e);

    let rem_pe = remaining
        .iter()
        .find(|s| matches!(s, SimpleSelector::PseudoElement { .. }));
    let ext_pe = ext_compound
        .0
        .iter()
        .find(|s| matches!(s, SimpleSelector::PseudoElement { .. }));
    let pe_conflict = match (rem_pe, ext_pe) {
        (Some(r), Some(e)) => !pseudo_element_eq_normalized(r, e),
        _ => false,
    };

    type_conflict || id_conflict || pe_conflict
}

/// 构建扩展后的 complex selector。
fn build_extended_complex(
    selector: &ComplexSelector,
    ext_complex: &ComplexSelector,
    match_pos: usize,
    remaining: &[SimpleSelector],
    sel_leading: bool,
    sel_trailing: bool,
) -> Vec<ComplexSelector> {
    let prefix = &selector.compounds[..match_pos];
    let suffix = &selector.compounds[match_pos + 1..];
    let orig_combinator = selector.compounds[match_pos].0;

    let ext_first_combinator = ext_complex.compounds.first().and_then(|(c, _)| *c);
    let resolved_first_combinator = if sel_leading {
        orig_combinator.or(ext_first_combinator).or(Some(Combinator::Descendant))
    } else {
        ext_first_combinator.or(orig_combinator)
    };

    let remaining_to_attach = remaining;

    let results: Vec<Vec<(Option<Combinator>, CompoundSelector)>> = match ext_complex.compounds.len() {
        0 => vec![vec![(
            resolved_first_combinator.or(Some(Combinator::Descendant)),
            CompoundSelector(Vec::new()),
        )]],
        1 if ext_complex.compounds[0].1.0.is_empty() => vec![vec![(
            resolved_first_combinator.or(Some(Combinator::Descendant)),
            CompoundSelector(Vec::new()),
        )]],
        1 => {
            let ext_sims = &ext_complex.compounds[0].1 .0;
            let merged: Vec<SimpleSelector> = remaining_to_attach
                .iter()
                .cloned()
                .chain(ext_sims.iter().cloned())
                .collect();
            let mut result = prefix.to_vec();
            (!merged.is_empty()).then(|| {
                result.push((resolved_first_combinator, CompoundSelector(merged)));
            });
            vec![result]
        }
        _ => {
            let last_idx = ext_complex.compounds.len() - 1;
            let ext_has_trailing = ext_complex.compounds[last_idx].1.0.is_empty();

            match ext_has_trailing {
                true => {
                    let trailing_combinator = ext_complex.compounds[last_idx].0;
                    let (ext_first_comb, first_comp) = &ext_complex.compounds[0];

                    let head: Vec<_> = (!first_comp.0.is_empty())
                        .then_some((*ext_first_comb, first_comp.clone()))
                        .into_iter()
                        .collect();

                    let prefix_combinator = trailing_combinator.or(Some(Combinator::Descendant));
                    let tail: Vec<_> = selector
                        .compounds
                        .iter()
                        .enumerate()
                        .map(|(i, (c, comp))| {
                            (if i == 0 { prefix_combinator } else { *c }, comp.clone())
                        })
                        .collect();

                    vec![head.into_iter().chain(tail).collect()]
                }
                false => {
                    let parent_idx = last_idx - 1;
                    let ext_head = &ext_complex.compounds[..parent_idx];
                    let (_, ext_parent) = &ext_complex.compounds[parent_idx];
                    let (_, ext_tail) = &ext_complex.compounds[last_idx];

                    let merged_tail: Vec<SimpleSelector> = remaining_to_attach
                        .iter()
                        .cloned()
                        .chain(ext_tail.0.iter().cloned())
                        .collect();
                    let tail_comp = (None, CompoundSelector(merged_tail));

                    let is_two_compound = ext_complex.compounds.len() <= 2;

                    if is_two_compound {
                        let parent_comp = (None, ext_parent.clone());
                        let prefix_part: Vec<_> = prefix
                            .iter()
                            .enumerate()
                            .map(|(i, (c, comp))| {
                                (if i == 0 { Some(Combinator::Descendant) } else { *c }, comp.clone())
                            })
                            .collect();

                        match prefix.len() {
                            0 => vec![std::iter::once(parent_comp)
                                .chain(std::iter::once(tail_comp))
                                .collect()],
                            1 => vec![std::iter::once(parent_comp)
                                .chain(prefix_part)
                                .chain(std::iter::once(tail_comp))
                                .collect()],
                            _ => vec![prefix_part
                                .into_iter()
                                .chain(std::iter::once(parent_comp))
                                .chain(std::iter::once(tail_comp))
                                .collect()],
                        }
                    } else {
                        let prefix_for_insert = if prefix.is_empty() {
                            prefix
                        } else {
                            &prefix[..prefix.len() - 1]
                        };

                        (0..=ext_head.len())
                            .map(|split| {
                                let prefix_combinator = ext_head
                                    .first()
                                    .and_then(|(c, _)| *c)
                                    .or(Some(Combinator::Descendant));

                                let prefix_part: Vec<_> = prefix_for_insert
                                    .iter()
                                    .enumerate()
                                    .map(|(i, (c, comp))| {
                                        (if i == 0 { prefix_combinator } else { *c }, comp.clone())
                                    })
                                    .collect();

                                let before_split: Vec<_> = ext_head[..split]
                                    .iter()
                                    .map(|(c, comp)| (*c, comp.clone()))
                                    .collect();
                                let after_split: Vec<_> = ext_head[split..]
                                    .iter()
                                    .map(|(c, comp)| (*c, comp.clone()))
                                    .collect();

                                let parent_comp = (None, ext_parent.clone());

                                before_split
                                    .into_iter()
                                    .chain(prefix_part)
                                    .chain(after_split)
                                    .chain(std::iter::once(parent_comp))
                                    .chain(std::iter::once(tail_comp.clone()))
                                    .collect::<Vec<_>>()
                            })
                            .collect()
                    }
                }
            }
        }
    };

    results
        .into_iter()
        .map(|built| {
            let with_suffix: Vec<_> = built.into_iter().chain(suffix.iter().cloned()).collect();
            let len = with_suffix.len();
            let adjusted: Vec<_> = match (sel_trailing, len) {
                (true, 0) => with_suffix,
                (true, _) => with_suffix
                    .into_iter()
                    .enumerate()
                    .map(|(i, (c, comp))| {
                        (if i == len - 1 { orig_combinator.or(c) } else { c }, comp)
                    })
                    .collect(),
                _ => with_suffix,
            };
            adjusted
                .into_iter()
                .filter(|(comb, comp)| !comp.0.is_empty() || comb.is_some())
                .collect::<Vec<_>>()
        })
        .filter(|filtered| !filtered.is_empty())
        .map(|filtered| ComplexSelector { compounds: filtered })
        .collect()
}

/// 精确匹配多 compound extendee。
fn try_exact_complex_match(
    selector: &ComplexSelector,
    extendee: &ComplexSelector,
    extender: &Selector,
) -> Option<Selector> {
    let sel_compounds: Vec<&CompoundSelector> = selector.compounds.iter().map(|(_, c)| c).collect();
    let ext_compounds: Vec<&CompoundSelector> = extendee.compounds.iter().map(|(_, c)| c).collect();

    if sel_compounds.len() < ext_compounds.len() {
        return None;
    }

    (0..=sel_compounds.len() - ext_compounds.len())
        .find(|&start| {
            let slice = &sel_compounds[start..start + ext_compounds.len()];
            slice.iter().zip(ext_compounds.iter()).all(|(s, e)| {
                is_super_compound(e, s) && is_super_compound(s, e)
            })
        })
        .map(|start| {
            let prefix = &selector.compounds[..start];
            let suffix = &selector.compounds[start + ext_compounds.len()..];

            let results: Vec<ComplexSelector> = extender
                .0
                .iter()
                .map(|ext_complex| {
                    let compounds: Vec<(Option<Combinator>, CompoundSelector)> = prefix
                        .iter()
                        .cloned()
                        .chain(ext_complex.compounds.iter().enumerate().map(|(i, (c, comp))| {
                            let comb = if i == 0 {
                                c.or(Some(Combinator::Descendant))
                            } else {
                                *c
                            };
                            (comb, comp.clone())
                        }))
                        .chain(suffix.iter().cloned())
                        .collect();

                    ComplexSelector { compounds }
                })
                .collect();

            Selector(results)
        })
}

/// 检查 complex selector 的第一个 compound 是否有 leading combinator。
fn has_leading_combinator(complex: &ComplexSelector) -> bool {
    complex
        .compounds
        .first()
        .is_some_and(|(comb, _)| comb.is_some())
}

/// 检查 complex selector 的最后一个 compound 是否有 trailing combinator。
fn has_trailing_combinator(complex: &ComplexSelector) -> bool {
    complex
        .compounds
        .last()
        .is_some_and(|(comb, comp)| comb.is_some() && comp.0.is_empty())
}

/// 检查 extender Selector 的任意 complex 是否有 leading combinator。
fn extender_has_leading_combinator(extender: &Selector) -> bool {
    extender.0.iter().any(|c| has_leading_combinator(c))
}

/// 检查 extender Selector 的任意 complex 是否有 trailing combinator。
fn extender_has_trailing_combinator(extender: &Selector) -> bool {
    extender.0.iter().any(|c| has_trailing_combinator(c))
}

/// 替换选择器：在 `selector` 中查找匹配 `original` 的部分，用 `replacement` 替换。
#[tracing::instrument(level = "info", fields(original = %original, replacement = %replacement))]
pub fn replace_selector(
    selector: &Selector,
    original: &Selector,
    replacement: &Selector,
) -> Selector {
    let results: Vec<ComplexSelector> = selector
        .0
        .iter()
        .flat_map(|complex| {
            let replaced = original
                .0
                .iter()
                .find_map(|oc| replace_complex(complex, oc, replacement));
            match replaced {
                Some(sel) => sel.0.into_iter().collect::<Vec<_>>(),
                None => vec![complex.clone()],
            }
        })
        .fold(Vec::new(), |mut acc, c| {
            if !acc.contains(&c) {
                acc.push(c);
            }
            acc
        });

    Selector(results)
}

/// 在单个复杂选择器上执行替换。
fn replace_complex(
    selector: &ComplexSelector,
    original: &ComplexSelector,
    replacement: &Selector,
) -> Option<Selector> {
    let sel_compounds: Vec<&CompoundSelector> = selector.compounds.iter().map(|(_, c)| c).collect();
    let orig_compounds: Vec<&CompoundSelector> = original.compounds.iter().map(|(_, c)| c).collect();

    if sel_compounds.len() >= orig_compounds.len() {
        let start = sel_compounds.len() - orig_compounds.len();
        let suffix = &sel_compounds[start..];

        let matches = suffix
            .iter()
            .zip(orig_compounds.iter())
            .all(|(s, o)| is_super_compound(o, s) && is_super_compound(s, o));

        if matches {
            let prefix = &selector.compounds[..start];
            let results: Vec<ComplexSelector> = replacement
                .0
                .iter()
                .map(|rep_complex| {
                    let compounds: Vec<(Option<Combinator>, CompoundSelector)> = prefix
                        .iter()
                        .cloned()
                        .chain(rep_complex.compounds.iter().cloned())
                        .collect();
                    ComplexSelector { compounds }
                })
                .collect();
            return (!results.is_empty()).then_some(Selector(results));
        }
    }

    if orig_compounds.len() == 1 {
        let orig_compound = orig_compounds[0];

        let found = selector.compounds.iter().enumerate().find(|(_, (_, sel_compound))| {
            orig_compound
                .0
                .iter()
                .all(|orig_simple| sel_compound.0.contains(orig_simple))
        });

        if let Some((_idx, (comb, sel_compound))) = found {
            if let Some(rep_complex) = replacement.0.first() {
                if let Some((_, rep_compound)) = rep_complex.compounds.first() {
                    let mut new_simples: Vec<SimpleSelector> = Vec::new();
                    let mut replacement_done = false;

                    for simple in &sel_compound.0 {
                        if orig_compound.0.contains(simple) && !replacement_done {
                            new_simples.extend(rep_compound.0.clone());
                            replacement_done = true;
                        } else if !orig_compound.0.contains(simple) {
                            new_simples.push(simple.clone());
                        }
                    }

                    let new_compound = CompoundSelector(new_simples);
                    let new_complex = ComplexSelector {
                        compounds: vec![(*comb, new_compound)],
                    };
                    return Some(Selector(vec![new_complex]));
                }
            }
        }
    }

    None
}
