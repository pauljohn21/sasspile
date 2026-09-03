//! CSS @import 提升策略——将所有 `@import` `AtRule` 提升到输出顶部。

use crate::css::node::CssNode;

/// CSS @import 提升——纯函数版（消费 Vec 返回新 Vec）。
///
/// Sass 规范要求 CSS `@import`（`@import "file.css"`）出现在输出顶部，
/// 保持源码中的相对顺序。此函数递归扫描 CSS 树，提取 @import 节点。
pub(crate) fn hoist_css_imports(nodes: Vec<CssNode>) -> Vec<CssNode> {
    let span = crate::__tracing::debug_span!("hoist_css_imports", n = nodes.len());
    let _enter = span.enter();
    // 先递归处理嵌套节点，再按 @import 分流
    let processed: Vec<CssNode> = nodes
        .into_iter()
        .map(|node| match node {
            CssNode::AtRule {
                name,
                params,
                children,
                has_body: true,
            } => CssNode::AtRule {
                name,
                params,
                children: hoist_css_imports(children),
                has_body: true,
            },
            CssNode::AtRoot(kids, q) => CssNode::AtRoot(hoist_css_imports(kids), q),
            other => other,
        })
        .collect();
    let (imports, rest): (Vec<CssNode>, Vec<CssNode>) = processed.into_iter().partition(
        |node| matches!(node, CssNode::AtRule { name, has_body: false, .. } if name == "import"),
    );
    if !imports.is_empty() {
        crate::__tracing::debug!(n_imports = imports.len(), "hoisted css imports");
    }
    let mut result = imports;
    result.extend(rest);
    result
}
