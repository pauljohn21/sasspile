//! —— extend 结果构建算法 ——

use super::super::selector_ast::{Combinator, ComplexSelector, CompoundSelector, Selector, SimpleSelector};
use super::super::selector_is_super::is_super_compound;

pub(super) fn build_extended_complex(
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
        0 | 1 if ext_complex.compounds.first().is_none_or(|(c, comp)| c.is_some() && comp.0.is_empty()) => {
            vec![vec![(resolved_first_combinator.or(Some(Combinator::Descendant)), CompoundSelector(Vec::new()))]]
        }
        1 => {
            let ext_sims = &ext_complex.compounds[0].1 .0;
            let ext_is_type = ext_sims.iter().any(|s| matches!(s, SimpleSelector::Type { .. }));
            let merged: Vec<SimpleSelector> = if ext_is_type {
                ext_sims.iter().cloned().chain(remaining.iter().cloned()).collect()
            } else {
                remaining.iter().cloned().chain(ext_sims.iter().cloned()).collect()
            };
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

pub(super) fn replace_compound_with_ext(
    selector: &ComplexSelector,
    extender: &Selector,
    match_pos: usize,
    ext_has_non_descendant: bool,
    sel_leading: bool,
    sel_trailing: bool,
    _orig_combinator_0: Option<Combinator>,
) -> Option<Vec<ComplexSelector>> {
    let sel_comb_at_match = selector.compounds[match_pos].0;
    let has_combinator_conflict = ext_has_non_descendant
        && matches!(sel_comb_at_match, Some(Combinator::Child) | Some(Combinator::Adjacent) | Some(Combinator::Sibling));

    if has_combinator_conflict { return None; }

    let remaining: Vec<SimpleSelector> = Vec::new();
    Some(extender.0.iter().flat_map(|ext_complex| {
        build_extended_complex(selector, ext_complex, match_pos, &remaining, sel_leading, sel_trailing)
    }).collect())
}

pub(super) fn try_exact_complex_match(selector: &ComplexSelector, extendee: &ComplexSelector, extender: &Selector) -> Option<Selector> {
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
