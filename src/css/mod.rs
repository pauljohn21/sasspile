//! CSS 序列化器——CssNode 树 → CSS 字符串。

pub mod node;
mod selector;

pub use node::CssNode;
use selector::sanitize_selector;

use crate::OutputStyle;

/// 序列化器。
pub struct Serializer;

impl Serializer {
    /// 序列化 CssNode 列表为 CSS 字符串。
    pub fn serialize(nodes: &[CssNode], style: OutputStyle) -> String {
        let flattened = Self::flatten_nodes(nodes);
        let merged = Self::merge_at_rules(flattened);
        let css = match style {
            OutputStyle::Expanded => Self::serialize_expanded(&merged, 0),
            OutputStyle::Compressed => Self::serialize_compressed(&merged),
        };
        // 当输出包含非 ASCII 字符时，Dart Sass 在 expanded 模式下添加 @charset 前缀
        if !css.is_ascii() {
            match style {
                OutputStyle::Expanded => format!("@charset \"UTF-8\";\n{css}"),
                OutputStyle::Compressed => format!("@charset\"UTF-8\";{css}"),
            }
        } else {
            css
        }
    }

    /// 合并相邻的 @media/@supports 块（相同 query）。
    fn merge_at_rules(nodes: Vec<CssNode>) -> Vec<CssNode> {
        let mut result: Vec<CssNode> = Vec::new();
        for node in nodes {
            match &node {
                CssNode::AtRule {
                    name,
                    params,
                    children,
                    has_body: true,
                } => {
                    // 检查是否与 result 中最后一个节点同名同 query
                    if let Some(last) = result.last()
                        && let CssNode::AtRule {
                            name: last_name,
                            params: last_params,
                            children: last_children,
                            has_body: true,
                        } = last
                            && last_name == name && last_params == params {
                                // 合并 children
                                let mut merged = last_children.clone();
                                merged.extend(children.clone());
                                if let Some(last_mut) = result.last_mut() {
                                    *last_mut = CssNode::AtRule {
                                        name: name.clone(),
                                        params: params.clone(),
                                        children: merged,
                                        has_body: true,
                                    };
                                }
                                continue;
                            }
                    result.push(node);
                }
                _ => result.push(node),
            }
        }
        result
    }

