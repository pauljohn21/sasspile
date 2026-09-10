//! —— CSS @import 提升策略 ——
//!
//! 概要：将所有 `@import` AtRule 提升到输出 CSS 的顶部。
//!
//! ## 核心概念
//! - 遍历 CssNode 树收集 `@import` 节点
//! - 按原始顺序移到输出顶部
//! - 从原来位置移除
//! - 递归提取 Rule 内嵌的 @import（Sass 规范要求提升）
//!

use crate::css::node::CssNode;

/// CSS @import 提升——纯函数版（消费 Vec 返回新 Vec）。
///
/// Sass 规范要求 CSS `@import`（`@import "file.css"`）出现在输出顶部，
/// 保持源码中的相对顺序。此函数递归扫描 CSS 树，提取 @import 节点。
///
/// 两阶段算法：
/// 1. 递归提取所有层次的 @import（包括 Rule children 内部）
/// 2. 将提取的 imports 置顶，其余节点保持原序（不含 import）
pub(crate) fn hoist_css_imports(nodes: Vec<CssNode>) -> Vec<CssNode> {
    let span = crate::__tracing::debug_span!("hoist_css_imports", n = nodes.len());
    let _enter = span.enter();
    let (imports, rest) = hoist_recursive(nodes);
    match !imports.is_empty() {
        true => crate::__tracing::debug!(n_imports = imports.len(), "hoisted css imports"),
        false => {}
    }
    let mut result = imports;
    result.extend(rest);
    result
}

/// 递归提取 @import 节点，返回 (imports, 剩余节点)。
fn hoist_recursive(nodes: Vec<CssNode>) -> (Vec<CssNode>, Vec<CssNode>) {
    let mut imports = Vec::new();
    let mut rest = Vec::new();
    for node in nodes {
        match node {
            // AtRule — 检查是否是 @import（无 body）或需要递归
            CssNode::AtRule {
                name,
                params,
                children,
                has_body,
            } => {
                if name == "import" && !has_body {
                    // @import — 提取到顶部
                    imports.push(CssNode::AtRule {
                        name,
                        params,
                        children,
                        has_body: false,
                    });
                } else {
                    // 有 body 的 AtRule — 递归处理 children
                    let (extracted, remaining) = hoist_recursive(children);
                    imports.extend(extracted);
                    rest.push(CssNode::AtRule {
                        name,
                        params,
                        children: remaining,
                        has_body,
                    });
                }
            }
            // AtRoot — 递归处理
            CssNode::AtRoot(kids, q) => {
                let (extracted, remaining) = hoist_recursive(kids);
                imports.extend(extracted);
                rest.push(CssNode::AtRoot(remaining, q));
            }
            // Rule — 递归处理 children，提取嵌套的 @import
            CssNode::Rule {
                selector,
                declarations,
                children,
            } => {
                let (extracted, remaining) = hoist_recursive(children);
                imports.extend(extracted);
                rest.push(CssNode::Rule {
                    selector,
                    declarations,
                    children: remaining,
                });
            }
            other => rest.push(other),
        }
    }
    (imports, rest)
}
