//! extend/replace 算法——选择器扩展与替换。

use super::selector_ast::{Combinator, ComplexSelector, CompoundSelector, Namespace, Selector, SimpleSelector};
use super::selector_ops::{
    compounds_conflict, extender_has_leading_combinator, extender_has_multiple_combinators,
    extender_has_trailing_combinator, has_leading_combinator, has_multiple_combinators,
    has_trailing_combinator, is_semantic_subset, selector_simple_covers_ext, simple_contained_in,
};
use super::selector_is_super::is_super_compound;
use super::selector_unify::unify_extendee_list;

#[tracing::instrument(level = "info", fields(extendee = %extendee, extender = %extender))]
pub fn extend_selector(selector: &Selector, extendee: &Selector, extender: &Selector) -> Selector {
    extend_selector_with_mode(selector, extendee, extender, false)
}

/// 带模式选择的 extend_selector。
///
/// `full_match_only` 为 true 时（selector 以 List 形式传入），仅进行 FULL 复杂匹配：
/// extendee 的 compound 数量必须与 selector 的 complex compound 数量完全相等。
#[tracing::instrument(level = "info", fields(extendee = %extendee, extender = %extender, full_match_only))]
pub fn extend_selector_with_mode(selector: &Selector, extendee: &Selector, extender: &Selector, full_match_only: bool) -> Selector {
    let unified_extendee = unify_extendee_list(extendee);
    tracing::debug!(unified = ?unified_extendee, "extend: unified extendee");

    // 检查 selector 是否有 *|type 模式是 extendee 的超集（需要更细致的检查）
    let selector_any_namespace_no_op = {
        // 如果 selector 有 Type{Any, name} 但 extendee 没有对应的 Type{Any, name}，
        // 且 extendee 有 Type{其他namespace, name}，则 extend 是 no-op
        let sel_has_any_type = selector.0.iter().any(|c| {
            c.compounds.iter().any(|(_, comp)| comp.0.iter().any(|s| matches!(s, SimpleSelector::Type { namespace: Namespace::Any, .. })))
        });
        let ext_has_any_type = extendee.0.iter().any(|c| {
            c.compounds.iter().any(|(_, comp)| comp.0.iter().any(|s| matches!(s, SimpleSelector::Type { namespace: Namespace::Any, .. })))
        });
        sel_has_any_type && !ext_has_any_type
    };

    // 如果 selector 包含 Universal 或 *|*（any namespace + wildcard name），任何 extend 都是 no-op
    // *|* 等同于 Universal（匹配任何命名空间的任何类型）
    let selector_has_universal = selector.0.iter().any(|c| {
        c.compounds.iter().any(|(_, comp)| comp.0.iter().any(|s| {
            matches!(s, SimpleSelector::Universal)
                || matches!(s, SimpleSelector::Type { namespace: Namespace::Any, name: n } if n == "*")
        }))
    });
    let is_no_op = unified_extendee.is_none() || selector_has_universal || selector_any_namespace_no_op || is_more_specific_than(extender, extendee);
    if is_no_op {
        tracing::debug!("extend: NO-OP");
        return selector.clone();
    }
    let unified_extendee = unified_extendee.expect("checked");
    let results: Vec<ComplexSelector> = selector.0.iter().flat_map(|complex| {
        let original = std::iter::once(complex.clone());
        let extendee_matches_complex = full_match_only && unified_extendee.compounds.len() != complex.compounds.len();
        let extended = if extendee_matches_complex {
            // list 模式：extendee compound 数量不匹配，跳过此 complex
            None
        } else {
            extend_complex(complex, &unified_extendee, extender)
        };
        let extended_items = extended.map(|s| s.0.into_iter()).unwrap_or_default();
        original.chain(extended_items)
    }).fold(Vec::new(), |mut acc, c| { if !acc.contains(&c) { acc.push(c); } acc });
    let result = Selector(results);
    tracing::debug!(%result, "extend: result");
    result
}

