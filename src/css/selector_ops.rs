//! 选择器代数运算——unify（统一）、is_superselector（超选择器判断）、extend（扩展）。
//!
//! 这些算法基于 AST 结构操作，而非字符串匹配。

use super::selector_ast::{
    Combinator, ComplexSelector, CompoundSelector, Selector, SimpleSelector,
};

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
/// 算法：对 selector 的每个 complex，尝试两种匹配策略：
/// 1. Complex-level suffix matching（原算法）：extendee 作为整个 complex 的后缀匹配
/// 2. Compound-level subset matching（新算法）：extendee 的单个 compound 是某 compound 的子集
#[tracing::instrument(level = "info", fields(extendee = %extendee, extender = %extender))]
pub fn extend_selector(selector: &Selector, extendee: &Selector, extender: &Selector) -> Selector {
    let results: Vec<ComplexSelector> = selector
        .0
        .iter()
        .flat_map(|complex| {
            let original = std::iter::once(complex.clone());
            let extended = extendee
                .0
                .iter()
                .filter_map(move |ec| extend_complex(complex, ec, extender))
                .flat_map(|s| s.0.into_iter());
            original.chain(extended)
        })
        .fold(Vec::new(), |mut acc, c| {
            if !acc.contains(&c) {
                acc.push(c);
            }
            acc
        });

    Selector(results)
}

