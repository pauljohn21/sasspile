//! 选择器算法实现。
//!
//! - selector-is-superselector
//! - selector-unify
//! - selector-extend

use crate::error::Result;
use crate::eval::selector::{parse_selector_list, Combinator, CompoundSelector, CompoundWithCombinator, ComplexSelector};
use crate::parse::ast::*;

// —— selector-is-superselector ——

/// 判断 $super 是否是 $sub 的 superselector。
/// 即：$super 匹配的所有元素，$sub 是否也能匹配。
pub fn is_superselector(super_str: &str, sub_str: &str) -> Result<Value> {
    let supers = parse_selector_list(super_str)?;
    let subs = parse_selector_list(sub_str)?;

    // $sub 中的每个选择器都必须被 $super 中的某个选择器覆盖
    for sub in &subs {
        let mut covered = false;
        for super_sel in &supers {
            if complex_is_super(super_sel, sub) {
                covered = true;
                break;
            }
        }
        if !covered {
            return Ok(Value::Bool(false));
        }
    }
    Ok(Value::Bool(true))
}

fn complex_is_super(super_sel: &ComplexSelector, sub: &ComplexSelector) -> bool {
    // 简化实现：检查 super 的最后一段 compound 是否覆盖 sub 的最后一段
    // 完整实现需要从右到左匹配
    if super_sel.parts.is_empty() || sub.parts.is_empty() {
        return super_sel.parts.is_empty() && sub.parts.is_empty();
    }

    // 检查 super 的最后 compound 是否是 sub 最后 compound 的超集
    let super_last = &super_sel.parts.last().unwrap().compound;
    let sub_last = &sub.parts.last().unwrap().compound;

    compound_is_super(super_last, sub_last)
}

fn compound_is_super(super_c: &CompoundSelector, sub_c: &CompoundSelector) -> bool {
    // 通配符 * 匹配任何元素
    match &super_c.element {
        Some(elem) if elem == "*" => {} // * 匹配任何元素
        Some(elem) => {
            // 类型选择器：sub 必须有相同类型
            match &sub_c.element {
                Some(sub_elem) if sub_elem != elem => return false,
                None => return false, // super 有类型但 sub 没有
                _ => {}
            }
        }
        None => {
            // super 没有类型选择器，sub 可以有也可以没有
        }
    }

    // super 的 classes 必须是 sub classes 的子集（sub 必须包含 super 的所有 class）
    for class in &super_c.classes {
        if !sub_c.classes.contains(class) {
            return false;
        }
    }

    // super 的 ids 必须是 sub ids 的子集
    for id in &super_c.ids {
        if !sub_c.ids.contains(id) {
            return false;
        }
    }

    // super 的 pseudos 必须是 sub pseudos 的子集（简化）
    for pseudo in &super_c.pseudos {
        if !sub_c.pseudos.contains(pseudo) {
            return false;
        }
    }

    true
}

// —— selector-unify ——

/// 合并两个选择器为一个能同时匹配两者的选择器。
pub fn unify(selector1: &str, selector2: &str) -> Result<Value> {
    let list1 = parse_selector_list(selector1)?;
    let list2 = parse_selector_list(selector2)?;

    let mut result = Vec::new();

    for s1 in &list1 {
        for s2 in &list2 {
            if let Some(unified) = unify_complex(s1, s2) {
                result.push(unified);
            }
        }
    }

    if result.is_empty() {
        Ok(Value::Null)
    } else {
        Ok(Value::String(
            result
                .iter()
                .map(complex_to_string)
                .collect::<Vec<_>>()
                .join(", "),
            false,
        ))
    }
}

fn unify_complex(s1: &ComplexSelector, s2: &ComplexSelector) -> Option<ComplexSelector> {
    // 简化实现：只处理最后一段 compound 可合并的情况
    if s1.parts.is_empty() || s2.parts.is_empty() {
        return None;
    }

    let last1 = &s1.parts.last()?.compound;
    let last2 = &s2.parts.last()?.compound;

    // 检查前缀是否相同
    if s1.parts.len() != s2.parts.len() {
        return None;
    }

    for (p1, p2) in s1.parts[..s1.parts.len() - 1].iter().zip(s2.parts[..s2.parts.len() - 1].iter())
    {
        if p1.combinator != p2.combinator || p1.compound != p2.compound {
            return None;
        }
    }

    // 合并最后一段
    let merged = unify_compound(last1, last2)?;

    let mut parts = s1.parts[..s1.parts.len() - 1].to_vec();
    parts.push(CompoundWithCombinator {
        compound: merged,
        combinator: s1.parts.last()?.combinator.clone(),
    });

    Some(ComplexSelector { parts })
}

