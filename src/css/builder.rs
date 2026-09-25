//! CssBuilder — scan_map reducer 将 style-line tokens 构建为 CssNode 树
//!
//! 设计:
//!   - CssBuilder 持有 scan_map 的 Acc 状态 (&mut self 就地修改)
//!   - feed 方法消费一行 token, 输出 Vec<CssNode>
//!   - 用 Rule 嵌套栈管理 "{" "}" 配对
//!   - @at-root 提升到顶层输出
//!
//! @media 合并由管线 merge_media_nodes 函数处理 (汇聚后一次性合并)

use std::borrow::Cow;

use super::node::CssNode;
use tracing::info_span;

// ─── 构建器状态 ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
pub struct CssBuilder {
    /// 顶层输出累积
    pub output: Vec<CssNode>,
    /// 当前嵌套栈 (最近一个 Rule/AtRule)
    pub rule_stack: Vec<RuleFrame>,
}

#[derive(Clone, Debug)]
pub struct RuleFrame {
    pub selector: String,
    pub children: Vec<CssNode>,
    pub is_atrule: bool,
    pub at_root: bool,
}

impl CssBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// 消费一行 token, 产出 CssNode (可能有多个, 或 0 个)
    pub fn feed(&mut self, line: &str) -> Vec<CssNode> {
        let _span = info_span!("css_builder.feed", line = %line).entered();
        let trimmed = line.trim();

        // 跳过空行
        if trimmed.is_empty() {
            return vec![];
        }

        // 闭合 brace: 弹出栈顶 Rule 并 emit
        if trimmed == "}" {
            return self.close_rule();
        }

        // At-Rule: @media @keyframes @supports
        if trimmed.starts_with("@media ") || trimmed.starts_with("@keyframes ") || trimmed.starts_with("@supports ") {
            return self.start_atrule(trimmed);
        }

        // @at-root: 标记下一个规则提升到顶层
        let (trimmed, at_root) = if let Some(rest) = trimmed.strip_prefix("@at-root ") {
            (rest, true)
        } else {
            (trimmed, false)
        };

        // @extend 标记行: 由 ops.rs 注入的特殊标记 (必须在 declaration 检查之前)
        // 格式: >>EXTEND:extender:target1:target2:...:optional
        if let Some(rest) = trimmed.strip_prefix(">>EXTEND:") {
            let parts: Vec<&str> = rest.split(':').collect();
            if parts.len() >= 3 {
                let extender = parts[0].to_string();
                let optional = parts[parts.len() - 1] == "true";
                // 中间部分都是 target (支持多目标)
                let target = parts[1..parts.len() - 1].join(",");
                return vec![CssNode::ExtendMarker { extender, target, optional }];
            }
        }

        // 单行完整规则: "selector { prop: val; ... }" — 直接解析 emit
        if !trimmed.starts_with('@') && trimmed.contains('{') && trimmed.ends_with('}') && !trimmed.starts_with('$') {
            return self.parse_single_line_rule(trimmed, at_root);
        }

        // 规则开启: "selector {"
        if trimmed.ends_with('{') && !trimmed.starts_with('@') {
            return self.start_rule(trimmed, at_root);
        }

        // 纯声明: "property: value;"
        if !trimmed.starts_with('@') && !trimmed.starts_with('$') && trimmed.contains(':') {
            return self.add_declaration(trimmed);
        }

        // 注释: // ... 或 /* ... */
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            return vec![CssNode::Comment(
                trimmed
                    .trim_start_matches("//")
                    .trim_start_matches("/*")
                    .trim_end_matches("*/")
                    .trim()
                    .to_string(),
            )];
        }

        // 其他行: 递归处理后检测 @at-root (回溯 prefix)
        vec![]
    }

    // ── 私有方法 ────────────────────────────────────────────────────────────

    fn start_rule(&mut self, line: &str, at_root: bool) -> Vec<CssNode> {
        // line = "selector {"
        let selector = line[..line.len() - 1].trim().to_string();
        // @at-root: 展开 & 为父选择器 (Cow 零分配回退)
        let selector = if at_root {
            self.expand_parent_ref(&selector).into_owned()
        } else {
            selector
        };
        self.rule_stack.push(RuleFrame {
            selector,
            children: vec![],
            is_atrule: false,
            at_root,
        });
        vec![]
    }

    /// 展开选择器中的 & 为父选择器 (无 & 时零分配, 借用输入)
    fn expand_parent_ref<'a>(&self, selector: &'a str) -> Cow<'a, str> {
        if !selector.contains('&') {
            return Cow::Borrowed(selector);
        }
        let parent = self.rule_stack.last().map(|f| f.selector.as_str()).unwrap_or("");
        if parent.is_empty() {
            Cow::Owned(selector.replace("&", ""))
        } else {
            Cow::Owned(selector.replace('&', parent))
        }
    }

    fn start_atrule(&mut self, line: &str) -> Vec<CssNode> {
        // line = "@media ... {" or "@keyframes ... {"
        let query = if let Some(brace_idx) = line.rfind('{') {
            line[..brace_idx].trim().to_string()
        } else {
            line.to_string()
        };
        self.rule_stack.push(RuleFrame {
            selector: query,
            children: vec![],
            is_atrule: true,
            at_root: false,
        });
        vec![]
    }

    fn close_rule(&mut self) -> Vec<CssNode> {
        let Some(frame) = self.rule_stack.pop() else {
            return vec![];
        };
        let node = if frame.is_atrule {
            CssNode::AtRule {
                query: frame.selector,
                children: frame.children,
            }
        } else {
            CssNode::Rule {
                selector: frame.selector,
                children: frame.children,
            }
        };

        // @at-root: 提升到顶层 (不附加到父 Rule)
        if frame.at_root {
            return self.emit_top_level(node);
        }

        if let Some(parent) = self.rule_stack.last_mut() {
            parent.children.push(node);
            vec![]
        } else {
            self.emit_top_level(node)
        }
    }

    /// 处理顶层输出
    fn emit_top_level(&mut self, node: CssNode) -> Vec<CssNode> {
        self.output.push(node.clone());
        vec![node]
    }

    fn add_declaration(&mut self, line: &str) -> Vec<CssNode> {
        // line 可能含有多条: "color: red; background: blue;"
        let parts: Vec<&str> = line.split(';').filter(|p| !p.trim().is_empty()).collect();
        let nodes: Vec<CssNode> = parts.iter().filter_map(|part| {
            let part = part.trim();
            let (prop, val) = part.split_once(':')?;
            let decl = CssNode::Declaration {
                property: prop.trim().to_string(),
                value: val.trim().trim_end_matches(';').trim().to_string(),
            };
            Some(decl)
        }).collect();

        // 附加到当前 Rule 或顶层
        if let Some(parent) = self.rule_stack.last_mut() {
            parent.children.extend(nodes);
            vec![]
        } else {
            // 无 Rule 上下文 (顶层声明), 直接 emit
            nodes
        }
    }

    fn parse_single_line_rule(&mut self, line: &str, at_root: bool) -> Vec<CssNode> {
        // line = "selector { prop: val; prop2: val2; }"
        let Some(brace_open) = line.find('{') else { return vec![]; };
        let Some(brace_close) = line.rfind('}') else { return vec![]; };
        if brace_close <= brace_open { return vec![]; }

        let selector = line[..brace_open].trim().to_string();
        let selector = if at_root {
            // @at-root 单行规则: 展开 & 为父选择器 (Cow 零分配回退)
            self.expand_parent_ref(&selector).into_owned()
        } else {
            selector
        };
        let body = &line[brace_open + 1..brace_close];

        // 解析 body 中的声明
        let children: Vec<CssNode> = body.split(';')
            .filter(|p| !p.trim().is_empty())
            .filter_map(|part| {
                let part = part.trim();
                let (prop, val) = part.split_once(':')?;
                Some(CssNode::Declaration {
                    property: prop.trim().to_string(),
                    value: val.trim().to_string(),
                })
            })
            .collect();

        let node = CssNode::Rule { selector, children };

        // @at-root: 提升到顶层
        if at_root {
            return self.emit_top_level(node);
        }

        // 附加到父 Rule 或作为顶层输出
        if let Some(parent) = self.rule_stack.last_mut() {
            parent.children.push(node);
            vec![]
        } else {
            self.emit_top_level(node)
        }
    }
}