/// 在单个复杂选择器上执行 extend。
///
/// 尝试两种策略：
/// 1. Suffix matching：extendee 的 compounds 序列作为 selector 的 suffix 精确匹配
/// 2. Compound subset matching：extendee 的单个 compound 是 selector 某 compound 的子集
fn extend_complex(
    selector: &ComplexSelector,
    extendee: &ComplexSelector,
    extender: &Selector,
) -> Option<Selector> {
    // Combinator no_op 检测：当 selector 和 extender 都有 leading combinator
    // 或都有 trailing combinator 时，不应扩展（避免产生无效选择器）
    if has_leading_combinator(selector) && extender_has_leading_combinator(extender) {
        return None;
    }
    if has_trailing_combinator(selector) && extender_has_trailing_combinator(extender) {
        return None;
    }

    // Unification no_op 检测：当 extender 的某个 complex 包含 extendee 的某个 complex
    // 作为子选择器时，不应扩展（避免循环引用和冗余输出）
    let extendee_selector = Selector(vec![extendee.clone()]);
    if extender_contains_extendee(extender, &extendee_selector) {
        return None;
    }

    // 策略 1: Complex-level suffix matching
    try_suffix_match(selector, extendee, extender)
        .or_else(|| {
            // 策略 2: Compound-level subset matching（仅当 extendee 只有单个 compound 时）
            match extendee.compounds.len() {
                1 => try_compound_subset_match(selector, extendee, extender),
                _ => None,
            }
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
        .is_some_and(|(comb, _)| comb.is_some())
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

/// 检查 extender 是否包含 extendee 作为子选择器。
///
/// 当 extender 的某个 complex 包含 extendee 的某个 complex 的所有 compounds 时,
/// 不应扩展(避免循环引用)。
/// 例：extendee="c", extender="c.d" → 包含,不扩展
fn extender_contains_extendee(extender: &Selector, extendee: &Selector) -> bool {
    extender.0.iter().any(|ext_c| {
        extendee.0.iter().any(|ee_c| {
            // 检查 extendee 的所有 compounds 是否都是 extender 对应位置 compounds 的超集
            let ee_compounds: Vec<&CompoundSelector> =
                ee_c.compounds.iter().map(|(_, comp)| comp).collect();
            let ext_compounds: Vec<&CompoundSelector> =
                ext_c.compounds.iter().map(|(_, comp)| comp).collect();
            // extendee 必须完全匹配 extender 的前缀
            ee_compounds.len() <= ext_compounds.len()
                && ee_compounds.iter().zip(ext_compounds.iter()).all(|(ee, ext)| {
                    // extendee compound 必须是 extender compound 的超集(包含所有 simples)
                    ee.0.iter().all(|ee_simple| ext.0.contains(ee_simple))
                })
        })
    })
}

/// 策略 1: Suffix matching — extendee 的 compounds 序列作为 selector 的后缀精确匹配。
fn try_suffix_match(
    selector: &ComplexSelector,
    extendee: &ComplexSelector,
    extender: &Selector,
) -> Option<Selector> {
    let sel_compounds: Vec<&CompoundSelector> = selector.compounds.iter().map(|(_, c)| c).collect();
    let ext_compounds: Vec<&CompoundSelector> = extendee.compounds.iter().map(|(_, c)| c).collect();

    match sel_compounds.len() < ext_compounds.len() {
        true => return None,
        false => {}
    }

    let start = sel_compounds.len() - ext_compounds.len();
    let suffix = &sel_compounds[start..];

    let matches = suffix
        .iter()
        .zip(ext_compounds.iter())
        .all(|(s, e)| is_super_compound(e, s) && is_super_compound(s, e));

    match matches {
        false => return None,
        true => {}
    }

    let prefix = &selector.compounds[..start];

    let results: Vec<ComplexSelector> = extender
        .0
        .iter()
        .map(|ext_complex| {
            let compounds: Vec<(Option<Combinator>, CompoundSelector)> = match (
                !prefix.is_empty(),
                !ext_complex.compounds.is_empty(),
            ) {
                (true, true) => prefix
                    .iter()
                    .cloned()
                    .chain(
                        ext_complex
                            .compounds
                            .iter()
                            .enumerate()
                            .map(|(i, (c, comp))| match i {
                                0 => (c.or(Some(Combinator::Descendant)), comp.clone()),
                                _ => (*c, comp.clone()),
                            }),
                    )
                    .collect(),
                _ => prefix
                    .iter()
                    .cloned()
                    .chain(ext_complex.compounds.iter().cloned())
                    .collect(),
            };
            ComplexSelector { compounds }
        })
        .collect();

    (!results.is_empty()).then_some(Selector(results))
}

/// 策略 2: Compound-level subset matching — extendee 的单个 compound 是 selector 某 compound 的子集。
///
/// 核心思路：extender 的首 compound 替换匹配位置，remaining simples 与 extender 后续 compound 串联。
///
/// 例 1：`.c.x .d` extend `.c` → `.e`（单 compound extender）
/// - selector: `[".c.x", ".d"]`
/// - extendee: `[".c"]`（单个 compound）
/// - 匹配位置：index 0（compound `.c.x`），remaining: `[".x"]`
/// - extender: `[".e"]`
/// - 新 compound 0: `.e`（替换整个 `.c.x`）
/// - 新 compound 1: `.x`（remaining）+ extender 后续无 compound，所以只剩 `.x`
/// - 结果：`.e .x .d`... 不对
///
/// 重新分析 spec 结果 `.c.x .d, .x.e .d`：
/// - `.x.e .d`：`.x` 保留，`.e `.x.f .d`
///
/// 正确理解：extendee 替换 compound 内部的 partial simple，extender 整体作为替换插入。
/// 即：`compound = remaining_simples + extender_compounds`
/// 当 extender 有多个 compound 时，remaining simples 附加到 extender 的**最后一个**compound。
///
/// 例 2：selector.extend(".c.x .d", ".c", ".e .f") → ".c.x .d, .e .x.f .d"
/// - extender: `[".e", ".f"]` 两个 compound
/// - 新 complex: [".e"[None], ".x.f"[Descendant], ".d"[Descendant]]
/// - 结果: `.e .x.f .d` ✓
///
/// **Type selector 不部分匹配规则**：当 extendee 只包含 type selector（如 `c`）且 selector
/// compound 还包含其他 simples（如 `c.d`）时，不扩展。Type selector 必须完全匹配整个 compound。
/// 例：`selector.extend("c.d", "c, .d", ".e")` → `"c.d, .e"`（`"c"` 不部分匹配 `"c.d"`）
fn try_compound_subset_match(
    selector: &ComplexSelector,
    extendee: &ComplexSelector,
    extender: &Selector,
) -> Option<Selector> {
    let ext_compound = &extendee.compounds[0].1;

    // 找到第一个匹配的 compound 位置
    let match_pos = selector
        .compounds
        .iter()
        .position(|(_, sel_compound)| is_subset_compound(ext_compound, sel_compound))?;

    let (_, sel_compound) = &selector.compounds[match_pos];

    // Type selector 不部分匹配：如果 extendee 只包含 type selector 且 selector compound
    // 还包含其他 simples，则不扩展（type selector 必须完全匹配整个 compound）
    let ext_is_single_type = ext_compound.0.len() == 1
        && matches!(&ext_compound.0[0], SimpleSelector::Type(_));
    let sel_has_multiple_simples = sel_compound.0.len() > 1;
    if ext_is_single_type && sel_has_multiple_simples {
        return None;
    }

    // 计算 parent context（匹配位置之前的 compounds，包括它们的 combinator）
    let prefix: Vec<(Option<Combinator>, CompoundSelector)> =
        selector.compounds[..match_pos].iter().cloned().collect();

    // 计算后缀（匹配位置之后的 compounds）
    let suffix: Vec<(Option<Combinator>, CompoundSelector)> = selector.compounds[match_pos + 1..]
        .iter()
        .cloned()
        .collect();

    // 计算匹配 compound 中除去 extendee simples 后的剩余部分
    let remaining: Vec<SimpleSelector> = sel_compound
        .0
        .iter()
        .filter(|s| !ext_compound.0.contains(s))
        .cloned()
        .collect();

    // 对 extender 的每个 complex 生成替换
    let results: Vec<ComplexSelector> = extender
        .0
        .iter()
        .map(|ext_complex| {
            let mut new_compounds: Vec<(Option<Combinator>, CompoundSelector)> = Vec::new();

            // 添加 prefix
            new_compounds.extend(prefix.iter().cloned());

            // extender 的 compounds 处理
            // 关键规则：remaining simples 始终在 extender 首 compound 之前
            // 单 compound: remaining + extender_compound
            // 多 compound: extender 首 compound 不变, remaining 放在最后一个 compound 前面
            // 确定首 compound 的 combinator 优先级：
            // 1. extender 自带的 combinator（如 `+ .d` 中的 `+`）
            // 2. selector 原始 compound 的 combinator（如 `> .c.x` 中的 `>`）
            // 3. 仅当 prefix 非空时使用 Descendant（避免产生无效的前导组合器）
            let ext_first_combinator = ext_complex.compounds[0].0;
            let orig_combinator = selector.compounds[match_pos].0;
            let resolved_combinator = ext_first_combinator
                .or(orig_combinator)
                .or_else(|| (!prefix.is_empty()).then_some(Combinator::Descendant));

            match ext_complex.compounds.len() {
                0 => {}
                1 => {
                    // 单 compound extender：remaining 在 extender compound 前面
                    let ext_sims = &ext_complex.compounds[0].1 .0;
                    let merged_simples: Vec<SimpleSelector> = remaining
                        .iter()
                        .cloned()
                        .chain(ext_sims.iter().cloned())
                        .collect();
                    new_compounds.push((resolved_combinator, CompoundSelector(merged_simples)));
                }
                _ => {
                    // 多 compound extender：
                    // - 首 compound 保持不变
                    // - remaining simples 放在最后一个 compound 前面
                    let last_idx = ext_complex.compounds.len() - 1;

                    // 首 compound
                    let (_, first_comp) = &ext_complex.compounds[0];
                    new_compounds.push((resolved_combinator, first_comp.clone()));

                    // 中间 compounds（如果有）
                    for (c, comp) in &ext_complex.compounds[1..last_idx] {
                        new_compounds.push((*c, comp.clone()));
                    }

                    // 最后一个 compound：remaining 在前面
                    let (_, last_comp) = &ext_complex.compounds[last_idx];
                    let merged_simples: Vec<SimpleSelector> = remaining
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

            ComplexSelector { compounds: new_compounds }
        })
        .collect();

    (!results.is_empty()).then_some(Selector(results))
}

/// 检查 `subset` 是否是 `superset` 的 simple selector 子集。
fn is_subset_compound(subset: &CompoundSelector, superset: &CompoundSelector) -> bool {
    subset.0.iter().all(|s| superset.0.contains(s))
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
