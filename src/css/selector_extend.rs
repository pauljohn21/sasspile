//! —— 选择器 extend/replace 算法 ——
//!
//! 概要：实现 sass:selector 模块的 extend 和 replace 算法。
//!
//! ## 核心概念
//! - `extend_selector`：将选择器中匹配 extendee 的部分替换为 extender
//! - `replace_selector`：精确替换选择器中匹配 original 的部分
//! - 前导/尾随组合器检测与处理
//! - 多组合器 extendee 拆分
//! - `compounds_conflict` 检查扩展冲突

use super::selector_ast::{Combinator, ComplexSelector, CompoundSelector, Namespace, Selector, SimpleSelector};
use super::selector_ops::{
    compounds_conflict, extender_has_leading_combinator, extender_has_multiple_combinators,
    extender_has_trailing_combinator, has_leading_combinator, has_multiple_combinators,
    has_trailing_combinator, is_semantic_subset, selector_simple_covers_ext,
};
use super::selector_is_super::is_super_compound;
use super::selector_unify::unify_extendee_list;

// ─── 伪类检测辅助函数 ─────────────────────────────────────────────

/// 检测 compound 是否包含 `:not()` 伪类。
fn has_not_pseudo(compound: &CompoundSelector) -> bool {
    compound.0.iter().any(|s| matches!(s, SimpleSelector::PseudoClass { name, .. } if name == "not"))
}

/// 提取 `:not()` 伪类的参数（如 `.c` 或 `.c, .d`）。
fn extract_not_arg(compound: &CompoundSelector) -> Option<String> {
    compound.0.iter().find_map(|s| match s {
        SimpleSelector::PseudoClass { name, arg } if name == "not" => arg.clone(),
        _ => None,
    })
}

/// 在 compound 上追加新的 `:not()` 伪类。
#[allow(dead_code)]
fn append_not_pseudo(compound: &CompoundSelector, new_not: &SimpleSelector) -> CompoundSelector {
    let mut simples: Vec<SimpleSelector> = compound.0.clone();
    simples.push(new_not.clone());
    CompoundSelector(simples)
}

/// 检查 extender 是否包含任何 `:not()` 伪类。
/// 如果包含，根据 Sass known limitation，扩展是 no-op。
fn extender_contains_not(extender: &Selector) -> bool {
    extender.0.iter().any(|c| {
        c.compounds.iter().any(|(_, comp)| has_not_pseudo(comp))
    })
}

