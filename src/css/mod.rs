//! CSS 序列化器——CssNode 树 → CSS 字符串。

pub mod node;
mod selector;

pub use node::CssNode;
use selector::sanitize_selector;

use crate::OutputStyle;

/// 序列化器。
pub struct Serializer;

impl Serializer {
    /// 序列化 `CssNode` 列表为 CSS 字符串。
    pub fn serialize(nodes: &[CssNode], style: OutputStyle) -> String {
        let flattened = Self::flatten_nodes(nodes, 0);
        let merged = Self::merge_at_rules(flattened);
        let css = match style {
            OutputStyle::Expanded => Self::serialize_expanded(&merged, 0),
            OutputStyle::Compressed => Self::serialize_compressed(&merged),
        };
        // 当输出包含非 ASCII 字符时，SCSS 规范要求 expanded 模式下添加 @charset 前缀
        if css.is_ascii() {
            css
        } else {
            match style {
                OutputStyle::Expanded => format!("@charset \"UTF-8\";\n{css}"),
                OutputStyle::Compressed => format!("@charset\"UTF-8\";{css}"),
            }
        }
    }

    /// 合并相邻的 @media/@supports 块（相同 query）。
    fn merge_at_rules(nodes: Vec<(CssNode, usize)>) -> Vec<(CssNode, usize)> {
        nodes.into_iter().fold(Vec::new(), |mut result, (node, gid)| {
            match &node {
                CssNode::AtRule {
                    name,
                    params,
                    children,
                    has_body: true,
                } => {
                    let should_merge = result.last().is_some_and(|(last, _)| {
                        matches!(last, CssNode::AtRule { name: last_name, params: last_params, has_body: true, .. } if last_name == name && last_params == params)
                    });
                    if should_merge {
                        let (merged_children, last_gid) = if let Some((CssNode::AtRule { children: last_children, .. }, last_gid)) = result.last() {
                            let mut merged = last_children.clone();
                            merged.extend(children.clone());
                            (merged, *last_gid)
                        } else {
                            (children.clone(), gid)
                        };
                        if let Some((last_mut, last_gid_val)) = result.last_mut() {
                            *last_mut = CssNode::AtRule {
                                name: name.clone(),
                                params: params.clone(),
                                children: merged_children,
                                has_body: true,
                            };
                            *last_gid_val = last_gid;
                        }
                    } else {
                        result.push((node, gid));
                    }
                }
                _ => result.push((node, gid)),
            }
            result
        })
    }

    /// 展平嵌套规则。返回 (`CssNode`, `group_id`) 对——同源展平规则共享相同 `group_id`。
    /// 同一组顶层兄弟节点（来自同一个 `eval_rule` 输出）共享 `group_id`。
    fn flatten_nodes(nodes: &[CssNode], start_group: usize) -> Vec<(CssNode, usize)> {
        nodes
            .iter()
            .fold(
                (Vec::new(), start_group),
                |(mut result, mut next_group), node| {
                    match node {
                        CssNode::Rule {
                            selector,
                            declarations,
                            children,
                        } => {
                            let gid = next_group;
                            next_group += 1;
                            if !declarations.is_empty() {
                                result.push((
                                    CssNode::Rule {
                                        selector: selector.clone(),
                                        declarations: declarations.clone(),
                                        children: vec![],
                                    },
                                    gid,
                                ));
                            }
                            let has_non_rule_children =
                                children.iter().any(|c| !matches!(c, CssNode::Rule { .. }));
                            if has_non_rule_children {
                                let flat = Self::flatten_children(selector, children, gid);
                                let (rule_kids, other_kids): (
                                    Vec<(CssNode, usize)>,
                                    Vec<(CssNode, usize)>,
                                ) = flat
                                    .into_iter()
                                    .partition(|(k, _)| matches!(k, CssNode::Rule { .. }));
                                result.extend(rule_kids);
                                if !other_kids.is_empty() {
                                    result.push((
                                        CssNode::Rule {
                                            selector: selector.clone(),
                                            declarations: vec![],
                                            children: other_kids
                                                .into_iter()
                                                .map(|(n, _)| n)
                                                .collect(),
                                        },
                                        gid,
                                    ));
                                }
                            } else {
                                result.extend(Self::flatten_children(selector, children, gid));
                            }
                        }
                        // AtRoot：保留为节点
                        // 连续 AtRoot（来自 @forward 链）共享 group_id（无空行）
                        // AtRoot 和 Rule 之间有不同 group_id（有空行）
                        CssNode::AtRoot(_, _) => {
                            let gid = if let Some((prev_n, prev_gid)) = result.last() {
                                if matches!(prev_n, CssNode::AtRoot(_, _)) {
                                    *prev_gid
                                } else {
                                    next_group += 1;
                                    next_group - 1
                                }
                            } else {
                                next_group += 1;
                                next_group - 1
                            };
                            result.push((node.clone(), gid));
                        }
                        // 非 Rule 节点：继承前一个兄弟的 group_id（同源）
                        other => {
                            let gid = if let Some((_, prev_gid)) = result.last() {
                                *prev_gid
                            } else {
                                next_group
                            };
                            result.push((other.clone(), gid));
                        }
                    }
                    (result, next_group)
                },
            )
            .0
    }