fn unify_compound(c1: &CompoundSelector, c2: &CompoundSelector) -> Option<CompoundSelector> {
    // 命名空间兼容检查：两个 Some 且不同 → 冲突
    let namespace = match (&c1.namespace, &c2.namespace) {
        (Some(n1), Some(n2)) if n1 != n2 => return None, // 命名空间冲突
        (Some(n), None) | (None, Some(n)) => Some(n.clone()),
        _ => None,
    };

    // 元素选择器兼容检查
    let element = match (&c1.element, &c2.element) {
        (Some(e1), Some(e2)) => {
            if e1 == e2 {
                Some(e1.clone())
            } else if e1 == "*" {
                Some(e2.clone())
            } else if e2 == "*" {
                Some(e1.clone())
            } else {
                return None; // 冲突
            }
        }
        (Some(e), None) | (None, Some(e)) => Some(e.clone()),
        (None, None) => None,
    };

    // 合并 classes（取并集）
    let mut classes = c1.classes.clone();
    for c in &c2.classes {
        if !classes.contains(c) {
            classes.push(c.clone());
        }
    }

    // 合并 ids（取并集）
    let mut ids = c1.ids.clone();
    for id in &c2.ids {
        if !ids.contains(id) {
            ids.push(id.clone());
        }
    }

    // 合并 attrs（取并集）
    let mut attrs = c1.attrs.clone();
    for a in &c2.attrs {
        if !attrs.contains(a) {
            attrs.push(a.clone());
        }
    }

    // 合并 pseudos（取并集）
    let mut pseudos = c1.pseudos.clone();
    for p in &c2.pseudos {
        if !pseudos.contains(p) {
            pseudos.push(p.clone());
        }
    }

    Some(CompoundSelector {
        element,
        namespace,
        classes,
        ids,
        attrs,
        pseudos,
    })
}

// —— selector-extend ——

/// 在选择器列表中将匹配 $target 的部分替换为 $extender。
pub fn extend(selector: &str, target: &str, extender: &str) -> Result<Value> {
    let selector_list = parse_selector_list(selector)?;
    let target_list = parse_selector_list(target)?;
    let extender_list = parse_selector_list(extender)?;

    let mut result = Vec::new();

    for sel in &selector_list {
        // 检查是否匹配 target，如果匹配则替换
        let mut matched = false;
        for t in &target_list {
            if let Some(replaced) = extend_complex(sel, t, &extender_list) {
                matched = true;
                result.extend(replaced);
            }
        }
        if !matched {
            result.push(sel.clone());
        }
    }

    if result.is_empty() {
        Ok(Value::Null)
    } else {
        Ok(Value::String(
            result
                .iter()
                .map(complex_to_string)
                .collect::<Vec<_>>()
                .join(", "),
            false,
        ))
    }
}

/// 尝试将 sel 中匹配 target 的部分替换为 extender。
/// 返回 Some(替换后的选择器列表) 如果匹配，否则返回 None。
fn extend_complex(
    sel: &ComplexSelector,
    target: &ComplexSelector,
    extender_list: &[ComplexSelector],
) -> Option<Vec<ComplexSelector>> {
    if target.parts.len() > sel.parts.len() {
        return None;
    }

    // 尝试所有可能的起始位置
    for offset in 0..=sel.parts.len() - target.parts.len() {
        // 检查 target 是否匹配 sel 中从 offset 开始的部分
        let mut matched = true;
        for (i, target_part) in target.parts.iter().enumerate() {
            let sel_part = &sel.parts[offset + i];
            // 对于 target 的第一个 compound（i=0）：
            // - 如果 offset > 0，不检查 combinator
            // - 如果 offset == 0，检查 combinator
            if (i > 0 || offset == 0) && sel_part.combinator != target_part.combinator {
                matched = false;
                break;
            }
            if !compound_matches(&sel_part.compound, &target_part.compound) {
                matched = false;
                break;
            }
        }

        if matched {
            // 检查命名空间冲突：extender 不能给已有命名空间的 compound 引入不同命名空间
            let ns_conflict = extender_list[0].parts.iter().enumerate().any(|(i, ext_part)| {
                let sel_idx = offset + i;
                if sel_idx >= sel.parts.len() {
                    return false;
                }
                match (&sel.parts[sel_idx].compound.namespace, &ext_part.compound.namespace) {
                    (Some(sel_ns), Some(ext_ns)) => sel_ns != ext_ns,
                    _ => false,
                }
            });
            if ns_conflict {
                continue; // 尝试下一个 offset，如果都不行最终返回 None
            }

            // 构建结果：前缀 + extender + 后缀
            let mut result = Vec::new();
            for ext in extender_list {
                let mut parts = Vec::new();
                // 添加前缀
                parts.extend_from_slice(&sel.parts[..offset]);
                // 添加 extender 的 parts
                for (i, ext_part) in ext.parts.iter().enumerate() {
                    let combinator = if i == 0 && offset > 0 {
                        // extender 的第一个 part 使用 sel 中 offset 位置的 combinator
                        sel.parts[offset].combinator.clone()
                    } else if i > 0 {
                        ext_part.combinator.clone()
                    } else {
                        None
                    };
                    parts.push(CompoundWithCombinator {
                        compound: ext_part.compound.clone(),
                        combinator,
                    });
                }
                // 添加后缀
                parts.extend_from_slice(&sel.parts[offset + target.parts.len()..]);
                result.push(ComplexSelector { parts });
            }
            return Some(result);
        }
    }

    None
}

