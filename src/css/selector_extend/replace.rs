//! —— 选择器 replace 算法 ——

use super::super::selector_ast::{Combinator, ComplexSelector, CompoundSelector, Selector, SimpleSelector};
use super::super::selector_is_super::is_super_compound;

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
