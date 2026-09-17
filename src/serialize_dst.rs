//! Serialize Stage — CssNode stream → char stream
//!
//! scan_map(Serializer) 累积,每个 CssNode 展开为 char 序列,flat_map 进流.

use crate::ast::CssNode;

/// 序列化器 — scan_map 的 Accumulator + emit char 流
#[derive(Debug, Clone, Default)]
pub struct Serializer;

impl Serializer {
    pub fn new() -> Self {
        Self
    }

    /// 将单个 CssNode 渲染为 owned String,由 flat_map 展开
    pub fn render_one(&self, node: CssNode) -> Vec<char> {
        node.render().chars().collect()
    }
}

/// 便捷函数: CssNode → char Vec（纯函数,用于 flat_map）
pub fn render_node_to_chars(node: CssNode) -> Vec<char> {
    node.render().chars().collect()
}