#[allow(dead_code)]
fn complex_matches_target(sel: &ComplexSelector, target: &ComplexSelector) -> bool {
    // 简化：检查 sel 是否包含 target 的所有部分
    if target.parts.len() > sel.parts.len() {
        return false;
    }

    // 检查 target 是否是 sel 的后缀
    let offset = sel.parts.len() - target.parts.len();
    for (i, target_part) in target.parts.iter().enumerate() {
        let sel_part = &sel.parts[offset + i];
        if sel_part.combinator != target_part.combinator
            || !compound_matches(&sel_part.compound, &target_part.compound)
        {
            return false;
        }
    }
    true
}

fn compound_matches(sel: &CompoundSelector, target: &CompoundSelector) -> bool {
    // 命名空间兼容：target 有命名空间则必须匹配 sel 的命名空间
    match (&target.namespace, &sel.namespace) {
        (Some(t), Some(s)) if t != s => return false,
        (Some(_), None) => return false,
        _ => {}
    }

    // sel 必须包含 target 的所有选择器组件
    match (&target.element, &sel.element) {
        (Some(t), Some(s)) if t != s => return false,
        (Some(_), None) => return false,
        _ => {}
    }

    for class in &target.classes {
        if !sel.classes.contains(class) {
            return false;
        }
    }

    for id in &target.ids {
        if !sel.ids.contains(id) {
            return false;
        }
    }

    for pseudo in &target.pseudos {
        if !sel.pseudos.contains(pseudo) {
            return false;
        }
    }

    true
}

// —— 辅助函数：ComplexSelector → String ——

fn complex_to_string(c: &ComplexSelector) -> String {
    let mut result = String::new();
    for (i, part) in c.parts.iter().enumerate() {
        if let Some(comb) = &part.combinator {
            match comb {
                Combinator::Descendant => result.push(' '),
                Combinator::Child => result.push_str(" > "),
                Combinator::Adjacent => result.push_str(" + "),
                Combinator::Sibling => result.push_str(" ~ "),
            }
        } else if i > 0 {
            result.push(' ');
        }
        result.push_str(&compound_to_string(&part.compound));
    }
    result
}

fn compound_to_string(c: &CompoundSelector) -> String {
    let mut result = String::new();

    // 命名空间前缀
    if let Some(ns) = &c.namespace {
        result.push_str(ns);
        result.push('|');
    }

    if let Some(elem) = &c.element {
        result.push_str(elem);
    }

    for class in &c.classes {
        result.push('.');
        result.push_str(class);
    }

    for id in &c.ids {
        result.push('#');
        result.push_str(id);
    }

    for attr in &c.attrs {
        result.push('[');
        result.push_str(&attr.name);
        if let Some(op) = &attr.op {
            result.push_str(op);
            if let Some(val) = &attr.value {
                result.push_str(val);
            }
        }
        result.push(']');
    }

    for pseudo in &c.pseudos {
        result.push(':');
        if !pseudo.is_class {
            result.push(':');
        }
        result.push_str(&pseudo.name);
        if let Some(arg) = &pseudo.argument {
            result.push('(');
            result.push_str(arg);
            result.push(')');
        }
    }

    result
}
