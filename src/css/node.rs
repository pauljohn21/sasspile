//! CssNode AST — rxrust 管线输出的结构化 CSS 树
//!
//! 设计:
//!   - CssBuilder 在 scan_map 内消费 style-line tokens
//!   - flat_map 展开 Vec<CssNode> 为单个 CssNode
//!   - render_node 递归渲染为 String

use tracing::info_span;

// ─── CSS AST 节点 ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum CssNode {
    /// 规则节点 (选择器 + 子节点)
    Rule {
        selector: String,
        children: Vec<CssNode>,
    },
    /// 属性声明
    Declaration {
        property: String,
        value: String,
    },
    /// @media / @keyframes / @supports 等 at-rule
    AtRule {
        query: String,
        children: Vec<CssNode>,
    },
    /// @extend 标记 — 在 post-processing 阶段合并选择器
    /// (extender_selector, target_selector, optional)
    ExtendMarker {
        extender: String,
        target: String,
        optional: bool,
    },
    /// 注释
    Comment(String),
    /// 原始 CSS 语句 (不解析内部结构 — @import / @charset 等顶层 at-rule)
    /// 整行原样输出, 管线不做语法分析
    Statement(String),
}

// ─── 节点渲染 (AST → String) ───────────────────────────────────────────────

/// 渲染单个 CssNode 为 CSS 字符串 (递归)
pub fn render_node(node: &CssNode) -> String {
    let _span = info_span!("render_node", variant = ?node_variant_name(node)).entered();
    let indent_step = "  ";
    render_node_indent(node, 0, indent_step)
}

fn render_node_indent(node: &CssNode, depth: usize, indent_step: &str) -> String {
    let indent = indent_step.repeat(depth);
    match node {
        CssNode::Rule { selector, children } => {
            let mut out = format!("{indent}{selector} {{\n");
            for child in children {
                out.push_str(&render_node_indent(child, depth + 1, indent_step));
            }
            out.push_str(&format!("{indent}}}\n"));
            out
        }
        CssNode::Declaration { property, value } => {
            format!("{indent}{property}: {value};\n")
        }
        CssNode::AtRule { query, children } => {
            let mut out = format!("{indent}{query} {{\n");
            for child in children {
                out.push_str(&render_node_indent(child, depth + 1, indent_step));
            }
            out.push_str(&format!("{indent}}}\n"));
            out
        }
        CssNode::Comment(text) => {
            format!("{indent}/* {text} */\n")
        }
        // ExtendMarker 仅在 post-processing 阶段存在, 不应到达 render
        CssNode::ExtendMarker { .. } => String::new(),
        CssNode::Statement(s) => format!("{indent}{s};\n"),
    }
}

fn node_variant_name(node: &CssNode) -> &'static str {
    match node {
        CssNode::Rule { .. } => "Rule",
        CssNode::Declaration { .. } => "Declaration",
        CssNode::AtRule { .. } => "AtRule",
        CssNode::Comment(_) => "Comment",
        CssNode::ExtendMarker { .. } => "ExtendMarker",
        CssNode::Statement(_) => "Statement",
    }
}
