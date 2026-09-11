//! —— 选择器简化 ——
//!
//! 概要：@extend 后产生的选择器需经过两阶段简化：
//! 1. Compound 内 simple 去重：`.a.a` → `.a`
//! 2. Selector list 去重：移除被 superselector 覆盖的 complex
//!
//! 仅在序列化前调用，不影响 eval 阶段。

use crate::css::node::CssNode;
use crate::css::selector_ast::{ComplexSelector, CompoundSelector, Selector, SimpleSelector};
use crate::css::selector_parser::parse_selector;
use crate::css::selector_is_super::is_superselector;

/// 主入口：简化 CssNode 列表中的选择器。
pub fn simplify_rules(nodes: &mut Vec<(CssNode, usize)>) {
    let _span = tracing::info_span!("simplify_rules", n_nodes = nodes.len()).entered();
    dedup_compound_simples(nodes);
    eliminate_superselectors(nodes);
}

/// 阶段 1：compound 内 simple 去重（`.a.a` → `.a`）。
fn dedup_compound_simples(nodes: &mut [(CssNode, usize)]) {
    nodes.iter_mut().for_each(|(node, _)| match node {
        CssNode::Rule { selector, .. } => {
            let simplified = simplify_selector_str(selector);
            if simplified != *selector {
                tracing::trace!(original = %selector, simplified = %simplified, "dedup compound");
                *selector = simplified;
            }
        }
        CssNode::AtRule { children, has_body: true, .. } => {
            let mut child_pairs: Vec<(CssNode, usize)> = children.iter().cloned().enumerate().map(|(i, n)| (n, i)).collect();
            dedup_compound_simples(&mut child_pairs);
            children.clone_from(&child_pairs.into_iter().map(|(n, _)| n).collect::<Vec<_>>());
        }
        _ => {}
    });
}

fn simplify_selector_str(selector: &str) -> String {
    let ast = parse_selector(selector);
    if ast.0.is_empty() {
        return selector.to_string();
    }
    let simplified = Selector(
        ast.0.into_iter()
            .map(|complex| ComplexSelector {
                compounds: complex.compounds.into_iter()
                    .map(|(comb, comp)| (comb, dedup_simple_in_compound(comp)))
                    .collect(),
            })
            .collect(),
    );
    simplified.to_string()
}

fn dedup_simple_in_compound(compound: CompoundSelector) -> CompoundSelector {
    let seen: Vec<SimpleSelector> = compound.0.into_iter()
        .fold(Vec::new(), |mut acc, s| {
            if !acc.contains(&s) {
                acc.push(s);
            }
            acc
        });
    CompoundSelector(seen)
}

/// 阶段 2：消除 superselector 冗余。
///
/// 仅比较相同 group_id 且相同 declarations 的规则——这些来自同一 @extend 扩展的输出。
/// 如果 group A 是 group B 的 superselector 且 specificity(A) ≥ specificity(extender)，移除 B。
fn eliminate_superselectors(nodes: &mut Vec<(CssNode, usize)>) {
    // 按 group_id 分组
    let max_group = nodes.iter().map(|(_, g)| *g).max().unwrap_or(0);
    let mut group_buckets: Vec<Vec<usize>> = vec![Vec::new(); max_group + 1];
    nodes.iter().enumerate()
        .filter(|(_, (n, _))| matches!(n, CssNode::Rule { .. }))
        .for_each(|(i, (_, g))| {
            if *g < group_buckets.len() {
                group_buckets[*g].push(i);
            }
        });

    let parsed: Vec<Option<Selector>> = nodes.iter().map(|(n, _)| match n {
        CssNode::Rule { selector, .. } => Some(parse_selector(selector)),
        _ => None,
    }).collect();

    let mut to_remove: Vec<usize> = Vec::new();

    for bucket in &group_buckets {
        for &i in bucket {
            if to_remove.contains(&i) { continue; }
            let sel_i = match &parsed[i] {
                Some(s) if !s.0.is_empty() => s,
                _ => continue,
            };

            for &j in bucket {
                if i == j || to_remove.contains(&j) { continue; }
                let sel_j = match &parsed[j] {
                    Some(s) if !s.0.is_empty() => s,
                    _ => continue,
                };

                if !same_declarations(&nodes[i], &nodes[j]) { continue; }

                // 只在 group 内消除：sel_i 是 sel_j 的 superselector
                if is_superselector(sel_i, sel_j) {
                    tracing::trace!(super_ = %sel_i, sub_ = %sel_j, group = ?nodes[i].1, "eliminate redundant subselector");
                    to_remove.push(j);
                }
            }
        }
    }

    to_remove.sort_unstable();
    to_remove.dedup();
    to_remove.reverse();
    for i in to_remove {
        let should_remove = match nodes.get(i) {
            Some((CssNode::Rule { declarations, .. }, _)) => !declarations.is_empty(),
            _ => false,
        };
        if should_remove {
            nodes.remove(i);
        }
    }
}

fn same_declarations(a: &(CssNode, usize), b: &(CssNode, usize)) -> bool {
    match (&a.0, &b.0) {
        (CssNode::Rule { declarations: da, .. }, CssNode::Rule { declarations: db, .. }) => da == db,
        _ => false,
    }
}