fn is_more_specific_than(extender: &Selector, extendee: &Selector) -> bool {
    // extender 比 extendee 更具体 → extend 是 no-op
    // 特殊情况1：extendee 包含 Universal → 任何 extender 都更具体 → no-op
    // 特殊情况2：selector（原始选择器）包含 Universal → 任何 extend 都是 no-op
    //   （因为 * 已经匹配所有元素，无法通过 extend 增加新信息）
    extender.0.iter().all(|ext_c| {
        extendee.0.iter().any(|ee_c| {
            // 如果 extendee 的 compound 包含 Universal，任何 extender 都更具体
            let ee_has_universal = ee_c.compounds.iter().any(|(_, comp)| comp.0.iter().any(|s| matches!(s, SimpleSelector::Universal)));
            if ee_has_universal { return true; }
            
            ext_c.compounds.len() >= ee_c.compounds.len()
                && ee_c.compounds.iter().enumerate().all(|(i, (_, ee_comp))| {
                    ext_c.compounds.get(i).is_some_and(|(_, ext_comp)| {
                        ee_comp.0.iter().all(|s| match s {
                            SimpleSelector::Type { namespace: ns_ee, name: name_ee } => ext_comp.0.iter().any(|es| match es {
                                SimpleSelector::Type { namespace: ns_ext, name: name_ext } => {
                                    name_ee == name_ext && (ns_ee == ns_ext || matches!(ns_ee, Namespace::Any) || matches!(ns_ext, Namespace::Any))
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

fn extend_complex(selector: &ComplexSelector, extendee: &ComplexSelector, extender: &Selector) -> Option<Selector> {
    if has_multiple_combinators(selector) || extender_has_multiple_combinators(extender) { return None; }
    let sel_leading = has_leading_combinator(selector);
    let sel_trailing = has_trailing_combinator(selector);
    let ext_leading = extender_has_leading_combinator(extender);
    let ext_trailing = extender_has_trailing_combinator(extender);
    if sel_leading && ext_leading { return None; }
    if sel_trailing && ext_trailing { return None; }
    // 检查 extender 是否有非 descendant 组合器（>、+、~）
    let ext_has_non_descendant = extender.0.iter().any(|c| {
        c.compounds.iter().any(|(comb, _)| matches!(comb, Some(Combinator::Child) | Some(Combinator::Adjacent) | Some(Combinator::Sibling)))
    });
    if extendee.compounds.len() != 1 { return try_exact_complex_match(selector, extendee, extender); }

    let ext_compound = &extendee.compounds[0].1;
    let match_positions: Vec<usize> = selector.compounds.iter().enumerate()
        .filter(|(_, (_, sel_compound))| is_semantic_subset(ext_compound, sel_compound))
        .map(|(i, _)| i).collect();
    if match_positions.is_empty() { return None; }

    let unique_results: Vec<ComplexSelector> = match_positions.iter().filter_map(|&match_pos| {
        let (_, sel_compound) = &selector.compounds[match_pos];
        // remaining = simples in sel_compound that are NOT consumed by ext_compound
        // A sel_simplex is consumed if it covers (is more general than or equal to) some ext_simplex
        let remaining: Vec<SimpleSelector> = sel_compound.0.iter()
            .filter(|s| !ext_compound.0.iter().any(|ext_s| selector_simple_covers_ext(s, ext_s)))
            .cloned()
            .collect();
        let has_type_conflict = !remaining.is_empty() && extender.0.first()
            .and_then(|c| c.compounds.first())
            .map(|(_, ext_comp)| compounds_conflict(&remaining, ext_comp))
            .unwrap_or(false);
        // 检查：selector 在 match_pos 处有非 descendant 组合器，且 extender 也有非 descendant 组合器 → 冲突
        let sel_comb_at_match = selector.compounds[match_pos].0;
        let has_combinator_conflict = ext_has_non_descendant
            && matches!(sel_comb_at_match, Some(Combinator::Child) | Some(Combinator::Adjacent) | Some(Combinator::Sibling));

        (!has_type_conflict && !has_combinator_conflict).then(|| {
            extender.0.iter().flat_map(|ext_complex| {
                build_extended_complex(selector, ext_complex, match_pos, &remaining, sel_leading, sel_trailing)
            }).collect::<Vec<_>>()
        })
    }).flatten().fold(Vec::new(), |mut acc, c| { if !acc.contains(&c) { acc.push(c); } acc });

    (!unique_results.is_empty()).then_some(Selector(unique_results))
}

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

    let results: Vec<Vec<(Option<Combinator>, CompoundSelector)>> = match ext_complex.compounds.len() {
        0 | 1 if ext_complex.compounds.get(0).map_or(true, |(c, comp)| c.is_some() && comp.0.is_empty()) => {
            vec![vec![(resolved_first_combinator.or(Some(Combinator::Descendant)), CompoundSelector(Vec::new()))]]
        }
        1 => {
            let ext_sims = &ext_complex.compounds[0].1 .0;
            let merged: Vec<SimpleSelector> = remaining.iter().cloned().chain(ext_sims.iter().cloned()).collect();
            let mut result = prefix.to_vec();
            (!merged.is_empty()).then(|| { result.push((resolved_first_combinator, CompoundSelector(merged))); });
            vec![result]
        }
        _ => {
            let last_idx = ext_complex.compounds.len() - 1;
            let ext_has_trailing = ext_complex.compounds[last_idx].1.0.is_empty();
            match ext_has_trailing {
                true => {
                    let trailing_combinator = ext_complex.compounds[last_idx].0;
                    let (ext_first_comb, first_comp) = &ext_complex.compounds[0];
                    let head: Vec<_> = (!first_comp.0.is_empty()).then_some((*ext_first_comb, first_comp.clone())).into_iter().collect();
                    let prefix_combinator = trailing_combinator.or(Some(Combinator::Descendant));
                    let tail: Vec<_> = selector.compounds.iter().enumerate().map(|(i, (c, comp))| {
                        (if i == 0 { prefix_combinator } else { *c }, comp.clone())
                    }).collect();
                    vec![head.into_iter().chain(tail).collect()]
                }
                false => {
                    let parent_idx = last_idx - 1;
                    let ext_head = &ext_complex.compounds[..parent_idx];
                    let (_, ext_parent) = &ext_complex.compounds[parent_idx];
                    let (_, ext_tail) = &ext_complex.compounds[last_idx];
                    let merged_tail: Vec<SimpleSelector> = remaining.iter().cloned().chain(ext_tail.0.iter().cloned()).collect();
                    let tail_comp = (None, CompoundSelector(merged_tail));
                    let is_two_compound = ext_complex.compounds.len() <= 2;
                    if is_two_compound {
                        let parent_comp = (None, ext_parent.clone());
                        let prefix_part: Vec<_> = prefix.iter().enumerate().map(|(i, (c, comp))| {
                            (if i == 0 { Some(Combinator::Descendant) } else { *c }, comp.clone())
                        }).collect();
                        match prefix.len() {
                            0 => vec![std::iter::once(parent_comp).chain(std::iter::once(tail_comp)).collect()],
                            1 => vec![std::iter::once(parent_comp).chain(prefix_part).chain(std::iter::once(tail_comp)).collect()],
                            _ => vec![prefix_part.into_iter().chain(std::iter::once(parent_comp)).chain(std::iter::once(tail_comp)).collect()],
                        }
                    } else {
                        let prefix_for_insert = if prefix.is_empty() { prefix } else { &prefix[..prefix.len() - 1] };
                        (0..=ext_head.len()).map(|split| {
                            let prefix_combinator = ext_head.first().and_then(|(c, _)| *c).or(Some(Combinator::Descendant));
                            let prefix_part: Vec<_> = prefix_for_insert.iter().enumerate().map(|(i, (c, comp))| {
                                (if i == 0 { prefix_combinator } else { *c }, comp.clone())
                            }).collect();
                            let before_split: Vec<_> = ext_head[..split].iter().map(|(c, comp)| (*c, comp.clone())).collect();
                            let after_split: Vec<_> = ext_head[split..].iter().map(|(c, comp)| (*c, comp.clone())).collect();
                            let parent_comp = (None, ext_parent.clone());
                            before_split.into_iter().chain(prefix_part).chain(after_split)
                                .chain(std::iter::once(parent_comp)).chain(std::iter::once(tail_comp.clone())).collect::<Vec<_>>()
                        }).collect()
                    }
                }
            }
        }
    };

    results.into_iter().map(|built| {
        let with_suffix: Vec<_> = built.into_iter().chain(suffix.iter().cloned()).collect();
        let len = with_suffix.len();
        let adjusted: Vec<_> = match (sel_trailing, len) {
            (true, 0) => with_suffix,
            (true, _) => with_suffix.into_iter().enumerate().map(|(i, (c, comp))| {
                (if i == len - 1 { orig_combinator.or(c) } else { c }, comp)
            }).collect(),
            _ => with_suffix,
        };
        adjusted.into_iter().filter(|(comb, comp)| !comp.0.is_empty() || comb.is_some()).collect::<Vec<_>>()
    }).filter(|f| !f.is_empty()).map(|f| ComplexSelector { compounds: f }).collect()
}

fn try_exact_complex_match(selector: &ComplexSelector, extendee: &ComplexSelector, extender: &Selector) -> Option<Selector> {
    let sel_compounds: Vec<&CompoundSelector> = selector.compounds.iter().map(|(_, c)| c).collect();
    let ext_compounds: Vec<&CompoundSelector> = extendee.compounds.iter().map(|(_, c)| c).collect();
    if sel_compounds.len() < ext_compounds.len() { return None; }
    (0..=sel_compounds.len() - ext_compounds.len()).find(|&start| {
        let slice = &sel_compounds[start..start + ext_compounds.len()];
        slice.iter().zip(ext_compounds.iter()).all(|(s, e)| is_super_compound(e, s) && is_super_compound(s, e))
    }).map(|start| {
        let prefix = &selector.compounds[..start];
        let suffix = &selector.compounds[start + ext_compounds.len()..];
        let results: Vec<ComplexSelector> = extender.0.iter().map(|ext_complex| {
            let compounds: Vec<(Option<Combinator>, CompoundSelector)> = prefix.iter().cloned()
                .chain(ext_complex.compounds.iter().enumerate().map(|(i, (c, comp))| {
                    let comb = if i == 0 { c.or(Some(Combinator::Descendant)) } else { *c };
                    (comb, comp.clone())
                })).chain(suffix.iter().cloned()).collect();
            ComplexSelector { compounds }
        }).collect();
        Selector(results)
    })
}

#[tracing::instrument(level = "info", fields(original = %original, replacement = %replacement))]
pub fn replace_selector(selector: &Selector, original: &Selector, replacement: &Selector) -> Selector {
    let results: Vec<ComplexSelector> = selector.0.iter().flat_map(|complex| {
        let replaced = original.0.iter().find_map(|oc| replace_complex(complex, oc, replacement));
        match replaced {
            Some(sel) => sel.0.into_iter().collect::<Vec<_>>(),
            None => vec![complex.clone()],
        }
    }).fold(Vec::new(), |mut acc, c| { if !acc.contains(&c) { acc.push(c); } acc });
    Selector(results)
}

fn replace_complex(selector: &ComplexSelector, original: &ComplexSelector, replacement: &Selector) -> Option<Selector> {
    let sel_compounds: Vec<&CompoundSelector> = selector.compounds.iter().map(|(_, c)| c).collect();
    let orig_compounds: Vec<&CompoundSelector> = original.compounds.iter().map(|(_, c)| c).collect();
    if sel_compounds.len() >= orig_compounds.len() {
        let start = sel_compounds.len() - orig_compounds.len();
        let suffix = &sel_compounds[start..];
        let matches = suffix.iter().zip(orig_compounds.iter()).all(|(s, o)| is_super_compound(o, s) && is_super_compound(s, o));
        if matches {
            let prefix = &selector.compounds[..start];
            let results: Vec<ComplexSelector> = replacement.0.iter().map(|rep_complex| {
                let compounds: Vec<(Option<Combinator>, CompoundSelector)> = prefix.iter().cloned()
                    .chain(rep_complex.compounds.iter().cloned()).collect();
                ComplexSelector { compounds }
            }).collect();
            return (!results.is_empty()).then_some(Selector(results));
        }
    }
    if orig_compounds.len() == 1 {
        let orig_compound = orig_compounds[0];
        let found = selector.compounds.iter().enumerate().find(|(_, (_, sel_compound))| {
            orig_compound.0.iter().all(|orig_simple| sel_compound.0.contains(orig_simple))
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
                    return Some(Selector(vec![ComplexSelector { compounds: vec![(*comb, CompoundSelector(new_simples))] }]));
                }
            }
        }
    }
    None
}
