//! 统一响应式管线 — rxrust 1.0.0-rc.5
//!
//! 管线 = 算子链, 每个算子消费上游、产出下游:
//!   from_stream → scan_map → flat_map → scan_map → flat_map → collect → last → subscribe

use crate::css::{CssBuilder, CssNode, render_node};
use rxrust::prelude::*;
use tracing::info_span;

use super::eval::dispatch_pass;
use super::state::CompileState;

// ─── @media 合并 ─────────────────────────────────────────────────────────
fn merge_media_nodes(nodes: Vec<CssNode>) -> Vec<CssNode> {
    let mut result: Vec<CssNode> = Vec::new();
    let mut media_idx: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for node in nodes {
        let merge_idx = match &node {
            CssNode::AtRule { query, .. } if query.starts_with("@media ") => {
                media_idx.get(query).copied()
            }
            _ => None
        };

        if let Some(idx) = merge_idx {
            let new_children = match &node {
                CssNode::AtRule { children, .. } => children.iter().cloned().collect::<Vec<_>>(),
                _ => unreachable!(),
            };
            if let Some(at_rule) = result.get_mut(idx) {
                if let CssNode::AtRule { children: existing, .. } = at_rule {
                    existing.extend(new_children);
                }
            }
            continue;
        }

        if let CssNode::AtRule { query, .. } = &node {
            if query.starts_with("@media ") {
                media_idx.insert(query.clone(), result.len());
            }
        }
        result.push(node);
    }
    result
}

// ═══════════════════════════════════════════════════════════════════════════
// 管线入口
// ═══════════════════════════════════════════════════════════════════════════

pub fn compile_pipeline(input: &str) -> String {
    let _root = info_span!("compile_pipeline", bytes = input.len()).entered();

    if tokio::runtime::Handle::try_current().is_ok() {
        std::thread::scope(|s| {
            s.spawn(|| compile_pipeline_inner(input)).join().unwrap()
        })
    } else {
        compile_pipeline_inner(input)
    }
}

fn compile_pipeline_inner(input: &str) -> String {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");
    let _enter_guard = rt.enter();

    let lines: Vec<String> = input.lines().map(|l| l.to_string()).collect();
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let mut tx_opt = Some(tx);

    let subject = Shared::subject::<String, std::convert::Infallible>();
    let mut emitter = subject.clone();

    let _subscription = subject.clone()
        .scan_map(CompileState::new(), dispatch_pass)
        .flat_map(|v| Shared::from_iter(v))
        .scan_map(CssBuilder::new(), |builder, line: String| builder.feed(&line))
        .flat_map(|v| Shared::from_iter(v))
        .collect::<Vec<CssNode>>()
        .last()
        .map(|nodes| merge_media_nodes(nodes))
        .flat_map(|nodes| Shared::from_iter(nodes))
        .map(|node: CssNode| render_node(&node))
        .collect::<Vec<String>>()
        .last()
        .subscribe(move |css_vec: Vec<String>| {
            if let Some(tx) = tx_opt.take() {
                let _ = tx.send(css_vec.join("\n"));
            }
        });

    for line in lines {
        emitter.clone().next(line);
    }
    emitter.complete();

    rt.block_on(rx).unwrap_or_default()
}
