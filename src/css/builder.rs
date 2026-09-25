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
use std::collections::HashMap;

use super::node::CssNode;
use tracing::info_span;

// ─── 构建器状态 ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
pub struct CssBuilder {
    /// 顶层输出累积
    pub output: Vec<CssNode>,
    /// 当前嵌套栈 (最近一个 Rule/AtRule)
    pub rule_stack: Vec<RuleFrame>,
    /// @mixin 定义表: mixin_name -> body_lines (不含 @{mixin name { 和 })
    pub mixins: HashMap<String, Vec<String>>,
    /// 当前正在收集的 @mixin: (name, body_lines)
    pending_mixin: Option<(String, Vec<String>)>,
    /// 当前 mixin 收集的 brace 嵌套深度
    collect_depth: i32,
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

        // 正在收集 @mixin body
        //
        // NOTE: 需要用 take 取出 pending 避免 borrow 冲突（body push 需要 &mut self,
        // 存入 mixins 也需要 &mut self）
        if self.pending_mixin.is_some() {
            // 正在收集 @mixin body
            if let Some((name, mut body)) = self.pending_mixin.take() {
                let delta = Self::count_braces(line);
                self.collect_depth += delta;
                if self.collect_depth <= 0 {
                    // 收集完成, 存入 mixins
                    self.mixins.insert(name, body);
                    self.collect_depth = 0;
                } else {
                    body.push(line.to_string());
                    self.pending_mixin = Some((name, body));
                }
            }
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

        // @mixin 定义开始: @mixin name { ... }
        if let Some(rest) = trimmed.strip_prefix("@mixin ") {
            // 解析 mixin 名 (到空格或 '(' 或 '{')
            let name_end = rest
                .find(|c: char| c == ' ' || c == '(' || c == '{')
                .unwrap_or(rest.len());
            let name = rest[..name_end].trim().to_string();
            let after_name = &rest[name_end..];

            // 判断是否是单行格式: @mixin name { body }
            if let Some(brace_open) = after_name.find('{') {
                // 找到对应的闭合: 检查整行是否包含匹配的 }
                if let Some(brace_close) = after_name.rfind('}') {
                    if brace_close > brace_open {
                        // 单行 mixin: body 在 { ... } 中间
                        let inner = &after_name[brace_open + 1..brace_close];
                        let body_lines: Vec<String> = inner
                            .split('\n')
                            .map(String::from)
                            .filter(|l| !l.trim().is_empty())
                            .collect();
                        self.mixins.insert(name, body_lines);
                        return vec![];
                    }
                }
            }

            // 多行 mixin: 进入收集模式
            self.pending_mixin = Some((name, vec![]));
            self.collect_depth = Self::count_braces(after_name);
            if self.collect_depth <= 0 {
                // 单行空 mixin @mixin name {}
                if let Some((name, body)) = self.pending_mixin.take() {
                    self.mixins.insert(name, body);
                }
                self.collect_depth = 0;
            }
            return vec![];
        }

            // @include: 展开 mixin
        if let Some(rest) = trimmed.strip_prefix("@include ") {
            let invoke = rest.trim().trim_end_matches(';').trim();
            // clone body 避免 borrow 冲突 (feed 需要 &mut self)
            let body_clone = self.mixins.get(invoke).cloned();
            if let Some(body) = body_clone {
                // 展开 mixin body: 逐行递归调用 self.feed
                let mut result = vec![];
                for line in &body {
                    result.extend(self.feed(line));
                }
                return result;
            }
        }

        // @at-root: 标记下一个规则提升到顶层
        let (trimmed, at_root) = self.strip_at_root(trimmed);

        // @extend 标记行: 由 ops.rs 注入的特殊标记
        if let Some(nodes) = Self::parse_extend_marker(trimmed) {
            return nodes;
        }

        // 单行完整规则: "selector { prop: val; ... }"
        if !trimmed.starts_with('@')
            && trimmed.contains('{')
            && trimmed.ends_with('}')
            && !trimmed.starts_with('$')
        {
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

        // 顶层 at-rule (单行, 不嵌套): @import / @charset — 整行 passthrough
        if trimmed.starts_with("@import ") || trimmed.starts_with("@charset ") {
            if trimmed.ends_with(';') {
                return vec![CssNode::Statement(trimmed.trim_end_matches(';').trim().to_string())];
            }
            return vec![CssNode::Statement(trimmed.to_string())];
        }

        // 其他行: 无法识别的顶层 at-rule / 未知行 — 静默丢弃
        vec![]
    }

    // ── 私有方法 ────────────────────────────────────────────────────────────

    /// 计算一行中的 { } 净深度 ({ +1, } -1)
    fn count_braces(line: &str) -> i32 {
        line.chars()
            .fold(0, |d, c| match c {
                '{' => d + 1,
                '}' => d - 1,
                _ => d,
            })
    }

    /// 预处理 @at-root: 返回 (去除前缀后的 trimmed, at_root 标志)
    fn strip_at_root<'a>(&self, trimmed: &'a str) -> (&'a str, bool) {
        if let Some(rest) = trimmed.strip_prefix("@at-root ") {
            (rest, true)
        } else {
            (trimmed, false)
        }
    }

    /// 处理 @extend 标记: >>EXTEND:extender:target:optional → ExtendMarker
    fn parse_extend_marker(trimmed: &str) -> Option<Vec<CssNode>> {
        let rest = trimmed.strip_prefix(">>EXTEND:")?;
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() >= 3 {
            let extender = parts[0].to_string();
            let optional = parts[parts.len() - 1] == "true";
            let target = parts[1..parts.len() - 1].join(",");
            Some(vec![CssNode::ExtendMarker { extender, target, optional }])
        } else {
            None
        }
    }

    // ── 私有方法 (Rule/Declaration) ────────────────────────────────────────

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
        } else if frame.children.is_empty() {
            // 空规则 (e.g. "a { @extend b !optional }" 未匹配) → 省略
            return vec![]
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

        // 空规则省略 (e.g. "a { @extend b !optional }")
        if children.is_empty() {
            return vec![];
        }

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
