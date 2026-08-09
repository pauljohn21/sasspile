//! CSS 序列化器——CssNode 树 → CSS 字符串。

pub mod node;

pub use node::CssNode;

use crate::OutputStyle;

/// 序列化器。
pub struct Serializer;

impl Serializer {
    /// 序列化 CssNode 列表为 CSS 字符串。
    pub fn serialize(nodes: &[CssNode], style: OutputStyle) -> String {
        let flattened = Self::flatten_nodes(nodes);
        match style {
            OutputStyle::Expanded => Self::serialize_expanded(&flattened, 0),
            OutputStyle::Compressed => Self::serialize_compressed(&flattened),
        }
    }

    /// 展平嵌套规则。
    fn flatten_nodes(nodes: &[CssNode]) -> Vec<CssNode> {
        let mut result = Vec::new();
        for node in nodes {
            match node {
                CssNode::Rule { selector, declarations, children } => {
                    if !declarations.is_empty() {
                        result.push(CssNode::Rule {
                            selector: selector.clone(),
                            declarations: declarations.clone(),
                            children: vec![],
                        });
                    }
                    result.extend(Self::flatten_children(selector, children));
                }
                other => result.push(other.clone()),
            }
        }
        result
    }

    fn flatten_children(_parent: &str, children: &[CssNode]) -> Vec<CssNode> {
        let mut result = Vec::new();
        for child in children {
            match child {
                CssNode::Rule { selector, declarations, children: nested } => {
                    // 选择器已由 Evaluator 合并——不再二次合并
                    if !declarations.is_empty() {
                        result.push(CssNode::Rule {
                            selector: selector.clone(),
                            declarations: declarations.clone(),
                            children: vec![],
                        });
                    }
                    result.extend(Self::flatten_children(selector, nested));
                }
                other => result.push(other.clone()),
            }
        }
        result
    }

    fn serialize_expanded(nodes: &[CssNode], depth: usize) -> String {
        let indent = "  ".repeat(depth);
        let result: String = nodes
            .iter()
            .map(|n| Self::serialize_node_expanded(n, &indent, depth))
            .collect::<Vec<_>>()
            .join("\n");
        if depth == 0 {
            format!("{result}\n")
        } else {
            result
        }
    }

    fn serialize_compressed(nodes: &[CssNode]) -> String {
        nodes.iter().map(Self::serialize_node_compressed).collect::<Vec<_>>().join("")
    }

    fn serialize_node_expanded(node: &CssNode, indent: &str, depth: usize) -> String {
        match node {
            CssNode::Declaration { property, value, important } => {
                if *important {
                    format!("{indent}{property}: {value} !important;")
                } else {
                    format!("{indent}{property}: {value};")
                }
            }
            CssNode::Comment(text) => format!("{indent}/* {text} */"),
            CssNode::AtRoot(nodes) => Self::serialize_expanded(nodes, depth),
            CssNode::Rule { selector, declarations, children } => {
                let inner = "  ".repeat(depth + 1);
                let mut parts = vec![format!("{indent}{selector} {{")];
                for decl in declarations {
                    if let CssNode::Declaration { property, value, important } = decl {
                        if *important {
                            parts.push(format!("{inner}{property}: {value} !important;"));
                        } else {
                            parts.push(format!("{inner}{property}: {value};"));
                        }
                    }
                }
                if !children.is_empty() {
                    let child_css = Self::serialize_expanded(children, depth + 1);
                    if !child_css.is_empty() {
                        parts.push(child_css);
                    }
                }
                parts.push(format!("{indent}}}"));
                parts.join("\n")
            }
            CssNode::AtRule { name, params, children } => {
                let p = params.as_deref().unwrap_or("");
                if children.is_empty() {
                    // 空块——单行输出
                    if p.is_empty() {
                        format!("{indent}@{name} {{}}")
                    } else {
                        format!("{indent}@{name} {p} {{}}")
                    }
                } else {
                    let mut parts = if p.is_empty() {
                        vec![format!("{indent}@{name} {{")]
                    } else {
                        vec![format!("{indent}@{name} {p} {{")]
                    };
                    let child_css = Self::serialize_expanded(children, depth + 1);
                    if !child_css.is_empty() {
                        parts.push(child_css);
                    }
                    parts.push(format!("{indent}}}"));
                    parts.join("\n")
                }
            }
        }
    }

    fn serialize_node_compressed(node: &CssNode) -> String {
        match node {
            CssNode::Declaration { property, value, important } => {
                if *important {
                    format!("{property}:{value} !important;")
                } else {
                    format!("{property}:{value};")
                }
            }
            CssNode::Comment(_) => String::new(),
            CssNode::AtRoot(nodes) => nodes.iter().map(Self::serialize_node_compressed).collect::<String>(),
            CssNode::Rule { selector, declarations, children } => {
                let decls: String = declarations.iter().map(Self::serialize_node_compressed).collect();
                let kids: String = children.iter().map(Self::serialize_node_compressed).collect();
                format!("{selector}{{{decls}{kids}}}")
            }
            CssNode::AtRule { name, params, children } => {
                let p = params.as_deref().unwrap_or("");
                let kids: String = children.iter().map(Self::serialize_node_compressed).collect();
                format!("@{name} {p}{{{kids}}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_decl() {
        let nodes = vec![CssNode::Declaration {
            property: "color".into(), value: "red".into(), important: false,
        }];
        assert_eq!(Serializer::serialize(&nodes, OutputStyle::Expanded), "color: red;\n");
    }

    #[test]
    fn test_serialize_rule() {
        let nodes = vec![CssNode::Rule {
            selector: "a".into(),
            declarations: vec![CssNode::Declaration {
                property: "color".into(), value: "red".into(), important: false,
            }],
            children: vec![],
        }];
        assert_eq!(Serializer::serialize(&nodes, OutputStyle::Expanded), "a {\n  color: red;\n}\n");
    }
}