    /// 展平嵌套规则。
    fn flatten_nodes(nodes: &[CssNode]) -> Vec<CssNode> {
        let mut result = Vec::new();
        for node in nodes {
            match node {
                CssNode::Rule {
                    selector,
                    declarations,
                    children,
                } => {
                    if !declarations.is_empty() {
                        result.push(CssNode::Rule {
                            selector: selector.clone(),
                            declarations: declarations.clone(),
                            children: vec![],
                        });
                    }
                    // 如果 children 中有非 Rule 节点（如 AtRule），保留 Rule 包裹
                    let has_non_rule_children = children.iter().any(|c| !matches!(c, CssNode::Rule { .. }));
                    if has_non_rule_children {
                        let flat = Self::flatten_children(selector, children);
                        // 分离 Rule 子节点和非 Rule 子节点
                        let mut rule_kids = Vec::new();
                        let mut other_kids = Vec::new();
                        for kid in flat {
                            match &kid {
                                CssNode::Rule { .. } => rule_kids.push(kid),
                                _ => other_kids.push(kid),
                            }
                        }
                        result.extend(rule_kids);
                        if !other_kids.is_empty() {
                            result.push(CssNode::Rule {
                                selector: selector.clone(),
                                declarations: vec![],
                                children: other_kids,
                            });
                        }
                    } else {
                        result.extend(Self::flatten_children(selector, children));
                    }
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
                CssNode::Rule {
                    selector,
                    declarations,
                    children: nested,
                } => {
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
        let mut result = String::new();
        for (i, n) in nodes.iter().enumerate() {
            if i > 0 {
                result.push('\n');
                // 顶层规则之间添加空行（Dart Sass expanded 模式）
                // 但以下情况不加空行：
                // - @import 等无 body 的 @规则之间
                // - 连续注释之间
                if depth == 0 {
                    let prev_is_import = matches!(&nodes[i - 1], CssNode::AtRule { name, has_body: false, .. } if name == "import");
                    let curr_is_import = matches!(n, CssNode::AtRule { name, has_body: false, .. } if name == "import");
                    let prev_is_comment = matches!(&nodes[i - 1], CssNode::Comment(_));
                    let curr_is_comment = matches!(n, CssNode::Comment(_));
                    if !(prev_is_import || curr_is_import)
                        && !(prev_is_comment && curr_is_comment)
                    {
                        result.push('\n');
                    }
                }
            }
            Self::write_node_expanded(&mut result, n, &indent, depth);
        }
        if depth == 0 {
            result.push('\n');
        }
        result
    }

    fn serialize_compressed(nodes: &[CssNode]) -> String {
        let mut result = String::new();
        for n in nodes {
            Self::write_node_compressed(&mut result, n);
        }
        result
    }

    /// 直接写入 String 缓冲区——避免 format! + collect + join 的多重分配。
    fn write_node_expanded(buf: &mut String, node: &CssNode, indent: &str, depth: usize) {
        match node {
            CssNode::Declaration {
                property,
                value,
                important,
            } => {
                buf.push_str(indent);
                buf.push_str(property);
                buf.push_str(": ");
                buf.push_str(value);
                if *important {
                    buf.push_str(" !important");
                }
                buf.push(';');
            }
            CssNode::Comment(text) => {
                buf.push_str(indent);
                buf.push_str("/* ");
                buf.push_str(text);
                buf.push_str(" */");
            }
            CssNode::AtRoot(nodes) => {
                buf.push_str(&Self::serialize_expanded(nodes, depth));
            }
            CssNode::Rule {
                selector,
                declarations,
                children,
            } => {
                let selector = sanitize_selector(selector);
                if selector.is_empty() {
                    return;
                }
                let inner = "  ".repeat(depth + 1);
                buf.push_str(indent);
                buf.push_str(&selector);
                buf.push_str(" {\n");
                for decl in declarations {
                    if let CssNode::Declaration {
                        property,
                        value,
                        important,
                    } = decl
                    {
                        buf.push_str(&inner);
                        buf.push_str(property);
                        buf.push_str(": ");
                        buf.push_str(value);
                        if *important {
                            buf.push_str(" !important");
                        }
                        buf.push(';');
                        buf.push('\n');
                    }
                }
                if !children.is_empty() {
                    let child_css = Self::serialize_expanded(children, depth + 1);
                    if !child_css.is_empty() {
                        buf.push_str(&child_css);
                        buf.push('\n');
                    }
                }
                buf.push_str(indent);
                buf.push('}');
            }
            CssNode::AtRule {
                has_body: true,
                name,
                params,
                children,
            } => {
                let p = params.as_deref().unwrap_or("");
                if children.is_empty() {
                    buf.push_str(indent);
                    buf.push('@');
                    buf.push_str(name);
                    if !p.is_empty() {
                        buf.push(' ');
                        buf.push_str(p);
                    }
                    buf.push_str(" {}");
                } else {
                    buf.push_str(indent);
                    buf.push('@');
                    buf.push_str(name);
                    if !p.is_empty() {
                        buf.push(' ');
                        buf.push_str(p);
                    }
                    buf.push_str(" {\n");
                    let child_css = Self::serialize_expanded(children, depth + 1);
                    if !child_css.is_empty() {
                        buf.push_str(&child_css);
                        buf.push('\n');
                    }
                    buf.push_str(indent);
                    buf.push('}');
                }
            }
            CssNode::AtRule {
                has_body: false,
                name,
                params,
                ..
            } => {
                let p = params.as_deref().unwrap_or("");
                buf.push_str(indent);
                buf.push('@');
                buf.push_str(name);
                if !p.is_empty() {
                    buf.push(' ');
                    buf.push_str(p);
                }
                buf.push(';');
            }
            CssNode::Raw(text) => {
                buf.push_str(text);
            }
            CssNode::Return(_) => {}
        }
    }

    fn write_node_compressed(buf: &mut String, node: &CssNode) {
        match node {
            CssNode::Declaration {
                property,
                value,
                important,
            } => {
                buf.push_str(property);
                buf.push(':');
                buf.push_str(value);
                if *important {
                    buf.push_str(" !important");
                }
                buf.push(';');
            }
            CssNode::Comment(_) => {}
            CssNode::AtRoot(nodes) => {
                buf.push_str(&Self::serialize_compressed(nodes));
            }
            CssNode::Rule {
                selector,
                declarations,
                children,
            } => {
                let sel = sanitize_selector(selector);
                if sel.is_empty() {
                    return;
                }
                buf.push_str(&sel);
                buf.push('{');
                for decl in declarations {
                    Self::write_node_compressed(buf, decl);
                }
                for kid in children {
                    Self::write_node_compressed(buf, kid);
                }
                buf.push('}');
            }
            CssNode::AtRule {
                has_body: true,
                name,
                params,
                children,
            } => {
                let p = params.as_deref().unwrap_or("");
                buf.push('@');
                buf.push_str(name);
                if !p.is_empty() {
                    buf.push(' ');
                    buf.push_str(p);
                }
                buf.push('{');
                for kid in children {
                    Self::write_node_compressed(buf, kid);
                }
                buf.push('}');
            }
            CssNode::AtRule {
                has_body: false,
                name,
                params,
                ..
            } => {
                let p = params.as_deref().unwrap_or("");
                buf.push('@');
                buf.push_str(name);
                if !p.is_empty() {
                    buf.push(' ');
                    buf.push_str(p);
                }
                buf.push(';');
            }
            CssNode::Raw(text) => {
                buf.push_str(text);
            }
            CssNode::Return(_) => {}
        }
    }
}