/// 伪类参数匹配：检查两个参数化伪类是否匹配。
///
/// 用于 `:nth-child()`, `:nth-last-child()` 等需要参数匹配的伪类。
/// 对于这些伪类，必须参数完全一致才算匹配。
fn pseudo_args_match(name: &str, arg1: Option<&String>, arg2: Option<&String>) -> bool {
    match name {
        "nth-child" | "nth-last-child" | "nth-of-type" | "nth-last-of-type" => {
            // 这些伪类需要参数完全一致
            arg1 == arg2
        }
        // 其他参数化伪类（如 :not, :is, :where, :matches）走默认匹配
        _ => true,
    }
}

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

    // 如果 extendee 列表无法统一，逐个尝试每个 extendee 元素
    // 此时 replace 整个 matched compound（不保留 remaining simples）
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
                    // 在 multi-extendee 模式下，直接替换整个 compound 为 extender
                    if let Some(extended) = replace_compound_with_ext(complex, extender, match_pos, ext_has_non_descendant, sel_leading, sel_trailing, orig_combinator_0) {
                        for ec in extended {
                            if !results.contains(&ec) {
                                results.push(ec);
                            }
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

fn is_more_specific_than(selector: &Selector, extender: &Selector, extendee: &Selector) -> bool {
    // extender 比 extendee 更具体 → extend 是 no-op
    // 即：extendee 是 extender 的超选择器（extendee 匹配更多元素）
    use crate::css::selector_is_super::is_super_compound;

    // 检查 extendee 是 selector 中 compound 内部的 :is/:where/:matches → no-op
    // （如 .c:is(.d) + :is(.d) 扩展 = no-op，但 :is(.a, .b) + :is(.a, .b) = 正常扩展）
    if selector.0.iter().any(|c| extendee_is_is_where_matches_subselector(c, extendee)) {
        return true;
    }

    // 检查 extendee 是 :is/:where/:matches 的 subselector → no-op
    if is_is_where_matches_subselector(extender, extendee) {
        return true;
    }

    extender.0.iter().all(|ext_c| {
        extendee.0.iter().any(|ee_c| {
            // 如果 extendee 的 compound 包含 Universal，任何 extender 都更具体
            let ee_has_universal = ee_c.compounds.iter().any(|(_, comp)| comp.0.iter().any(|s| matches!(s, SimpleSelector::Universal)));
            if ee_has_universal { return true; }

            // 检查 extendee 是否是 extender 的超选择器
            let ee_compounds: Vec<_> = ee_c.compounds.iter().map(|(_, c)| c).collect();
            let ext_compounds: Vec<_> = ext_c.compounds.iter().map(|(_, c)| c).collect();

            if ee_compounds.len() <= ext_compounds.len() {
                // extender 更长或相等 → 检查 ee 是否是 ext 的后缀
                // "c.d" (ext) vs "c" (ee): ee 应该匹配 ext 的最后一个 compound
                let offset = ext_compounds.len() - ee_compounds.len();
                ee_compounds.iter().enumerate().all(|(i, ee_comp)| {
                    ext_compounds.get(i + offset).is_some_and(|ext_comp| is_super_compound(ee_comp, ext_comp))
                })
            } else {
                // extendee 更长 → 逐位匹配
                ee_compounds.iter().enumerate().all(|(i, ee_comp)| {
                    ext_compounds.get(i).is_some_and(|ext_comp| is_super_compound(ee_comp, ext_comp))
                })
            }
        })
    })
}

/// 检查 extendee 是选择器中 `:is()`, `:where()`, `:matches()` 的 subselector。
///
/// 如果是 subselector，extend 是 no-op 因为扩展不会缩小匹配范围。
fn is_is_where_matches_subselector(extender: &Selector, extendee: &Selector) -> bool {
    // extendee 必须是单个 simple 选择器
    let ee_simple = extendee.0.first().and_then(|c| c.compounds.first()).and_then(|(_, comp)| comp.0.first());
    let ee_simple = match ee_simple {
        Some(s) => s,
        None => return false,
    };

    // 检查 extender 的任何 compound 包含 :is/:where/:matches 且参数包含该 subselector
    extender.0.iter().any(|ext_c| {
        ext_c.compounds.iter().any(|(_, comp)| {
            comp.0.iter().any(|s| match s {
                SimpleSelector::PseudoClass { name, arg: Some(arg) }
                    if (name == "is" || name == "where" || name == "matches") =>
                {
                    // 解析参数为列表，检查是否包含 ee_simple
                    parse_selector_list_to_simples(arg).iter().any(|sel| {
                        let normalized = normalize_pseudo_compound(sel);
                        normalized == *ee_simple
                    })
                }
                _ => false,
            })
        })
    })
}

/// 检查 extendee 是否是 `:is()`, `:where()`, `:matches()` 伪类且 selector compound
/// 中包含该伪类和其他 simples（compound 长度 > 1）。
///
/// 这种情况下扩展是 no-op，因为扩展一个 compound 内部的 `:is()` 没有语义意义。
/// 但当整个 compound 只有一个 `:is()` simple 时（如 `:is(.a, .b)`），扩展应正常进行。
fn extendee_is_is_where_matches_subselector(selector: &ComplexSelector, extendee: &Selector) -> bool {
    // extendee 必须是纯粹的 :is/:where/:matches
    if !extendee.0.iter().any(|c| {
        c.compounds.iter().any(|(_, comp)| {
            comp.0.len() == 1 && matches!(&comp.0[0],
                SimpleSelector::PseudoClass { name, .. } if name == "is" || name == "where" || name == "matches"
            )
        })
    }) {
        return false;
    }

    // 且 selector 中存在某个 compound 包含该 :is/:where/:matches 和其他 simple
    let ee_name = extendee.0.iter()
        .flat_map(|c| c.compounds.iter())
        .find_map(|(_, comp)| {
            comp.0.first().and_then(|s| match s {
                SimpleSelector::PseudoClass { name, .. } if name == "is" || name == "where" || name == "matches" => Some(name.clone()),
                _ => None,
            })
        });

    let ee_name = match ee_name {
        Some(n) => n,
        None => return false,
    };

    selector.compounds.iter().any(|(_, comp)| {
        // compound 长度 > 1 且包含 :is/:where/:matches → 扩展 extendee 是 no-op
        comp.0.len() > 1 && comp.0.iter().any(|s| matches!(s,
            SimpleSelector::PseudoClass { name, .. } if *name == ee_name
        ))
    })
}

/// 将"伪" simple (`:__compound__`, `:__selector__`, `:__complex__`) 还原为普通 simple。
fn normalize_pseudo_compound(s: &SimpleSelector) -> SimpleSelector {
    match s {
        SimpleSelector::PseudoClass { name, arg: Some(arg) }
            if (name == "__compound__" || name == "__selector__" || name == "__complex__") =>
        {
            parse_single_simple_selector(arg).unwrap_or_else(|| s.clone())
        }
        _ => s.clone(),
    }
}

/// 将单个 simple selector 字符串解析为 SimpleSelector（仅支持基础 simple）。
fn parse_single_simple_selector(input: &str) -> Option<SimpleSelector> {
    let input = input.trim();
    if input.starts_with('.') && !input[1..].contains('.') && !input[1..].contains(':') && !input[1..].contains(' ') {
        Some(SimpleSelector::Class(input[1..].to_string()))
    } else if input.starts_with('#') && !input[1..].contains('#') && !input[1..].contains(':') && !input[1..].contains(' ') {
        Some(SimpleSelector::Id(input[1..].to_string()))
    } else if !input.starts_with(&['.', '#', ':', '[', '*']) && !input.contains(' ') {
        Some(SimpleSelector::Type { namespace: Namespace::None, name: input.to_string() })
    } else {
        None
    }
}

fn extend_complex(selector: &ComplexSelector, extendee: &ComplexSelector, extender: &Selector) -> Option<Selector> {
    // :not() 特殊处理：在 match_positions 计算之前检测
    if let Some(result) = try_extend_not_in_complex(selector, extendee, extender) {
        return Some(result);
    }

    // :is()/:where()/:matches() arg 内部扩展：标准匹配失败时检测
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
        extend_complex_at(selector, ext_compound, extender, match_pos, ext_has_non_descendant, sel_leading, sel_trailing)
    }).flatten().fold(Vec::new(), |mut acc, c| { if !acc.contains(&c) { acc.push(c); } acc });

    (!unique_results.is_empty()).then_some(Selector(unique_results))
}

