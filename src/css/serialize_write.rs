//! CSS 序列化写入器——`write_node_expanded` / `write_node_compressed`。
//!
//! 从 `serialize.rs` 拆分出来，包含直接写入 String 缓冲区的节点序列化逻辑。

use crate::css::node::CssNode;
use crate::css::selector::sanitize_selector;

use super::Serializer;

impl Serializer {
    /// 直接写入 String 缓冲区——避免 format! + collect + join 的多重分配。
    pub(super) fn write_node_expanded(buf: &mut String, node: &CssNode, indent: &str, depth: usize) {
        match node {
            CssNode::Declaration {
                property,
                value,
                important,
            } => {
                // 空字符串值不输出属性声明（Sass 规范）
                if value.is_empty() {
                    return;
                }
                buf.push_str(indent);
                buf.push_str(property);
                buf.push_str(": ");
                buf.push_str(value);
                match *important {
                    true => buf.push_str(" !important"),
                    false => {}
                }
                buf.push(';');
            }
            CssNode::Comment(text) => {
                buf.push_str(indent);
                buf.push_str("/* ");
                buf.push_str(text);
                buf.push_str(" */");
            }
            CssNode::AtRoot(nodes, _) => {
                let wrapped: Vec<(CssNode, usize)> = nodes
                    .iter()
                    .enumerate()
                    .map(|(i, n)| (n.clone(), i + 1))
                    .collect();
                let inner = Self::serialize_expanded(&wrapped, depth);
                let trimmed = inner.strip_suffix('\n').unwrap_or(&inner);
                buf.push_str(trimmed);
            }
            // AtRootDirect 在 flatten 中已展开，此处兜底：直接序列化内部节点
            CssNode::AtRootDirect(inner) => {
                let inner_css = Self::serialize_expanded(&[((**inner).clone(), 0)], depth);
                let trimmed = inner_css.strip_suffix('\n').unwrap_or(&inner_css);
                buf.push_str(trimmed);
            }
            CssNode::Rule {
                selector,
                declarations,
                children,
            } => {
                let selector = sanitize_selector(selector);
                match selector.is_empty() {
                    true => return,
                    false => {}
                }
                let inner = "  ".repeat(depth + 1);
                buf.push_str(indent);
                buf.push_str(&selector);
                buf.push_str(" {\n");
                let decls_css: String = declarations
                    .iter()
                    .filter_map(|decl| match decl {
                        CssNode::Declaration {
                            property,
                            value,
                            important,
                        } => {
                            // 空字符串值不输出属性声明（Sass 规范）
                            if value.is_empty() {
                                return None;
                            }
                            let mut s = String::new();
                            s.push_str(&inner);
                            s.push_str(property);
                            s.push_str(": ");
                            s.push_str(value);
                            match *important {
                                true => s.push_str(" !important"),
                                false => {}
                            }
                            s.push(';');
                            s.push('\n');
                            Some(s)
                        }
                        _ => None,
                    })
                    .collect();
                buf.push_str(&decls_css);
                match !children.is_empty() {
                    true => {
                        let wrapped: Vec<(CssNode, usize)> =
                            children.iter().cloned().map(|n| (n, 0)).collect();
                        let child_css = Self::serialize_expanded(&wrapped, depth + 1);
                        match !child_css.is_empty() {
                            true => {
                                buf.push_str(&child_css);
                                buf.push('\n');
                            }
                            false => {}
                        }
                    }
                    false => {}
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
                match children.is_empty() {
                    true => {
                        buf.push_str(indent);
                        buf.push('@');
                        buf.push_str(name);
                        match !p.is_empty() {
                            true => {
                                buf.push(' ');
                                buf.push_str(p);
                            }
                            false => {}
                        }
                        buf.push_str(" {}");
                    }
                    false => {
                        buf.push_str(indent);
                        buf.push('@');
                        buf.push_str(name);
                        match !p.is_empty() {
                            true => {
                                buf.push(' ');
                                buf.push_str(p);
                            }
                            false => {}
                        }
                        buf.push_str(" {\n");
                        let wrapped: Vec<(CssNode, usize)> =
                            children.iter().cloned().map(|n| (n, 0)).collect();
                        let child_css = Self::serialize_expanded(&wrapped, depth + 1);
                        match !child_css.is_empty() {
                            true => {
                                buf.push_str(&child_css);
                                buf.push('\n');
                            }
                            false => {}
                        }
                        buf.push_str(indent);
                        buf.push('}');
                    }
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
                match !p.is_empty() {
                    true => {
                        buf.push(' ');
                        buf.push_str(p);
                    }
                    false => {}
                }
                buf.push(';');
            }
            CssNode::Raw(text) => {
                buf.push_str(text);
            }
            CssNode::Return(_) => {}
        }
    }

    pub(super) fn write_node_compressed(buf: &mut String, node: &CssNode) {
        match node {
            CssNode::Declaration {
                property,
                value,
                important,
            } => {
                // 空字符串值不输出属性声明（Sass 规范）
                if value.is_empty() {
                    return;
                }
                buf.push_str(property);
                buf.push(':');
                buf.push_str(value);
                match *important {
                    true => buf.push_str(" !important"),
                    false => {}
                }
                buf.push(';');
            }
CssNode::Comment(_) => {}
CssNode::AtRoot(nodes, _) => {
    let wrapped: Vec<(CssNode, usize)> =
        nodes.iter().cloned().map(|n| (n, 0)).collect();
    buf.push_str(&Self::serialize_compressed(&wrapped));
}
CssNode::AtRootDirect(inner) => {
    buf.push_str(&Self::serialize_compressed(&[((**inner).clone(), 0)]));
}
            CssNode::Rule {
                selector,
                declarations,
                children,
            } => {
                let sel = sanitize_selector(selector);
                match sel.is_empty() {
                    true => return,
                    false => {}
                }
                buf.push_str(&sel);
                buf.push('{');
                declarations
                    .iter()
                    .for_each(|decl| Self::write_node_compressed(buf, decl));
                children
                    .iter()
                    .for_each(|kid| Self::write_node_compressed(buf, kid));
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
                match !p.is_empty() {
                    true => {
                        buf.push(' ');
                        buf.push_str(p);
                    }
                    false => {}
                }
                buf.push('{');
                children
                    .iter()
                    .for_each(|kid| Self::write_node_compressed(buf, kid));
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
                match !p.is_empty() {
                    true => {
                        buf.push(' ');
                        buf.push_str(p);
                    }
                    false => {}
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
