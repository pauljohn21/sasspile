//! CSS 序列化器——CssNode 树 → CSS 字符串。

pub mod node;
mod serialize;
pub mod selector_ast;
mod selector;
pub mod selector_ops;
pub mod selector_parser;

pub use node::CssNode;

use crate::OutputStyle;

/// 序列化器。
pub struct Serializer;

impl Serializer {
    /// 序列化 `CssNode` 列表为 CSS 字符串。
    pub fn serialize(nodes: &[CssNode], style: OutputStyle) -> String {
        let flattened = Self::flatten_nodes(nodes, 0);
        crate::__tracing::debug!(count = flattened.len(), items = ?flattened.iter().map(|(n, g)| (n.to_string(), *g)).collect::<Vec<_>>(), "flatten result");
        let merged = Self::merge_at_rules(flattened);
        let css = match style {
            OutputStyle::Expanded => Self::serialize_expanded(&merged, 0),
            OutputStyle::Compressed => Self::serialize_compressed(&merged),
        };
        // 当输出包含非 ASCII 字符时，SCSS 规范要求 expanded 模式下添加 @charset 前缀
        match css.is_ascii() {
            true => css,
            false => match style {
                OutputStyle::Expanded => format!("@charset \"UTF-8\";\n{css}"),
                OutputStyle::Compressed => format!("@charset\"UTF-8\";{css}"),
            },
        }
    }
}