/// 在指定位置执行 extend 操作，返回扩展后的 complex selector 列表。
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
}

/// 尝试对 complex 中的 `:is()`, `:where()`, `:matches()` 伪类进行特殊扩展。
///
/// 当 extendee 匹配某个伪类参数（arg）内部的选择器时，将 extender 追加到该伪类参数中。
/// 例如：`downstream {@extend midstream}` 且 selector 为 `:is(midstream)`，
/// 标准匹配失败（`:is(midstream)` 的 simple 不包含 `midstream` type），
/// 此函数检测到 `midstream` 在 `:is()` arg 内部，生成 `:is(midstream, downstream)`。
fn try_extend_is_where_matches_in_complex(
    selector: &ComplexSelector,
    extendee: &ComplexSelector,
    extender: &Selector,
) -> Option<Selector> {
    if extendee.compounds.len() != 1 {
        return None;
    }
    let ext_compound = &extendee.compounds[0].1;

    // 查找所有包含 :is/:where/:matches 且 extendee 匹配 arg 内部选择器的 compound
    let mut match_positions: Vec<(usize, usize)> = Vec::new();
    for (ci, (_, compound)) in selector.compounds.iter().enumerate() {
        for (si, simple) in compound.0.iter().enumerate() {
            let (name, arg) = match simple {
                SimpleSelector::PseudoClass { name, arg: Some(arg) }
                    if name == "is" || name == "where" || name == "matches" =>
                {
                    (name, arg)
                }
                _ => continue,
            };

            // 解析 arg 为选择器，检查 ext_compound 是否匹配其中某个 compound
            let arg_selector = super::selector_parser::parse_selector(arg);
            let arg_matches = arg_selector.0.iter().any(|arg_complex| {
                arg_complex.compounds.iter().any(|(_, arg_comp)| {
                    super::selector_ops::is_semantic_subset(ext_compound, arg_comp)
                })
            });
            if arg_matches {
                match_positions.push((ci, si));
            }
        }
    }

    if match_positions.is_empty() {
        return None;
    }

    // 对每个匹配位置，创建新 compound：将 extender 追加到伪类参数中
    // 语义：extend into pseudo-arg 是 MODIFY IN-PLACE（替换原 compound 中的伪类 arg），
    // 不是添加新 complex——否则 standalone 的原 selector 会残留（不符合 sass-spec）。
    let mut extended_complexes: Vec<ComplexSelector> = Vec::new();
    for (compound_idx, simple_idx) in &match_positions {
        let (comb, old_compound) = &selector.compounds[*compound_idx];
        let (pseudo_name, pseudo_arg) = match old_compound.0.get(*simple_idx) {
            Some(SimpleSelector::PseudoClass { name, arg: Some(arg) })
                if name == "is" || name == "where" || name == "matches" =>
            {
                (name.clone(), arg.clone())
            }
            _ => continue,
        };

        let extender_str = extender.to_string();
        let new_arg = format!("{pseudo_arg}, {extender_str}");
        let new_simple = SimpleSelector::PseudoClass {
            name: pseudo_name,
            arg: Some(new_arg),
        };

        let mut new_simples: Vec<SimpleSelector> = old_compound.0.clone();
        new_simples[*simple_idx] = new_simple;
        let mut new_compounds = selector.compounds.clone();
        new_compounds[*compound_idx] = (*comb, CompoundSelector(new_simples));
        extended_complexes.push(ComplexSelector { compounds: new_compounds });
    }

    if extended_complexes.is_empty() {
        return None;
    }

    Some(Selector(extended_complexes))
}

