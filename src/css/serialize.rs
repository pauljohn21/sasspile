//! 序列化器——CssNode → String。

use super::CssNode;

/// 输出风格。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStyle {
    Expanded,
    Compressed,
}

/// 序列化完成。
pub struct Serialized {
    css: String,
}

impl Serialized {
    /// 从 CssNode 构建序列化结果。
    pub fn from_nodes(nodes: Vec<CssNode>, style: OutputStyle) -> Self {
        let mut output = String::new();
        serialize_nodes(&nodes, &mut output, style, 0);
        // 去掉末尾多余换行
        while output.ends_with('\n') {
            output.pop();
        }
        Self { css: output }
    }

    /// 消费为最终字符串。
    pub fn into_string(self) -> String {
        self.css
    }
}

fn serialize_nodes(
    nodes: &[CssNode],
    output: &mut String,
    style: OutputStyle,
    indent: usize,
) {
    for (i, node) in nodes.iter().enumerate() {
        if i > 0 && style == OutputStyle::Expanded {
            if needs_blank_line(node, &nodes[i - 1]) {
                output.push('\n');
            }
        }
        serialize_node(node, output, style, indent);
    }
}

fn needs_blank_line(current: &CssNode, prev: &CssNode) -> bool {
    matches!(current, CssNode::Rule { .. } | CssNode::AtRule { .. })
        || matches!(prev, CssNode::Rule { .. } | CssNode::AtRule { .. })
}

fn serialize_node(
    node: &CssNode,
    output: &mut String,
    style: OutputStyle,
    indent: usize,
) {
    let indent_str = if style == OutputStyle::Expanded {
        "  ".repeat(indent)
    } else {
        String::new()
    };

    match node {
        CssNode::Rule { selector, declarations, children } => {
            // 跳过空规则（无声明且无子规则）
            if declarations.is_empty() && children.is_empty() {
                return;
            }

            output.push_str(&indent_str);
            output.push_str(selector);
            if style == OutputStyle::Expanded {
                output.push_str(" {");
            } else {
                output.push('{');
            }

            for decl in declarations {
                serialize_node(decl, output, style, indent + 1);
            }

            for child in children {
                if style == OutputStyle::Expanded && !declarations.is_empty() {
                    output.push('\n');
                }
                serialize_node(child, output, style, indent + 1);
            }

            if style == OutputStyle::Expanded {
                output.push('\n');
                output.push_str(&indent_str);
            }
            output.push('}');
            output.push('\n');
        }
        CssNode::Declaration { property, value, important } => {
            if style == OutputStyle::Expanded {
                output.push('\n');
                output.push_str(&indent_str);
            }
            output.push_str(property);
            if style == OutputStyle::Expanded {
                output.push_str(": ");
            } else {
                output.push(':');
            }
            output.push_str(value);
            if *important {
                if style == OutputStyle::Expanded {
                    output.push_str(" !important");
                } else {
                    output.push_str("!important");
                }
            }
            output.push(';');
        }
        CssNode::Comment(s) => {
            // compressed 模式跳过注释
            if style == OutputStyle::Compressed {
                return;
            }
            output.push_str(&indent_str);
            output.push_str(&format!("/*{s}*/"));
            output.push('\n');
        }
        CssNode::AtRule { name, params, children, has_body } => {
            output.push_str(&indent_str);
            output.push('@');
            output.push_str(name);
            if !params.is_empty() {
                output.push(' ');
                output.push_str(params);
            }
            if *has_body {
                output.push_str(" {");
                serialize_nodes(children, output, style, indent + 1);
                if style == OutputStyle::Expanded {
                    output.push('\n');
                    output.push_str(&indent_str);
                }
                output.push('}');
            } else {
                output.push(';');
            }
            output.push('\n');
        }
        CssNode::AtRoot(nodes) => {
            serialize_nodes(nodes, output, style, indent);
        }
        CssNode::Return(_) => {}
    }
}
