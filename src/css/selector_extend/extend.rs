//! —— 选择器 extend 入口函数 ——

use super::super::selector_ast::{Combinator, ComplexSelector, Namespace, Selector, SimpleSelector};
use super::super::selector_ops::{has_leading_combinator, has_trailing_combinator, is_semantic_subset};
use super::super::selector_unify::unify_extendee_list;
use super::extend_build::replace_compound_with_ext;
use super::extend_complex::extend_complex;

#[tracing::instrument(level = "info", fields(extendee = %extendee, extender = %extender))]
pub fn extend_selector(selector: &Selector, extendee: &Selector, extender: &Selector) -> Selector {
    extend_selector_with_mode(selector, extendee, extender, false)
}

#[tracing::instrument(level = "info", fields(extendee = %extendee, extender = %extender, full_match_only))]
pub fn extend_selector_with_mode(selector: &Selector, extendee: &Selector, extender: &Selector, full_match_only: bool) -> Selector {
    let unified_extendee = unify_extendee_list(extendee);
    tracing::debug!(unified = ?unified_extendee, "extend: unified extendee");

    let selector_any_namespace_no_op = {
        let sel_has_any_type = selector.0.iter().any(|c| {
            c.compounds.iter().any(|(_, comp)| comp.0.iter().any(|s| matches!(s, SimpleSelector::Type { namespace: Namespace::Any, .. })))
        });
        let ext_has_any_type = extendee.0.iter().any(|c| {
            c.compounds.iter().any(|(_, comp)| comp.0.iter().any(|s| matches!(s, SimpleSelector::Type { namespace: Namespace::Any, .. })))
        });
        sel_has_any_type && !ext_has_any_type
    };

    let selector_has_universal = selector.0.iter().any(|c| {
        c.compounds.iter().any(|(_, comp)| comp.0.iter().any(|s| {
            matches!(s, SimpleSelector::Universal)
                || matches!(s, SimpleSelector::Type { namespace: Namespace::Any, name: n } if n == "*")
        }))
    });

    if unified_extendee.is_none() && extendee.0.len() > 1 {
        tracing::debug!(?extendee, "extend: multi-extendee path");
        let sel_leading = selector.0.iter().any(|c| has_leading_combinator(c));
        let sel_trailing = selector.0.iter().any(|c| has_trailing_combinator(c));
        let ext_has_non_descendant = extender.0.iter().any(|c| {
            c.compounds.iter().any(|(comb, _)| matches!(comb, Some(Combinator::Child) | Some(Combinator::Adjacent) | Some(Combinator::Sibling)))
        });
        let mut results: Vec<ComplexSelector> = Vec::new();
        for complex in &selector.0 {
            results.push(complex.clone());
            let orig_combinator_0 = if complex.compounds.is_empty() { None } else { complex.compounds[0].0 };
            for ext_cs in &extendee.0 {
                if ext_cs.compounds.len() != 1 { continue; }
                let ext_compound = &ext_cs.compounds[0].1;
                let match_positions: Vec<usize> = complex.compounds.iter().enumerate()
                    .filter(|(_, (_, sel_compound))| is_semantic_subset(ext_compound, sel_compound))
                    .map(|(i, _)| i).collect();
                for &match_pos in &match_positions {
                    if let Some(extended) = replace_compound_with_ext(complex, extender, match_pos, ext_has_non_descendant, sel_leading, sel_trailing, orig_combinator_0) {
                        for ec in extended {
                            if !results.contains(&ec) { results.push(ec); }
                        }
                    }
                }
            }
        }
        let result = Selector(results);
        tracing::debug!(%result, "extend: result (multi-extendee)");
        return result;
    }

    let is_no_op = unified_extendee.is_none() || selector_has_universal || selector_any_namespace_no_op || is_more_specific_than(selector, extender, extendee);
    if is_no_op {
        tracing::debug!("extend: NO-OP");
        return selector.clone();
    }
    let unified_extendee = unified_extendee.expect("checked");
    let results: Vec<ComplexSelector> = selector.0.iter().flat_map(|complex| {
        let original = std::iter::once(complex.clone());
        let extendee_matches_complex = full_match_only && unified_extendee.compounds.len() != complex.compounds.len();
        let extended = if extendee_matches_complex {
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

fn is_more_specific_than(selector: &Selector, extender: &Selector, extendee: &Selector) -> bool {
    use super::super::selector_is_super::is_super_compound;

    if selector.0.iter().any(|c| extendee_is_is_where_matches_subselector(c, extendee)) {
        return true;
    }

    if is_is_where_matches_subselector(extender, extendee) {
        return true;
    }

    extender.0.iter().all(|ext_c| {
        extendee.0.iter().any(|ee_c| {
            let ee_has_universal = ee_c.compounds.iter().any(|(_, comp)| comp.0.iter().any(|s| matches!(s, SimpleSelector::Universal)));
            if ee_has_universal { return true; }

            let ee_compounds: Vec<_> = ee_c.compounds.iter().map(|(_, c)| c).collect();
            let ext_compounds: Vec<_> = ext_c.compounds.iter().map(|(_, c)| c).collect();

            if ee_compounds.len() <= ext_compounds.len() {
                let offset = ext_compounds.len() - ee_compounds.len();
                ee_compounds.iter().enumerate().all(|(i, ee_comp)| {
                    ext_compounds.get(i + offset).is_some_and(|ext_comp| is_super_compound(ee_comp, ext_comp))
                })
            } else {
                ee_compounds.iter().enumerate().all(|(i, ee_comp)| {
                    ext_compounds.get(i).is_some_and(|ext_comp| is_super_compound(ee_comp, ext_comp))
                })
            }
        })
    })
}

fn is_is_where_matches_subselector(extender: &Selector, extendee: &Selector) -> bool {
    let ee_simple = extendee.0.first().and_then(|c| c.compounds.first()).and_then(|(_, comp)| comp.0.first());
    let Some(ee_simple) = ee_simple else { return false };

    extender.0.iter().any(|ext_c| {
        ext_c.compounds.iter().any(|(_, comp)| {
            comp.0.iter().any(|s| match s {
                SimpleSelector::PseudoClass { name, arg: Some(arg) }
                    if (name == "is" || name == "where" || name == "matches") =>
                {
                    super::extend_pseudo::parse_selector_list_to_simples(arg).iter().any(|sel| {
                        let normalized = super::extend_pseudo::normalize_pseudo_compound(sel);
                        normalized == *ee_simple
                    })
                }
                _ => false,
            })
        })
    })
}

fn extendee_is_is_where_matches_subselector(selector: &ComplexSelector, extendee: &Selector) -> bool {
    if !extendee.0.iter().any(|c| {
        c.compounds.iter().any(|(_, comp)| {
            comp.0.len() == 1 && matches!(&comp.0[0],
                SimpleSelector::PseudoClass { name, .. } if name == "is" || name == "where" || name == "matches"
            )
        })
    }) {
        return false;
    }

    let ee_name = extendee.0.iter()
        .flat_map(|c| c.compounds.iter())
        .find_map(|(_, comp)| {
            comp.0.first().and_then(|s| match s {
                SimpleSelector::PseudoClass { name, .. } if name == "is" || name == "where" || name == "matches" => Some(name.clone()),
                _ => None,
            })
        });

    let Some(ee_name) = ee_name else { return false };

    selector.compounds.iter().any(|(_, comp)| {
        comp.0.len() > 1 && comp.0.iter().any(|s| matches!(s,
            SimpleSelector::PseudoClass { name, .. } if *name == ee_name
        ))
    })
}