/// 尝试对 complex 中的 `:not()` 伪类进行特殊扩展。
///
/// 在标准 match_positions 匹配之前调用，因为 `:not(.c)` 不包含 `.c` simple，
/// 标准匹配会失败。这里检测 selector 的 compound 是否包含 `:not()`，
/// 且 extendee 是否匹配 `:not()` 内部的选择器。
fn try_extend_not_in_complex(selector: &ComplexSelector, extendee: &ComplexSelector, extender: &Selector) -> Option<Selector> {
    if extendee.compounds.len() != 1 { return None; }
    let ext_compound = &extendee.compounds[0].1;
    let sel_leading = has_leading_combinator(selector);
    let sel_trailing = has_trailing_combinator(selector);

    // 检查是否有任何 compound 包含 :not()
    let not_positions: Vec<usize> = selector.compounds.iter().enumerate()
        .filter(|(_, (_, comp))| has_not_pseudo(comp))
        .map(|(i, _)| i).collect();

    if not_positions.is_empty() { return None; }

    // 对包含 :not() 的 compound 执行扩展
    let mut all_results: Vec<ComplexSelector> = Vec::new();
    let mut any_extended = false;

    for &match_pos in &not_positions {
        let (_, sel_compound) = &selector.compounds[match_pos];
        if let Some(extended) = extend_not_pseudo(selector, sel_compound, ext_compound, extender, match_pos, sel_leading, sel_trailing) {
            for ec in extended {
                if !all_results.contains(&ec) {
                    all_results.push(ec);
                }
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

/// `:not()` 特殊扩展语义：追加新的 `:not()` 到 compound。
///
/// 根据 sass-spec：
/// - `extend(":not(.c)", ".c", ".d")` → `:not(.c):not(.d)`
/// - `extend(":not(.c)", ".c", ".d, .e")` → `:not(.c):not(.d):not(.e)`
/// - `extend(":not(.c, .d)", ".c", ".e")` → `:not(.c, .e, .d)`（添加到列表）
fn extend_not_pseudo(
    selector: &ComplexSelector,
    sel_compound: &CompoundSelector,
    ext_compound: &CompoundSelector,
    extender: &Selector,
    match_pos: usize,
    sel_leading: bool,
    _sel_trailing: bool,
) -> Option<Vec<ComplexSelector>> {
    // 提取 :not() 的参数
    let not_arg = extract_not_arg(sel_compound)?;

    // 解析 :not() 内部为选择器，检查 ext_compound 是否匹配
    let inner_selector = super::selector_parser::parse_selector(&not_arg);
    let inner_compound = inner_selector.0.first()?;

    // ext_compound 必须是 :not() 内部选择器的子集（意味着 extendee 匹配 :not() 的内容）
    if !is_semantic_subset(ext_compound, &inner_compound.compounds[0].1) {
        // 检查是否为单 simple 匹配（ext_compound 只有 1 个 simple 且在 inner 中存在）
        if ext_compound.0.len() != 1 { return None; }
        let ext_simple = &ext_compound.0[0];
        if !inner_compound.compounds[0].1 .0.iter().any(|s| s == ext_simple) {
            return None;
        }
    }

    // Sass 已知限制：extender 包含 :not() 时是 no-op
    // 因为会产生嵌套 :not(:not(...))，而 Sass 选择忽略嵌套 :not
    if extender_contains_not(extender) {
        return None;
    }

    // 收集 extender 中的所有 simple 选择器（展开列表/:is/:where/:matches）
    let extender_simples = flatten_extender_simples(extender);

    // 构建新的 compound：在原有 compound 基础上追加新的 :not()
    let prefix = &selector.compounds[..match_pos];
    let suffix = &selector.compounds[match_pos + 1..];
    let orig_combinator = selector.compounds[match_pos].0;
    let resolved_first_combinator = if sel_leading { orig_combinator.or(Some(Combinator::Descendant)) } else { orig_combinator };

    // 构建扩展后的 compound：原有 simples + 新的 :not(extender_simple)
    let extended_compound_with_nots: Vec<SimpleSelector> = sel_compound.0.iter().cloned()
        .chain(extender_simples.iter().map(|s| {
            // 将"伪" simple 转换为正确的 :not() 参数
            let not_arg = match s {
                SimpleSelector::PseudoClass { name, arg: Some(arg) }
                    if name == "__compound__" || name == "__selector__" || name == "__complex__" => arg.clone(),
                _ => s.to_string(),
            };
            SimpleSelector::PseudoClass {
                name: "not".to_string(),
                arg: Some(not_arg),
            }
        }))
        .collect();

    let mut result_compounds: Vec<(Option<Combinator>, CompoundSelector)> = prefix.to_vec();
    result_compounds.push((resolved_first_combinator, CompoundSelector(extended_compound_with_nots)));
    result_compounds.extend(suffix.iter().cloned());

    let complex = ComplexSelector { compounds: result_compounds };
    Some(vec![complex])
}

/// 将 extender 中的选择器展开为 simple selector 列表。
///
/// 处理以下场景：
/// - `.d, .e` → [.d, .e]
/// - `:is(.d, .e)` → [.d, .e]
/// - `:where(.d .e, .f .g)` → [.d .e, .f .g]（每个 compound 转字符串）
/// - `:matches(.d, .e)` → [.d, .e]
/// - `.d:is(.e, .f)` → [.d:is(.e, .f)]（整体保留）
fn flatten_extender_simples(extender: &Selector) -> Vec<SimpleSelector> {
    // 如果 extender 只有一个 complex 且只有一个 compound，直接取该 compound 的 simples
    if extender.0.len() == 1 {
        let complex = &extender.0[0];
        if complex.compounds.len() == 1 {
            let compound = &complex.compounds[0].1;

            // 只在 compound 仅由一个 :is/:where/:matches simple 组成时才展开
            // 如果 compound 包含其他 simples（如 .d:is(.e,.f)），整体保留
            let is_only_is_where_matches = compound.0.len() == 1 && matches!(&compound.0[0],
                SimpleSelector::PseudoClass { name, arg: Some(_) }
                if name == "is" || name == "where" || name == "matches"
            );

            if !is_only_is_where_matches {
                // 普通情况：每个 simple 单独为一项
                // 但如果 compound 有多个 simples（比如 .d:is(.e,.f)），整体作为一个 SimpleSelector 不合适
                // 这里我们返回一个"伪" simple 表示整个 compound
                if compound.0.len() == 1 {
                    return vec![compound.0[0].clone()];
                }
                // compound 有多个 simples 但不包含 :is/:where/:matches
                // 将整个 compound 编码到一个 PseudoClass 中（特殊情况）
                return vec![SimpleSelector::PseudoClass {
                    name: "__compound__".to_string(),
                    arg: Some(compound.to_string()),
                }];
            }

            // 包含 :is/:where/:matches，需要提取其参敂
            if let Some(arg) = compound.0.iter().find_map(|s| match s {
                SimpleSelector::PseudoClass { name, arg } if name == "is" || name == "where" || name == "matches" => arg.clone(),
                _ => None,
            }) {
                // 解析参敂为逗号分隔的选择器列表
                return parse_selector_list_to_simples(&arg);
            }
        }
    }

    // extender 是逗号分隔列表：.d, .e → 展开
    extender.0.iter().flat_map(|complex| {
        if complex.compounds.len() == 1 {
            let compound = &complex.compounds[0].1;
            if compound.0.len() == 1 {
                vec![compound.0[0].clone()]
            } else {
                vec![SimpleSelector::PseudoClass {
                    name: "__compound__".to_string(),
                    arg: Some(compound.to_string()),
                }]
            }
        } else {
            // multi-compound complex selector
            vec![SimpleSelector::PseudoClass {
                name: "__complex__".to_string(),
                arg: Some(complex.to_string()),
            }]
        }
    }).collect()
}

/// 将逗号分隔的选择器列表字符串解析为 SimpleSelector 向量。
fn parse_selector_list_to_simples(input: &str) -> Vec<SimpleSelector> {
    // 按逗号分割，但需要处理括号内的逗号
    let mut result = Vec::new();
    let mut depth = 0usize;
    let mut current = String::new();

    for ch in input.chars() {
        match ch {
            '(' => { depth += 1; current.push(ch); }
            ')' => { depth = depth.saturating_sub(1); current.push(ch); }
            ',' if depth == 0 => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    result.push(SimpleSelector::PseudoClass {
                        name: "__selector__".to_string(),
                        arg: Some(trimmed),
                    });
                }
                current = String::new();
                continue;
            }
            _ => current.push(ch),
        }
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        result.push(SimpleSelector::PseudoClass {
            name: "__selector__".to_string(),
            arg: Some(trimmed),
        });
    }
    result
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
        0 | 1 if ext_complex.compounds.first().is_none_or(|(c, comp)| c.is_some() && comp.0.is_empty()) => {
            vec![vec![(resolved_first_combinator.or(Some(Combinator::Descendant)), CompoundSelector(Vec::new()))]]
        }
        1 => {
            let ext_sims = &ext_complex.compounds[0].1 .0;
            // 合并 remaining 和 extender，保持 CSS 选择器顺序：
            // - 如果 extendee 是 TYPE 选择器，extender 应该放在 remaining 之前
            // - 否则（extendee 是 CLASS 等），extender 放在 remaining 之后
            let ext_is_type = ext_sims.iter().any(|s| matches!(s, SimpleSelector::Type { .. }));
            let merged: Vec<SimpleSelector> = if ext_is_type {
                // TYPE 选择器放在前面
                ext_sims.iter().cloned().chain(remaining.iter().cloned()).collect()
            } else {
                // 其他选择器放在后面
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

/// 在 multi-extendee 模式下，将整个 matched compound 替换为 extender（不保留 remaining）。
fn replace_compound_with_ext(
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

    // 使用空的 remaining 列表（替换整个 compound）
    let remaining: Vec<SimpleSelector> = Vec::new();
    Some(extender.0.iter().flat_map(|ext_complex| {
        build_extended_complex(selector, ext_complex, match_pos, &remaining, sel_leading, sel_trailing)
    }).collect())
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
