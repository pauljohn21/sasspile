//! 阶段 4: Evaluated —— CssNode 序列。

use super::serialized::Serialized;
use crate::OutputStyle;
use crate::css::node::CssNode;

/// 求值产物——CssNode 中间表示。
#[derive(Debug, Clone)]
pub struct Evaluated {
    /// CssNode 列表。
    pub nodes: Vec<CssNode>,
}

impl Evaluated {
    /// 序列化——Evaluated → Serialized。
    pub fn serialize(&self, style: OutputStyle) -> Serialized {
        use crate::css::Serializer;

        let css = Serializer::serialize(&self.nodes, style);
        Serialized { css }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::node::CssNode;

    #[test]
    fn test_serialize_empty() {
        let evaluated = Evaluated { nodes: vec![] };
        let serialized = evaluated.serialize(OutputStyle::Expanded);
        assert_eq!(serialized.css, "\n");
    }

    #[test]
    fn test_serialize_single_decl() {
        let evaluated = Evaluated {
            nodes: vec![CssNode::Declaration {
                property: "color".to_string(),
                value: "red".to_string(),
                important: false,
            }],
        };
        let serialized = evaluated.serialize(OutputStyle::Expanded);
        assert_eq!(serialized.css, "color: red;\n");
    }
}
