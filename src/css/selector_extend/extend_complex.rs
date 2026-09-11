//! —— extend_complex 主匹配逻辑 ——

use super::super::selector_ast::{Combinator, ComplexSelector, CompoundSelector, Selector, SimpleSelector};
use super::super::selector_ops::{
    compounds_conflict, has_leading_combinator, has_multiple_combinators, has_trailing_combinator,
    is_semantic_subset, selector_simple_covers_ext,
};
use super::super::selector_ops::{extender_has_leading_combinator, extender_has_multiple_combinators, extender_has_trailing_combinator};
use super::extend_pseudo::{
    extender_contains_not, extract_not_arg, flatten_extender_simples, has_not_pseudo,
};
use super::extend_build::{build_extended_complex, try_exact_complex_match};

pub(super) fn extend_complex(selector: &ComplexSelector, extendee: &ComplexSelector, extender: &Selector) -> Option<Selector> {
    if let Some(result) = try_extend_not_in_complex(selector, extendee, extender) {
        return Some(result);
    }
    if let Some(result) = try_extend_is_where_matches_in_complex(selector, extendee, extender) {
        return Some(result);
    }
    if has_multiple_combinators(selector) || extender_has_multiple_combinators(extender) { return None; }
    let sel_leading = has_leading_combinator(selector);
    let sel_trailing = has_trailing_combinator(selector);
    let ext_leading = extender_has_leading_combinator(extender);
    let ext_trailing = extender_has_trailing_combinator(extender);
    if sel_leading && ext_leading { return None; }
    if sel_trailing && ext_trailing { return None; }
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
        extend_complex_at(selector, ext_compound, extender, match_pos, ext_has_non_descendant, sel_leading, sel_trailing)
    }).flatten().fold(Vec::new(), |mut acc, c| { if !acc.contains(&c) { acc.push(c); } acc });

    (!unique_results.is_empty()).then_some(Selector(unique_results))
}

fn extend_complex_at(
    selector: &ComplexSelector,
    ext_compound: &CompoundSelector,
    extender: &Selector,
    match_pos: usize,
    ext_has_non_descendant: bool,
    sel_leading: bool,
    sel_trailing: bool,
) -> Option<Vec<ComplexSelector>> {
    let (_, sel_compound) = &selector.compounds[match_pos];
    let remaining: Vec<SimpleSelector> = sel_compound.0.iter()
        .filter(|s| !ext_compound.0.iter().any(|ext_s| selector_simple_covers_ext(s, ext_s)))
        .cloned()
        .collect();
    let has_type_conflict = !remaining.is_empty() && extender.0.first()
        .and_then(|c| c.compounds.first())
        .map(|(_, ext_comp)| compounds_conflict(&remaining, ext_comp))
        .unwrap_or(false);
    let sel_comb_at_match = selector.compounds[match_pos].0;
    let has_combinator_conflict = ext_has_non_descendant
        && matches!(sel_comb_at_match, Some(Combinator::Child) | Some(Combinator::Adjacent) | Some(Combinator::Sibling));

    (!has_type_conflict && !has_combinator_conflict).then(|| {
        extender.0.iter().flat_map(|ext_complex| {
            build_extended_complex(selector, ext_complex, match_pos, &remaining, sel_leading, sel_trailing)
        }).collect::<Vec<_>>()
    })
}

fn try_extend_is_where_matches_in_complex(
    selector: &ComplexSelector,
    extendee: &ComplexSelector,
    extender: &Selector,
) -> Option<Selector> {
    if extendee.compounds.len() != 1 { return None; }
    let ext_compound = &extendee.compounds[0].1;

    let mut match_positions: Vec<(usize, usize)> = Vec::new();
    for (ci, (_, compound)) in selector.compounds.iter().enumerate() {
        for (si, simple) in compound.0.iter().enumerate() {
            let (_name, arg) = match simple {
                SimpleSelector::PseudoClass { name, arg: Some(arg) }
                    if name == "is" || name == "where" || name == "matches" => (name, arg),
                _ => continue,
            };
            let arg_selector = super::super::selector_parser::parse_selector(arg);
            let arg_matches = arg_selector.0.iter().any(|arg_complex| {
                arg_complex.compounds.iter().any(|(_, arg_comp)| {
                    super::super::selector_ops::is_semantic_subset(ext_compound, arg_comp)
                })
            });
            if arg_matches { match_positions.push((ci, si)); }
        }
    }

    if match_positions.is_empty() { return None; }

    let mut extended_complexes: Vec<ComplexSelector> = Vec::new();
    for (compound_idx, simple_idx) in &match_positions {
        let (comb, old_compound) = &selector.compounds[*compound_idx];
        let (pseudo_name, pseudo_arg) = match old_compound.0.get(*simple_idx) {
            Some(SimpleSelector::PseudoClass { name, arg: Some(arg) })
                if name == "is" || name == "where" || name == "matches" => (name.clone(), arg.clone()),
            _ => continue,
        };

        let extender_str = extender.to_string();
        let new_arg = format!("{pseudo_arg}, {extender_str}");
        let new_simple = SimpleSelector::PseudoClass { name: pseudo_name, arg: Some(new_arg) };

        let mut new_simples: Vec<SimpleSelector> = old_compound.0.clone();
        new_simples[*simple_idx] = new_simple;
        let mut new_compounds = selector.compounds.clone();
        new_compounds[*compound_idx] = (*comb, CompoundSelector(new_simples));
        extended_complexes.push(ComplexSelector { compounds: new_compounds });
    }

    if extended_complexes.is_empty() { return None; }
    Some(Selector(extended_complexes))
}

