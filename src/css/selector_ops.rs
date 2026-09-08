//! 选择器代数运算——unify（统一）、is_superselector（超选择器判断）、extend（扩展）。
//!
//! 这些算法基于 AST 结构操作，而非字符串匹配。

use super::selector_ast::{
    Combinator, ComplexSelector, CompoundSelector, Selector, SimpleSelector,
};

// ─── 辅助函数 ──────────────────────────────────────────────────

/// 检查 complex selector 是否有多个连续组合器（中间、前导或尾随）。
/// 多个组合器的特征：存在两个连续 compound 都有 combinator，且第一个是空的。
/// 例如 `.c ~ ~ .d` 解析为 `[(None,.c),(Sibling,""),(Sibling,.d)]`，
/// 其中 `(Sibling,"")` 和 `(Sibling,.d)` 是连续的组合器。
/// 正常选择器如 `.c .d`、`> .c`、`.c +`（单个尾随组合器）没有多个连续组合器。
fn has_multiple_combinators(complex: &ComplexSelector) -> bool {
    complex
        .compounds
        .windows(2)
        .any(|w| {
            let (comb1, comp1) = &w[0];
            let (comb2, _) = &w[1];
            // 连续组合器 = 第一个 compound 有 combinator 且为空，第二个也有 combinator
            comb1.is_some() && comp1.0.is_empty() && comb2.is_some()
        })
}

/// 检查 extender Selector 是否有多个连续组合器。
fn extender_has_multiple_combinators(extender: &Selector) -> bool {
    extender
        .0
        .iter()
        .any(|c| has_multiple_combinators(c))
}

/// 检查 subset 是否是 superset 的 simple selector 子集。
fn is_subset_compound(subset: &CompoundSelector, superset: &CompoundSelector) -> bool {
    subset.0.iter().all(|s| superset.0.contains(s))
}

// ─── unify 算法 ──────────────────────────────────────────────────

/// 统一两个选择器列表（笛卡尔积）。
///
/// `.a` + `.b` → `.a.b`
/// `div` + `span` → None（类型冲突）
#[tracing::instrument(level = "debug", fields(a = %a, b = %b))]
pub fn unify(a: &Selector, b: &Selector) -> Option<Selector> {
    let results: Vec<ComplexSelector> = a
        .0
        .iter()
        .flat_map(|ca| b.0.iter().filter_map(move |cb| unify_complex(ca, cb)))
        .collect();
    (!results.is_empty()).then_some(Selector(results))
}

/// 将extendee列表（Selector可能包含多个complex）统一为单个complex。
/// 用于extendee是列表时，先统一再匹配。
fn unify_extendee_list(extendee: &Selector) -> Option<ComplexSelector> {
    match extendee.0.as_slice() {
        [] => None,
        [single] => Some(single.clone()),
        [first, rest @ ..] => {
            rest.iter().try_fold(first.clone(), |acc, next| {
                unify_complex(&acc, next)
            })
        }
    }
}

