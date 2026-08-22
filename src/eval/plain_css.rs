//! Plain CSS 处理——@import 提升和纯 CSS 模式。

use crate::css::CssNode;

/// CSS @import 提升策略——将所有无 body 的 @import AtRule 提升到输出顶部。
///
/// Sass 规范要求 CSS `@import`（`@import "file.css"`）出现在输出顶部，
/// 保持源码中的相对顺序。此函数递归扫描 CSS 树，提取 @import 节点。
pub fn hoist_css_imports(nodes: &mut Vec<CssNode>) {
    if nodes.is_empty() {
        return;
    }
    let mut imports = Vec::new();
    let mut rest = Vec::new();
    for mut node in nodes.drain(..) {
        // 先递归处理嵌套节点
        match &mut node {
            CssNode::AtRule { children, has_body: true, .. } => {
                hoist_css_imports(children);
            }
            CssNode::AtRoot(kids) => {
                hoist_css_imports(kids);
            }
            _ => {}
        }
        // 判断是否为 CSS @import（无 body 的 @import AtRule）
        let is_css_import = matches!(
            &node,
            CssNode::AtRule { name, has_body: false, .. } if name == "import"
        );
        if is_css_import {
            imports.push(node);
        } else {
            rest.push(node);
        }
    }
    nodes.extend(imports);
    nodes.extend(rest);
}