fn try_extend_not_in_complex(selector: &ComplexSelector, extendee: &ComplexSelector, extender: &Selector) -> Option<Selector> {
    if extendee.compounds.len() != 1 { return None; }
    let ext_compound = &extendee.compounds[0].1;
    let sel_leading = has_leading_combinator(selector);
    let sel_trailing = has_trailing_combinator(selector);

    let not_positions: Vec<usize> = selector.compounds.iter().enumerate()
        .filter(|(_, (_, comp))| has_not_pseudo(comp))
        .map(|(i, _)| i).collect();
    if not_positions.is_empty() { return None; }

    let mut all_results: Vec<ComplexSelector> = Vec::new();
    let mut any_extended = false;
    for &match_pos in &not_positions {
        let (_, sel_compound) = &selector.compounds[match_pos];
        if let Some(extended) = extend_not_pseudo(selector, sel_compound, ext_compound, extender, match_pos, sel_leading, sel_trailing) {
            for ec in extended {
                if !all_results.contains(&ec) { all_results.push(ec); }
            }
            any_extended = true;
        }
    }

    any_extended.then(|| {
        let original = selector.clone();
        let mut results = vec![original];
        results.extend(all_results);
        Selector(results)
    })
}

fn extend_not_pseudo(
    selector: &ComplexSelector,
    sel_compound: &CompoundSelector,
    ext_compound: &CompoundSelector,
    extender: &Selector,
    match_pos: usize,
    sel_leading: bool,
    _sel_trailing: bool,
) -> Option<Vec<ComplexSelector>> {
    let not_arg = extract_not_arg(sel_compound)?;
    let inner_selector = super::super::selector_parser::parse_selector(&not_arg);
    let inner_compound = inner_selector.0.first()?;

    if !super::super::selector_ops::is_semantic_subset(ext_compound, &inner_compound.compounds[0].1) {
        if ext_compound.0.len() != 1 { return None; }
        let ext_simple = &ext_compound.0[0];
        if !inner_compound.compounds[0].1 .0.iter().any(|s| s == ext_simple) { return None; }
    }

    if extender_contains_not(extender) { return None; }

    let extender_simples = flatten_extender_simples(extender);
    let prefix = &selector.compounds[..match_pos];
    let suffix = &selector.compounds[match_pos + 1..];
    let orig_combinator = selector.compounds[match_pos].0;
    let resolved_first_combinator = if sel_leading { orig_combinator.or(Some(Combinator::Descendant)) } else { orig_combinator };

    let extended_compound_with_nots: Vec<SimpleSelector> = sel_compound.0.iter().cloned()
        .chain(extender_simples.iter().map(|s| {
            let not_arg = match s {
                SimpleSelector::PseudoClass { name, arg: Some(arg) }
                    if name == "__compound__" || name == "__selector__" || name == "__complex__" => arg.clone(),
                _ => s.to_string(),
            };
            SimpleSelector::PseudoClass { name: "not".to_string(), arg: Some(not_arg) }
        }))
        .collect();

    let mut result_compounds: Vec<(Option<Combinator>, CompoundSelector)> = prefix.to_vec();
    result_compounds.push((resolved_first_combinator, CompoundSelector(extended_compound_with_nots)));
    result_compounds.extend(suffix.iter().cloned());

    Some(vec![ComplexSelector { compounds: result_compounds }])
}