/// 统一两个复杂选择器——从右向左逐位置合并复合选择器。
///
/// 算法：
/// 1. 若 a 是 b 的超选择器（或反之），返回更具体的那个。
/// 2. 对齐两 complex 的 compounds 从最右端。
/// 3. 对每对相同位置（从右数）的 compound 调用 `unify_compound`。
/// 4. 较长 selector 的左侧尾部（未对齐部分）保持原样。
#[tracing::instrument(level = "trace", fields(a = %a, b = %b))]
pub fn unify_complex(a: &ComplexSelector, b: &ComplexSelector) -> Option<ComplexSelector> {
    // 若 a 是 b 的超选择器，b 更具体——直接返回 b
    if is_super_complex(a, b) {
        return Some(b.clone());
    }
    // 若 b 是 a 的超选择器，a 更具体——直接返回 a
    if is_super_complex(b, a) {
        return Some(a.clone());
    }

    let a_len = a.compounds.len();
    let b_len = b.compounds.len();
    let min_len = a_len.min(b_len);

    // 从右向左逐位置 unify
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

    // 较长 selector 的左侧尾部（未对齐部分）
    let tail = match a_len.cmp(&b_len) {
        std::cmp::Ordering::Greater => &a.compounds[..a_len - min_len],
        std::cmp::Ordering::Less => &b.compounds[..b_len - min_len],
        std::cmp::Ordering::Equal => &[],
    };

    // 反转恢复从左到右顺序
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
/// - Type 冲突 → None；Id 冲突 → None；PseudoElement 冲突 → None
/// - Universal + Type → Type；Class/PseudoClass/Attribute → 并集去重
#[tracing::instrument(level = "trace", fields(a = %a, b = %b))]
pub fn unify_compound(a: &CompoundSelector, b: &CompoundSelector) -> Option<CompoundSelector> {
    // 冲突检测
    let a_type = a.0.iter().find(|s| matches!(s, SimpleSelector::Type(_)));
    let b_type = b.0.iter().find(|s| matches!(s, SimpleSelector::Type(_)));
    match (a_type, b_type) {
        (Some(SimpleSelector::Type(t1)), Some(SimpleSelector::Type(t2))) if t1 != t2 => {
            return None;
        }
        _ => {}
    }

    let a_id = a.0.iter().find(|s| matches!(s, SimpleSelector::Id(_)));
    let b_id = b.0.iter().find(|s| matches!(s, SimpleSelector::Id(_)));
    match (a_id, b_id) {
        (Some(SimpleSelector::Id(i1)), Some(SimpleSelector::Id(i2))) if i1 != i2 => {
            return None;
        }
        _ => {}
    }

    let a_pe = a
        .0
        .iter()
        .find(|s| matches!(s, SimpleSelector::PseudoElement { .. }));
    let b_pe = b
        .0
        .iter()
        .find(|s| matches!(s, SimpleSelector::PseudoElement { .. }));
    match (a_pe, b_pe) {
        (Some(pa), Some(pb)) if pa != pb => return None,
        _ => {}
    }

    // 合并：优先保留 Type（覆盖 Universal），其余取并集
    let chosen_type = b_type.or(a_type).cloned();
    let chosen_id = b_id.or(a_id).cloned();

    let has_universal = chosen_type.is_none()
        && (a.0.contains(&SimpleSelector::Universal)
            || b.0.contains(&SimpleSelector::Universal));

    // 其余：并集去重
    let rest: Vec<SimpleSelector> = a
        .0
        .iter()
        .chain(b.0.iter())
        .filter(|s| {
            !matches!(
                s,
                SimpleSelector::Type(_) | SimpleSelector::Universal | SimpleSelector::Id(_)
            )
        })
        .cloned()
        .collect();

    let rest: Vec<SimpleSelector> = rest
        .into_iter()
        .fold(Vec::new(), |mut acc, s| {
            if !acc.contains(&s) {
                acc.push(s);
            }
            acc
        });

    // 组装：Type → Universal → Id → rest
    let merged: Vec<SimpleSelector> = chosen_type
        .into_iter()
        .chain(has_universal.then_some(SimpleSelector::Universal))
        .chain(chosen_id.into_iter())
        .chain(rest.into_iter())
        .collect();

    (!merged.is_empty()).then_some(CompoundSelector(merged))
}

// ─── is_superselector 算法 ────────────────────────────────────────

/// 判断 `super_sel` 是否是 `sub_sel` 的超选择器。
#[tracing::instrument(level = "debug", fields(super_ = %super_sel, sub = %sub_sel))]
pub fn is_superselector(super_sel: &Selector, sub_sel: &Selector) -> bool {
    sub_sel
        .0
        .iter()
        .all(|sub_complex| {
            super_sel
                .0
                .iter()
                .any(|super_complex| is_super_complex(super_complex, sub_complex))
        })
}

/// 判断 `super_c` 是否是 `sub_c` 的超复杂选择器。
///
/// super 的复合选择器序列必须是 sub 的子序列。
#[tracing::instrument(level = "trace", fields(super_ = %super_c, sub = %sub_c))]
pub fn is_super_complex(super_c: &ComplexSelector, sub_c: &ComplexSelector) -> bool {
    let super_compounds: Vec<&CompoundSelector> = super_c.compounds.iter().map(|(_, c)| c).collect();
    let sub_compounds: Vec<&CompoundSelector> = sub_c.compounds.iter().map(|(_, c)| c).collect();

    // 子序列匹配：用 try_fold 跟踪 sub 中的匹配位置
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
/// super_c 中的每个简单选择器都出现在 sub_c 中（子集关系）。
#[tracing::instrument(level = "trace", fields(super_ = %super_c, sub = %sub_c))]
pub fn is_super_compound(super_c: &CompoundSelector, sub_c: &CompoundSelector) -> bool {
    // `*` 是任何复合选择器的超选择器
    super_c.0.contains(&SimpleSelector::Universal)
        || super_c.0.iter().all(|super_s| {
            match super_s {
                SimpleSelector::Type(t) => sub_c.0.iter().any(|sub_s| match sub_s {
                    SimpleSelector::Type(st) => st == t,
                    SimpleSelector::Universal => true,
                    _ => false,
                }),
                SimpleSelector::PseudoElement { name, arg } => sub_c.0.iter().any(|sub_s| match sub_s {
                    SimpleSelector::PseudoElement { name: sn, arg: sa } => sn == name && sa == arg,
                    _ => false,
                }),
                _ => sub_c.0.contains(super_s),
            }
        })
}

// ─── extend/replace 算法 ──────────────────────────────────────────

/// 扩展选择器：在 `selector` 中查找匹配 `extendee` 的部分，用 `extender` 追加。
///
/// 算法：对 selector 的每个 complex，先统一 extendee 列表为单个 complex，
/// 然后在任意位置匹配 extendee compound，用 extender 替换。
///
/// NO-OP 检测：若 extender 是 extendee 的子集（即 extendee 是 extender 的超集），
/// 则扩展不产生新选择器，返回原始 selector。
#[tracing::instrument(level = "info", fields(extendee = %extendee, extender = %extender))]
pub fn extend_selector(selector: &Selector, extendee: &Selector, extender: &Selector) -> Selector {
    // 统一 extendee 列表为单个 complex
    let unified_extendee = match unify_extendee_list(extendee) {
        Some(ee) => ee,
        None => return selector.clone(),
    };
    tracing::debug!(unified = %unified_extendee, "extend_selector: unified extendee");

    // NO-OP 检测：extender 是 extendee 的子集 → 扩展不产生新选择器
    if is_subset_selector(extender, extendee) {
        tracing::debug!("extend_selector: extender is subset of extendee, NO-OP");
        return selector.clone();
    }

    let results: Vec<ComplexSelector> = selector
        .0
        .iter()
        .flat_map(|complex| {
            let original = std::iter::once(complex.clone());
            let extended = extend_complex(complex, &unified_extendee, extender)
                .map(|s| s.0.into_iter())
                .unwrap_or_default();
            tracing::debug!(?extended, "extend_selector: extended result");
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

/// 检查 sub 是否是 sup 的选择器子集（sub 匹配的元素都是 sup 匹配的元素）。
fn is_subset_selector(sub: &Selector, sup: &Selector) -> bool {
    sub.0.iter().all(|sub_c| {
        sup.0.iter().any(|sup_c| is_super_complex(sup_c, sub_c))
    })
}

/// 在单个复杂选择器上执行 extend。
///
/// 算法：在任意位置匹配 unified extendee compound，用 extender 替换。
/// 支持多位置匹配：如果 extendee 匹配多个 compound，为每个位置生成扩展。
///
/// 匹配规则：
/// - extendee 的单个 compound 是 selector 某 compound 的子集或精确匹配
/// - 匹配位置可以是首 compound、中间 compound 或尾 compound
/// - 尾 compound 匹配且 prefix 非空时，NO-OP（避免冗余扩展）
///
/// 冲突检测（proper subset 时）：
/// - 若 remaining simples 与 extender 首 compound 存在 Type/Id/PseudoElement 冲突 → NO-OP
///
/// 组合器规则：
/// - 多连续组合器（selector 或 extender）→ NO-OP
/// - 双 leading combinator → NO-OP
/// - 双 trailing combinator → NO-OP
/// - extender leading 优先于 selector leading
/// - extender trailing 优先于 selector trailing
fn extend_complex(
    selector: &ComplexSelector,
    extendee: &ComplexSelector,
    extender: &Selector,
) -> Option<Selector> {
    // 1. 多组合器 NO-OP 检测
    if has_multiple_combinators(selector) || extender_has_multiple_combinators(extender) {
        return None;
    }

    // 2. 双 leading/trailing combinator NO-OP
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

    // 3. 统一 extendee 必须是单 compound（用于 subset matching）
    if extendee.compounds.len() != 1 {
        // 多 compound extendee：仅支持 suffix/whole-complex 精确匹配
        return try_exact_complex_match(selector, extendee, extender);
    }

    let ext_compound = &extendee.compounds[0].1;

    // 4. 找到所有匹配的 compound 位置（subset 或 exact）
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

    // 5. 对每个匹配位置生成扩展，然后去重
    let unique_results: Vec<ComplexSelector> = match_positions
        .iter()
        .filter_map(|&match_pos| {
            let (_, sel_compound) = &selector.compounds[match_pos];
            let prefix = &selector.compounds[..match_pos];
            let suffix = &selector.compounds[match_pos + 1..];
            let is_last = match_pos == selector.compounds.len() - 1;
            let has_prefix = !prefix.is_empty();
            let has_suffix = !suffix.is_empty();

            // 尾 compound 匹配且 prefix 非空 → NO-OP
            if is_last && has_prefix {
                return None;
            }

            // 计算 remaining simples
            let remaining: Vec<SimpleSelector> = sel_compound
                .0
                .iter()
                .filter(|s| !ext_compound.0.contains(s))
                .cloned()
                .collect();

            // 冲突检测（proper subset 时）：remaining 与 extender 首 compound 冲突
            if !remaining.is_empty() {
                if let Some(first_ext_complex) = extender.0.first() {
                    if let Some((_, first_ext_compound)) = first_ext_complex.compounds.first() {
                        if compounds_conflict(&remaining, first_ext_compound) {
                            return None;
                        }
                    }
                }
            }

            // 对 extender 的每个 complex 生成替换
            Some(
                extender
                    .0
                    .iter()
                    .filter_map(|ext_complex| {
                        build_extended_complex(
                            selector,
                            ext_complex,
                            match_pos,
                            &remaining,
                            has_prefix,
                            has_suffix,
                            sel_leading,
                            sel_trailing,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
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
/// 冲突定义：两者都有 Type 但不同，或都有 Id 但不同，或都有 PseudoElement 但不同。
fn compounds_conflict(remaining: &[SimpleSelector], ext_compound: &CompoundSelector) -> bool {
    let rem_type = remaining.iter().find(|s| matches!(s, SimpleSelector::Type(_)));
    let ext_type = ext_compound.0.iter().find(|s| matches!(s, SimpleSelector::Type(_)));
    if let (Some(SimpleSelector::Type(rt)), Some(SimpleSelector::Type(et))) = (rem_type, ext_type) {
        if rt != et {
            return true;
        }
    }

    let rem_id = remaining.iter().find(|s| matches!(s, SimpleSelector::Id(_)));
    let ext_id = ext_compound.0.iter().find(|s| matches!(s, SimpleSelector::Id(_)));
    if let (Some(SimpleSelector::Id(ri)), Some(SimpleSelector::Id(ei))) = (rem_id, ext_id) {
        if ri != ei {
            return true;
        }
    }

    let rem_pe = remaining.iter().find(|s| matches!(s, SimpleSelector::PseudoElement { .. }));
    let ext_pe = ext_compound.0.iter().find(|s| matches!(s, SimpleSelector::PseudoElement { .. }));
    if let (Some(rpe), Some(epe)) = (rem_pe, ext_pe) {
        if rpe != epe {
            return true;
        }
    }

    false
}

/// 构建扩展后的 complex selector。
fn build_extended_complex(
    selector: &ComplexSelector,
    ext_complex: &ComplexSelector,
    match_pos: usize,
    remaining: &[SimpleSelector],
    has_prefix: bool,
    has_suffix: bool,
    sel_leading: bool,
    sel_trailing: bool,
) -> Option<ComplexSelector> {
    let prefix = &selector.compounds[..match_pos];
    let suffix = &selector.compounds[match_pos + 1..];
    let orig_combinator = selector.compounds[match_pos].0;

    let mut new_compounds: Vec<(Option<Combinator>, CompoundSelector)> = Vec::new();

    // 添加 prefix
    new_compounds.extend(prefix.iter().cloned());

    // 确定首 compound 的 combinator
    let ext_first_combinator = ext_complex.compounds.first().and_then(|(c, _)| *c);
    let resolved_first_combinator = if sel_leading {
        // selector 有 leading combinator → 继承
        orig_combinator.or(ext_first_combinator).or(Some(Combinator::Descendant))
    } else {
        ext_first_combinator.or(orig_combinator)
    };

    // 处理 remaining simples：
    // - 匹配首 compound → remaining 附加到 extender 的首个 compound
    // - 匹配中间 compound → remaining 附加到 extender 的最后一个 compound
    // - 匹配尾 compound（无 prefix，即单 compound）→ remaining 附加到 extender 首个 compound
    let remaining_to_attach = if match_pos == 0 || (!has_prefix && !has_suffix) {
        remaining
    } else {
        remaining
    };

    match ext_complex.compounds.len() {
        0 => {
            // extender 无 compound（纯组合器）
            if !remaining_to_attach.is_empty() {
                new_compounds.push((resolved_first_combinator, CompoundSelector(remaining_to_attach.to_vec())));
            }
        }
        1 => {
            // 单 compound extender
            let ext_sims = &ext_complex.compounds[0].1 .0;
            let merged_simples: Vec<SimpleSelector> = if !remaining_to_attach.is_empty() {
                // remaining 附加到 extender compound
                remaining_to_attach.iter().cloned().chain(ext_sims.iter().cloned()).collect()
            } else {
                ext_sims.clone()
            };
            if !merged_simples.is_empty() {
                new_compounds.push((resolved_first_combinator, CompoundSelector(merged_simples)));
            }
        }
        _ => {
            // 多 compound extender
            let last_idx = ext_complex.compounds.len() - 1;

            // 首 compound
            let (_, first_comp) = &ext_complex.compounds[0];
            new_compounds.push((resolved_first_combinator, first_comp.clone()));

            // 中间 compounds（迭代器链替代 for 循环）
            new_compounds.extend(
                ext_complex.compounds[1..last_idx]
                    .iter()
                    .map(|(c, comp)| (*c, comp.clone())),
            );

            // 最后一个 compound：remaining 附加
            let (_, last_comp) = &ext_complex.compounds[last_idx];
            let merged_simples: Vec<SimpleSelector> = remaining_to_attach
                .iter()
                .cloned()
                .chain(last_comp.0.iter().cloned())
                .collect();
            let last_combinator = ext_complex.compounds[last_idx].0;
            new_compounds.push((last_combinator, CompoundSelector(merged_simples)));
        }
    }

    // 添加 suffix
    new_compounds.extend(suffix.iter().cloned());

    // 处理尾随组合器继承
    if sel_trailing && !new_compounds.is_empty() {
        // selector 有尾随组合器，extender 没有 → 继承
        let last_idx = new_compounds.len() - 1;
        new_compounds[last_idx].0 = orig_combinator.or(new_compounds[last_idx].0);
    }

    // 去重：移除空 compound（除非是唯一的）
    let filtered: Vec<_> = new_compounds
        .into_iter()
        .filter(|(comb, comp)| !comp.0.is_empty() || comb.is_some())
        .collect();

    if filtered.is_empty() {
        None
    } else {
        Some(ComplexSelector { compounds: filtered })
    }
}

/// 精确匹配：extendee 的 compounds 序列与 selector 的某个子序列完全匹配。
/// 用于多 compound extendee（如 `c.d` 匹配 `c.d`）。
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

    // 尝试在任意位置匹配 extendee 序列（迭代器链替代 for 循环）
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
/// 尾随组合器指最后一个 compound 是空的（有 combinator 但无 simple selector）。
/// 例如 `.c +` 解析为 `[(None, .c), (Some(Sibling), empty)]`。
/// 正常选择器如 `.c .d` 的最后一个 compound 是 `.d`，没有尾随组合器。
fn has_trailing_combinator(complex: &ComplexSelector) -> bool {
    complex
        .compounds
        .last()
        .is_some_and(|(comb, comp)| comb.is_some() && comp.0.is_empty())
}

/// 检查 extender Selector 的任意 complex 是否有 leading combinator。
fn extender_has_leading_combinator(extender: &Selector) -> bool {
    extender
        .0
        .iter()
        .any(|c| has_leading_combinator(c))
}

/// 检查 extender Selector 的任意 complex 是否有 trailing combinator。
fn extender_has_trailing_combinator(extender: &Selector) -> bool {
    extender
        .0
        .iter()
        .any(|c| has_trailing_combinator(c))
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
            let replaced =
                original
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

    // Strategy 1: Complex-level suffix matching — original 的化合物序列作为 selector 的后缀精确匹配
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

    // Strategy 2: Compound-level subset matching — original 的单个化合物是 selector 某化合物的子集
    // 例: selector-replace('.a.b', '.b', '.c') → '.a.c'
    // original 仅含一个化合物，且其简单选择器是 selector 某化合物的子集
    if orig_compounds.len() == 1 {
        let orig_compound = orig_compounds[0];

        let found = selector.compounds.iter().enumerate().find(|(_, (_, sel_compound))| {
            orig_compound
                .0
                .iter()
                .all(|orig_simple| sel_compound.0.contains(orig_simple))
        });

        if let Some((_idx, (comb, sel_compound))) = found {
            // 新化合物 = (sel_compound 去掉 orig 的 simples) ∪ replacement 第一个化合物的 simples
            if let Some(rep_complex) = replacement.0.first() {
                if let Some((_, rep_compound)) = rep_complex.compounds.first() {
                    let remaining: Vec<SimpleSelector> = sel_compound
                        .0
                        .iter()
                        .filter(|s| !orig_compound.0.contains(s))
                        .cloned()
                        .collect();

                    let new_simples: Vec<SimpleSelector> = remaining
                        .into_iter()
                        .chain(rep_compound.0.clone())
                        .collect();

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
