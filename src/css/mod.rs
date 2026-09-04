//! CSS 序列化器——CssNode 树 → CSS 字符串。

pub mod node;
pub mod selector_ast;
mod selector;
pub mod selector_ops;
pub mod selector_parser;

pub use node::CssNode;
use selector::sanitize_selector;

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
                        // pop 出最后一个元素获取所有权，消除 clone
                        match result.pop() {
                            Some((CssNode::AtRule { children: last_children, name: last_name, params: last_params, .. }, last_gid)) => {
                                let mut merged = last_children;
                                merged.extend(children.clone());
                                result.push((CssNode::AtRule {
                                    name: last_name,
                                    params: last_params,
                                    children: merged,
                                    has_body: true,
                                }, last_gid));
                            }
                            _ => result.push((node, gid)),
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
    ///
    /// 使用 `scan` 状态机替代 `fold` + 可变 Vec：
    /// 状态 = `(next_group, prev_output)`，每步产出 `Vec<(CssNode, usize)>`，
    /// `flat_map` 展平为最终序列。
    fn flatten_nodes(nodes: &[CssNode], start_group: usize) -> Vec<(CssNode, usize)> {
        /// scan 状态：下一个可用 `group_id` + 前一个输出节点（用于 AtRoot/other 回看）
        struct ScanState {
            next_group: usize,
            prev: Option<(CssNode, usize)>,
        }

        /// 对单个节点生成 0 或多个 `(CssNode, usize)` 输出，并更新状态
        fn process_node(node: &CssNode, state: &mut ScanState) -> Vec<(CssNode, usize)> {
            match node {
                CssNode::Rule {
                    selector,
                    declarations,
                    children,
                } => {
                    let gid = state.next_group;
                    state.next_group += 1;
                    let mut out = Vec::new();
                    if !declarations.is_empty() {
                        out.push((
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
                    let flat = Serializer::flatten_children(selector, children, gid);
                    if has_non_rule_children {
                        let (rule_kids, other_kids): (
                            Vec<(CssNode, usize)>,
                            Vec<(CssNode, usize)>,
                        ) = flat
                            .into_iter()
                            .partition(|(k, _)| matches!(k, CssNode::Rule { .. }));
                        out.extend(rule_kids);
                        if !other_kids.is_empty() {
                            out.push((
                                CssNode::Rule {
                                    selector: selector.clone(),
                                    declarations: vec![],
                                    children: other_kids.into_iter().map(|(n, _)| n).collect(),
                                },
                                gid,
                            ));
                        }
                    } else {
                        out.extend(flat);
                    }
                    if let Some(last) = out.last() {
                        state.prev = Some(last.clone());
                    }
                    out
                }
                // AtRoot：保留为节点。
                // 连续无配置 AtRoot（@forward 不带 with）共享 group_id（无空行）。
                // 带配置 AtRoot（@forward with）与前一个之间分配新 group_id（有空行）。
                CssNode::AtRoot(_, marker) => {
                    let prev_is_unconfigured_atroot = matches!(
                        &state.prev,
                        Some((prev_n, _)) if matches!(prev_n, CssNode::AtRoot(_, None))
                    );
                    let gid = if marker.is_none() && prev_is_unconfigured_atroot {
                        state.prev.as_ref().map_or(0, |(_, g)| *g)
                    } else {
                        let g = state.next_group;
                        state.next_group += 1;
                        g
                    };
                    let item = (node.clone(), gid);
                    state.prev = Some(item.clone());
                    vec![item]
                }
                // 非 Rule 节点：继承前一个兄弟的 group_id（同源）
                other => {
                    let gid = state.prev.as_ref().map_or(state.next_group, |(_, g)| *g);
                    let item = (other.clone(), gid);
                    state.prev = Some(item.clone());
                    vec![item]
                }
            }
        }

        nodes
            .iter()
            .scan(
                ScanState {
                    next_group: start_group,
                    prev: None,
                },
                |state, node| Some(process_node(node, state)),
            )
            .flatten()
            .collect()
    }

    fn flatten_children(
        _parent: &str,
        children: &[CssNode],
        group_id: usize,
    ) -> Vec<(CssNode, usize)> {
        children
            .iter()
            .flat_map(|child| match child {
                CssNode::Rule {
                    selector,
                    declarations,
                    children: nested,
                } => {
                    let decls: Vec<(CssNode, usize)> = if declarations.is_empty() {
                        Vec::new()
                    } else {
                        vec![(
                            CssNode::Rule {
                                selector: selector.clone(),
                                declarations: declarations.clone(),
                                children: vec![],
                            },
                            group_id,
                        )]
                    };
                    decls
                        .into_iter()
                        .chain(Self::flatten_children(selector, nested, group_id))
                        .collect::<Vec<_>>()
                }
                other => vec![(other.clone(), group_id)],
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
                    let same_group = prev_gid == gid;
                    let same_origin = !same_group && Self::is_same_origin(prev_n, n);
                    if !prev_is_import
                        && !curr_is_import
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
                let wrapped: Vec<(CssNode, usize)> = nodes
                    .iter()
                    .enumerate()
                    .map(|(i, n)| (n.clone(), i + 1))
                    .collect();
                let inner = Self::serialize_expanded(&wrapped, depth);
                let trimmed = inner.strip_suffix('\n').unwrap_or(&inner);
                buf.push_str(trimmed);
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
