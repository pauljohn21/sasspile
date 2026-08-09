//! CSS 序列化器——迭代器链实现。

pub mod node;

pub use node::CssNode;

use crate::OutputStyle;

/// 序列化器——CssNode 树 → CSS 字符串。
pub struct Serializer;

impl Serializer {
    /// 序列化 CssNode 列表为 CSS 字符串。
    pub fn serialize(nodes: &[CssNode], style: OutputStyle) -> String {
        // 先展平嵌套规则
        let flattened = Self::flatten_nodes(nodes);
        match style {
            OutputStyle::Expanded => Self::serialize_expanded(&flattened, 0),
            OutputStyle::Compressed => Self::serialize_compressed(&flattened),
        }
    }

    /// 展平嵌套规则：`.a { .b { ... } }` → `.a .b { ... }`。
    fn flatten_nodes(nodes: &[CssNode]) -> Vec<CssNode> {
        let mut result = Vec::new();
        for node in nodes {
            match node {
                CssNode::Rule {
                    selector,
                    declarations,
                    children,
                } => {
                    // 先添加当前规则（只含声明）
                    if !declarations.is_empty() {
                        result.push(CssNode::Rule {
                            selector: selector.clone(),
                            declarations: declarations.clone(),
                            children: vec![],
                        });
                    }
                    // 展平子规则，前缀当前选择器
                    let flattened_children = Self::flatten_children(selector, children);
                    result.extend(flattened_children);
                }
                other => result.push(other.clone()),
            }
        }
        result
    }

    /// 展平子规则并添加前缀。
    fn flatten_children(parent_selector: &str, children: &[CssNode]) -> Vec<CssNode> {
        let mut result = Vec::new();
        for child in children {
            match child {
                CssNode::Rule {
                    selector,
                    declarations,
                    children: nested,
                } => {
                    // 组合选择器
                    let combined = format!("{parent_selector} {selector}");
                    if !declarations.is_empty() {
                        result.push(CssNode::Rule {
                            selector: combined.clone(),
                            declarations: declarations.clone(),
                            children: vec![],
                        });
                    }
                    // 递归展平更深层的嵌套
                    let flattened = Self::flatten_children(&combined, nested);
                    result.extend(flattened);
                }
                // @规则保留子规则不展平
                CssNode::AtRule { .. } => result.push(child.clone()),
                other => result.push(other.clone()),
            }
        }
        result
    }

    /// 展开式序列化——带缩进和换行。
    fn serialize_expanded(nodes: &[CssNode], depth: usize) -> String {
        let indent = "  ".repeat(depth);
        let result: String = nodes
            .iter()
            .map(|node| Self::serialize_node_expanded(node, &indent, depth))
            .collect::<Vec<_>>()
            .join("\n");
        if depth == 0 {
            // 顶层加尾行换行
            format!("{result}\n")
        } else {
            result
        }
    }

    /// 压缩式序列化——无空白。
    fn serialize_compressed(nodes: &[CssNode]) -> String {
        nodes
            .iter()
            .map(Self::serialize_node_compressed)
            .collect::<Vec<_>>()
            .join("")
    }

    /// 序列化单个节点（展开式）。
    fn serialize_node_expanded(node: &CssNode, indent: &str, depth: usize) -> String {
        match node {
            CssNode::Declaration {
                property,
                value,
                important,
            } => {
                if *important {
                    format!("{indent}{property}: {value} !important;")
                } else {
                    format!("{indent}{property}: {value};")
                }
            }
            CssNode::Comment(text) => format!("{indent}/* {text} */"),
            CssNode::Rule {
                selector,
                declarations,
                children,
            } => {
                let inner_indent = "  ".repeat(depth + 1);
                let mut parts = vec![format!("{indent}{selector} {{")];
                // 声明
                for decl in declarations {
                    if let CssNode::Declaration {
                        property,
                        value,
                        important,
                    } = decl
                    {
                        if *important {
                            parts.push(format!("{inner_indent}{property}: {value} !important;"));
                        } else {
                            parts.push(format!("{inner_indent}{property}: {value};"));
                        }
                    }
                }
                // 子节点
                if !children.is_empty() {
                    let children_css = Self::serialize_expanded(children, depth + 1);
                    if !children_css.is_empty() {
                        parts.push(children_css);
                    }
                }
                parts.push(format!("{indent}}}"));
                parts.join("\n")
            }
            CssNode::AtRule {
                name,
                params,
                children,
            } => {
                let mut parts = vec![format!("{indent}@{} {{", name)];
                if let Some(p) = params {
                    parts = vec![format!("{indent}@{name} {p} {{")];
                }
                if !children.is_empty() {
                    let children_css = Self::serialize_expanded(children, depth + 1);
                    parts.push(children_css);
                }
                parts.push(format!("{indent}}}"));
                parts.join("\n")
            }
        }
    }

    /// 序列化单个节点（压缩式）。
    fn serialize_node_compressed(node: &CssNode) -> String {
        match node {
            CssNode::Declaration {
                property,
                value,
                important,
            } => {
                if *important {
                    format!("{property}:{value} !important;")
                } else {
                    format!("{property}:{value};")
                }
            }
            CssNode::Comment(text) => format!("/*{text}*/"),
            CssNode::Rule {
                selector,
                declarations,
                children,
            } => {
                let decls: Vec<String> = declarations
                    .iter()
                    .map(Self::serialize_node_compressed)
                    .collect();
                let children_css: Vec<String> = children
                    .iter()
                    .map(Self::serialize_node_compressed)
                    .collect();
                format!(
                    "{}{{{}{}}}",
                    selector,
                    decls.join(""),
                    children_css.join("")
                )
            }
            CssNode::AtRule {
                name,
                params,
                children,
            } => {
                let params_str = params.as_deref().unwrap_or("");
                let children_css: Vec<String> = children
                    .iter()
                    .map(Self::serialize_node_compressed)
                    .collect();
                format!("@{name} {}{{{}}}", params_str, children_css.join(""))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_decl_expanded() {
        let nodes = vec![CssNode::Declaration {
            property: "color".to_string(),
            value: "red".to_string(),
            important: false,
        }];
        let css = Serializer::serialize(&nodes, OutputStyle::Expanded);
        assert_eq!(css, "color: red;\n");
    }

    #[test]
    fn test_serialize_decl_compressed() {
        let nodes = vec![CssNode::Declaration {
            property: "color".to_string(),
            value: "red".to_string(),
            important: false,
        }];
        let css = Serializer::serialize(&nodes, OutputStyle::Compressed);
        assert_eq!(css, "color:red;");
    }

    #[test]
    fn test_serialize_important() {
        let nodes = vec![CssNode::Declaration {
            property: "color".to_string(),
            value: "red".to_string(),
            important: true,
        }];
        let css = Serializer::serialize(&nodes, OutputStyle::Expanded);
        assert_eq!(css, "color: red !important;\n");
    }

    #[test]
    fn test_serialize_rule() {
        let nodes = vec![CssNode::Rule {
            selector: "a".to_string(),
            declarations: vec![CssNode::Declaration {
                property: "color".to_string(),
                value: "red".to_string(),
                important: false,
            }],
            children: vec![],
        }];
        let css = Serializer::serialize(&nodes, OutputStyle::Expanded);
        assert_eq!(css, "a {\n  color: red;\n}\n");
    }
}
