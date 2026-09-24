//! 统一响应式管线 — rxrust 1.0.0-rc.5 (Shared 多线程上下文)
//!
//! 四大支柱在 Shared 约束下的真实实现:
//!   1. 借引用: scan_map 闭包内 &mut Acc 就地修改, render_node 借用 &CssNode
//!   2. 消费自身: CompileState / CssBuilder 作为 scan_map 的 &mut Acc 零 clone
//!   3. 响应式思想: chain 是声明, subscribe 是执行边界
//!   4. 终点收集: subscribe 闭包内 Vec<String> → join("\n") 产生唯一 owned String
//!
//! 多线程流程 (基于 rxrust 源码):
//!   Subject<Arc<Mutex<Subscribers>>> → next() 加锁广播 → ScanMapObserver(&mut State)
//!   → flat_map(MergeAll) → 子流 FromIter 同步迭代 → collect → last → subscribe terminal
//!
//! Shared 需要 'static → 入口 String 不可避免, 但中间步骤零额外 clone

use crate::css::{CssBuilder, CssNode, render_node};
use rxrust::prelude::*;
use std::convert::Infallible;
use tracing::info_span;

use super::eval::dispatch_pass;
use super::state::CompileState;

// ─── @media 合并 ─────────────────────────────────────────────────────────
/// 合并相同 query 的 @media 节点 — 函数式 fold 实现
/// 输入: 扁平的 CssNode 序列
/// 输出: 合并重复 @media query 的 CssNode 序列
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
                // 合并到已存在的 @media 节点
                if let (CssNode::AtRule { children: new_children, .. },
                         Some(CssNode::AtRule { children: existing, .. })) =
                    (&node, acc.get_mut(merge_idx))
                {
                    existing.extend(new_children.iter().cloned());
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
// 管线入口
// ═══════════════════════════════════════════════════════════════════════════

/// 编译 SCSS 源码为 CSS (Shared 多线程管线)
pub fn compile_pipeline(input: &str) -> String {
    let _root = info_span!("compile_pipeline", bytes = input.len()).entered();

    // Shared Subject 入口 (String 因为 Shared 需要 'static + Send)
    let subject = Shared::subject::<String, Infallible>();

    // std::mpsc channel — 单次值转移, 唯一终态产物
    // 不需要 tokio runtime, 因为管线流程完全同步:
    // subscribe(声明) → push all lines → complete → terminal 已执行完毕
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    // 构建反应式管线链 (chain = 声明, 此时不执行)
    // 所有算子 wrap 进 Context, subscribe 时 transform 成 boxed observer 加入 Subject 的 Arc<Mutex<Subscribers>>
    subject.clone()
        // Phase 1: CompileState 消费自身 (&mut 就地修改)
        .scan_map(CompileState::new(), dispatch_pass)
        // flat_map: Vec<String> → 逐个 String (MergeAll 订阅子流 Shared::from_iter)
        .flat_map(|v: Vec<String>| Shared::from_iter(v))
        // Phase 2: CssBuilder 消费自身 (&mut 就地修改)
        .scan_map(CssBuilder::new(), |builder: &mut CssBuilder, line: String| -> Vec<CssNode> {
            builder.feed(&line)
        })
        // flat_map: Vec<CssNode> → 逐个 CssNode
        .flat_map(|v: Vec<CssNode>| Shared::from_iter(v))
        // 🔑 支柱 4 第一步: 收集所有 CssNode
        .collect::<Vec<CssNode>>()
        .last()
        // @media 合并 (汇聚后统一合并)
        .map(|nodes| merge_media_nodes(nodes))
        // 展平 CssNode 为逐个 node
        .flat_map(|nodes| Shared::from_iter(nodes))
        // Phase 3: &CssNode → String (首次产生 owned String)
        .map(|node: CssNode| render_node(&node))
        // 🔑 支柱 4 第二步: 收集所有 String
        .collect::<Vec<String>>()
        .last()
        // subscribe = 执行边界, 从这里开始:
        // 1) 终态 observer 被 boxed 加入 Subject 的 Arc<Mutex<Subscribers>> 列表
        // 2) 返回 Subscription handle (必须保持存活到 complete 之后)
        .subscribe(move |css_vec: Vec<String>| {
            let _ = tx.send(css_vec.join("\n"));
        });

    // 注入所有行: 每次 next() → Arc<Mutex> lock → broadcast_value → 同步传播整条链
    input.lines().for_each(|line| subject.clone().next(line.to_string()));
    // complete() → lock → broadcast_complete → 链上每个 observer 的 complete(self) (move 消费)
    // collect 在此时 emit 累积结果, terminal observer 执行, tx.send() 被调用
    subject.clone().complete();

    // 终态产物已通过 channel 传递 (同步流程, terminal 已执行完毕)
    rx.recv().unwrap_or_default()
}