    fn flatten_children(
        _parent: &str,
        children: &[CssNode],
        group_id: usize,
    ) -> Vec<(CssNode, usize)> {
        children
            .iter()
            .flat_map(|child| {
                let mut result = Vec::new();
                match child {
                    CssNode::Rule {
                        selector,
                        declarations,
                        children: nested,
                    } => {
                        if !declarations.is_empty() {
                            result.push((
                                CssNode::Rule {
                                    selector: selector.clone(),
                                    declarations: declarations.clone(),
                                    children: vec![],
                                },
                                group_id,
                            ));
                        }
                        result.extend(Self::flatten_children(selector, nested, group_id));
                    }
                    other => result.push((other.clone(), group_id)),
                }
                result
            })
            .collect()
    }

    fn serialize_expanded(nodes: &[(CssNode, usize)], depth: usize) -> String {
        let indent = "  ".repeat(depth);
        let mut result = nodes.iter().enumerate().fold(String::new(), |mut acc, (i, (n, gid))| {
            if i > 0 {
                acc.push('\n');
                // 顶层规则之间添加空行（SCSS expanded 模式）
                // 但以下情况不加空行：
                // - @import 等无 body 的 @规则之间
                // - 连续注释之间
                // - 同源展平规则（group_id 相同）
                if depth == 0 {
                    let (prev_n, prev_gid) = &nodes[i - 1];
                    let prev_is_import = matches!(prev_n, CssNode::AtRule { name, has_body: false, .. } if name == "import");
                    let curr_is_import = matches!(n, CssNode::AtRule { name, has_body: false, .. } if name == "import");
                    let prev_is_comment = matches!(prev_n, CssNode::Comment(_));
                    let curr_is_comment = matches!(n, CssNode::Comment(_));
                    let same_group = prev_gid == gid;
                    let same_origin = !same_group && Self::is_same_origin(prev_n, n);
                    if !(prev_is_import || curr_is_import)
                        && !(prev_is_comment && curr_is_comment)
                        && !prev_is_comment
                        && !same_group
                        && !same_origin
                    {
                        acc.push('\n');
                    }
                }
            }
            Self::write_node_expanded(&mut acc, n, &indent, depth);
            acc
        });
        if depth == 0 {
            result.push('\n');
        }
        result
    }

    /// 启发式：判断两个顶层兄弟 Rule 是否来自同一 `eval_rule` 输出。
    /// 仅在 `group_id` 不同时使用——检查选择器后代关系（非完全相同）。
    fn is_same_origin(prev: &CssNode, curr: &CssNode) -> bool {
        match (prev, curr) {
            (
                CssNode::Rule {
                    selector: prev_sel, ..
                },
                CssNode::Rule {
                    selector: curr_sel, ..
                },
            ) => {
                let prev_sel = prev_sel.trim();
                let curr_sel = curr_sel.trim();
                // 选择器完全相同时不通过启发式判断——依赖 group_id
                if prev_sel == curr_sel {
                    return false;
                }
                // 后代关系：curr 以 prev 为前缀，或反过来
                let is_descendant = |a: &str, b: &str| {
                    a.split(',').all(|p| {
                        let p = p.trim();
                        b.split(',').any(|c| {
                            let c = c.trim();
                            c.starts_with(&format!("{p} "))
                        })
                    })
                };
                is_descendant(prev_sel, curr_sel) || is_descendant(curr_sel, prev_sel)
            }
            _ => false,
        }
    }

    fn serialize_compressed(nodes: &[(CssNode, usize)]) -> String {
        nodes.iter().fold(String::new(), |mut acc, (n, _)| {
            Self::write_node_compressed(&mut acc, n);
            acc
        })
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
            CssNode::AtRoot(nodes, _) => {
                let indent = "  ".repeat(depth);
                let inner = nodes.iter().fold(String::new(), |mut acc, kid| {
                    if !acc.is_empty() {
                        acc.push('\n');
                    }
                    Self::write_node_expanded(&mut acc, kid, &indent, depth);
                    acc
                });
                buf.push_str(&inner);
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
                    let wrapped: Vec<(CssNode, usize)> =
                        children.iter().cloned().map(|n| (n, 0)).collect();
                    let child_css = Self::serialize_expanded(&wrapped, depth + 1);
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
                    let wrapped: Vec<(CssNode, usize)> =
                        children.iter().cloned().map(|n| (n, 0)).collect();
                    let child_css = Self::serialize_expanded(&wrapped, depth + 1);
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
            CssNode::AtRoot(nodes, _) => {
                let wrapped: Vec<(CssNode, usize)> =
                    nodes.iter().cloned().map(|n| (n, 0)).collect();
                buf.push_str(&Self::serialize_compressed(&wrapped));
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
