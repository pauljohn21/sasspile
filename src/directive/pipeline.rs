//! 统一响应式管线 — Flux 思维 + rxrust 算子组合
//!
//! Flux 模型:  groupBy(classify) → flatMap(各 group 独立 scanWith) → merge
//! rxrust 等价: scan_map(accumulate_block) → flat_map(process_block) → collect
//!
//! 每个指令类型不是手写 handler,而是 rxrust 算子链的一个 sub-flow

use crate::css::{CssBuilder, CssNode, render_node};
use rxrust::prelude::*;
use std::convert::Infallible;
use tracing::info_span;

use super::blocks::{accumulate_block, expand_block, BlockAccumulator, DirectiveBlock};
use super::state::CompileState;

// ─── @media 合并 (函数式 fold, into_iter 零 clone) ──────────────────────────

/// 预处理: 拆分 "} @else" 行为单独行
fn split_else_line(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    // "} @else {"
    if let Some(rest) = trimmed.strip_prefix("}").map(str::trim_start) {
        if rest.starts_with("@else if ") || rest.starts_with("@elseif ") || rest == "@else" || rest.starts_with("@else ") {
            let mut result = vec!["}".to_string()];
            result.push(rest.to_string());
            return result;
        }
    }
    vec![line.to_string()]
}

/// CSS 选择器嵌套展平: .parent { .child { color: red; } } → .parent .child { color: red; }
fn flatten_nested_selectors(nodes: Vec<CssNode>) -> Vec<CssNode> {
    let mut result = Vec::new();
    for node in nodes {
        result.push(flatten_node(node, ""));
    }
    result
}

fn flatten_node(node: CssNode, parent_sel: &str) -> CssNode {
    match node {
        CssNode::Rule { selector, children } => {
            let full_selector = if parent_sel.is_empty() {
                selector
            } else if selector.starts_with('&') {
                format!("{}{}", parent_sel, &selector[1..])
            } else {
                format!("{} {}", parent_sel, selector)
            };
            let new_children: Vec<CssNode> = children
                .into_iter()
                .map(|c| flatten_node(c, &full_selector))
                .collect();
            CssNode::Rule {
                selector: full_selector,
                children: new_children,
            }
        }
        CssNode::AtRule { query, children } => {
            // AtRule 保持子节点嵌套 (不展平)
            CssNode::AtRule { query, children }
        }
        other => other,
    }
}

fn merge_media_nodes(nodes: Vec<CssNode>) -> Vec<CssNode> {
    let (result, _media_idx) = nodes.into_iter().fold(
        (Vec::<CssNode>::new(), std::collections::HashMap::<String, usize>::new()),
        |(mut acc, mut idx), node| {
            let merge_target = match &node {
                CssNode::AtRule { query, .. } if query.starts_with("@media ") => {
                    idx.get(query).copied()
                }
                _ => None,
            };

            if let Some(merge_idx) = merge_target {
                if let Some(CssNode::AtRule { children: existing, .. }) = acc.get_mut(merge_idx) {
                    if let CssNode::AtRule { children: new_children, .. } = node {
                        existing.extend(new_children);
                    }
                }
                (acc, idx)
            } else if let CssNode::AtRule { query, .. } = &node {
                if query.starts_with("@media ") {
                    idx.insert(query.clone(), acc.len());
                }
                acc.push(node);
                (acc, idx)
            } else {
                acc.push(node);
                (acc, idx)
            }
        },
    );
    result
}

// ═══════════════════════════════════════════════════════════════════════════
// 管线入口 — 算子链 (chain = 声明, subscribe = 执行边界)
// ═══════════════════════════════════════════════════════════════════════════

pub fn compile_pipeline(input: &str) -> String {
    let _root = info_span!("compile_pipeline", bytes = input.len()).entered();

    // Shared Subject 入口 (String: Shared 需要 'static + Send)
    // Flux 思维: 这就是 Flux.create() 的 Sinks.Many.asFlux()
    let subject = Shared::subject::<String, Infallible>();
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    // 构建算子链 (声明式, 此时不执行)
    subject
        .clone()
        // ══════════════════════════════════════════════════════════════════
        // Phase 1: 指令分块 + 展开 (scan_map + flat_map)
        // ══════════════════════════════════════════════════════════════════
        // scan_map = Flux.accumulate / Flux.bufferUntil
        //   累积行 → emit DirectiveBlock (多行) or Lines (单行)
        .scan_map(BlockAccumulator::default(), accumulate_block)
        // flat_map = Flux.groupBy + flatMap
        .flat_map(|v: Vec<DirectiveBlock>| Shared::from_iter(v))
        // 独立 scan_map: 各 block 展开 (共享 CompileState &mut 就地修改)
        .scan_map(CompileState::new(), expand_block)
        .flat_map(|v: Vec<String>| Shared::from_iter(v))
        // ══════════════════════════════════════════════════════════════════
        // Phase 2: CSS AST 构建 (scan_map CssBuilder)
        // ══════════════════════════════════════════════════════════════════
        .scan_map(CssBuilder::new(), |builder: &mut CssBuilder, line: String| {
            builder.feed(&line)
        })
        .flat_map(|v: Vec<CssNode>| Shared::from_iter(v))
        // 汇聚
        .collect::<Vec<CssNode>>()
        .last()
        // CSS 选择器嵌套展平
        .map(flatten_nested_selectors)
        // @media 合并
        .map(merge_media_nodes)
        .flat_map(|nodes| Shared::from_iter(nodes))
        // ══════════════════════════════════════════════════════════════════
        // Phase 3: 渲染 (&借用 → String, 零 clone)
        // ══════════════════════════════════════════════════════════════════
        .map(|node: CssNode| render_node(&node))
        .collect::<Vec<String>>()
        .last()
        // ══════════════════════════════════════════════════════════════════
        // 终端: subscribe = 执行边界, move 转移终态
        // ══════════════════════════════════════════════════════════════════
        .subscribe(move |css_vec: Vec<String>| {
            let _ = tx.send(css_vec.join("\n"));
        });

    // 驱动: 预处理 split } @else → 2 lines → push all → flush → complete
    let preprocessed: Vec<String> = input
        .lines()
        .flat_map(split_else_line)
        .collect();
    for line in &preprocessed {
        subject.clone().next(line.to_string());
    }
    // sentinel flush: 空行触发 pending_branches 回写
    subject.clone().next(String::new());
    subject.clone().complete();

    rx.recv().unwrap_or_default()
}
