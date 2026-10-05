//! —— 规则求值 ——
//!
//! 提供 `eval_rule`（规则求值入口）和 `selector_contains_ampersand`（& 检测）。
//! 纯函数已迁移到 `selector_combine.rs`，状态机已迁移到 `rule_builder.rs`。

use super::selector_combine::combine_selectors;
use super::rule_builder::RuleBuilder;
use super::*;
use crate::css::node::CssNode;
use crate::error::{Result, SassError};

impl Evaluator {
    /// 求值规则——按顺序穿插输出声明组和嵌套规则。
    pub(crate) fn eval_rule(
        selector: &str,
        body: &[Node],
        env: Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = crate::__tracing::info_span!("eval_rule", selector = selector);
        let _enter = span.enter();
        // 对选择器中的 #{...} 插值求值——展开 #{...} 内部的 & 父选择器引用
        // 字面量 & 保留给 combine_selectors 处理
        let selector = if selector.contains("#{") {
            crate::eval::value::eval_selector_str(selector, &env)
        } else {
            selector.to_string()
        };

        // 顶层父选择器后缀检测：&a 在无父选择器或 AtRule 上下文时非法
        let is_top_level = env.get_selector().is_none_or(|s| s.starts_with('@'));
        if is_top_level {
            let trimmed = selector.trim_start();
            if let Some(rest) = trimmed.strip_prefix('&')
                .and_then(|rest| rest.chars().next())
            {
                if rest.is_alphanumeric() || rest == '-' {
                    return Err(SassError::Eval(
                        "A top-level selector may not contain a parent selector with a suffix.".into(),
                    ));
                }
            }
        }

        // & 位置检测：& 必须在 compound selector 开头
        // compound selector 由空格、>、+、~、, 分隔
        // 伪选择器括号内的 & 也是合法的（如 :is(&), :where(&)）
        if selector.contains('&') {
            let chars: Vec<char> = selector.chars().collect();
            for (i, &c) in chars.iter().enumerate() {
                if c == '&' && i > 0 {
                    let prev = chars[i - 1];
                    if prev != ' '
                        && prev != '>'
                        && prev != '+'
                        && prev != '~'
                        && prev != ','
                        && prev != '\t'
                        && prev != '\n'
                        && prev != '('
                    {
                        return Err(SassError::Eval(
                            "\"&\" may only used at the beginning of a compound selector.".into(),
                        ));
                    }
                }
            }
        }

        // FIX: 解析 selector 中的字面 & 后再存入 env.current_selector，
        // 确保 mixin 内 $selector: & 读取到的是正确解析值（而非字面 &）。
        // BEM CHAIN FIX: 使用 selector_chain（完整嵌套链）而非仅 get_selector()（直接父级）
        // 解析 & — 当 &.block--mod 的 body 内调用 e() 时，& 应展开为完整链
        // ".block.block--mod"，而非仅 ".block"。
        // ## 关键优化：get_selector_chain() 仅在需要时（含 & 时）join 段
        let resolved_sel = if selector.contains('&') {
            let parent_sel = env.get_selector_chain()
                .unwrap_or_else(|| env.get_selector().unwrap_or("").to_string());
            combine_selectors(&parent_sel, selector.as_str())
        } else {
            selector.clone()
        };
        // 归一化 current_selector：trim 尾逗号和空白
        let resolved_sel = resolved_sel.trim().trim_end_matches(',').trim().to_string();

        // ## 性能优化：save 用 Vector clone（O(log n) 结构共享，无 format!）
        //
        // OLD: `get_selector_chain().map(Rc::from)` 调用 format! 整个链 → O(n) + 堆分配
        // NEW: `selector_chain.clone()` imbl Vector 结构共享 → O(log n) 无数据复制
        //
        // Element-Plus 大量嵌套规则时，这个 save 在每层 eval_rule 执行，
        // 消除 format! 使得 N 层嵌套从 O(n²) 降为 O(n log n)。
        let parent_selector = env.get_selector().map(String::from);
        let parent_chain = env.selector_chain.clone();
        tracing::debug!(target: "chain_trace", selector = %resolved_sel, chain_depth = parent_chain.len(), "eval_rule enter");
        let env = env
            .enter_scope()
            .with_selector(resolved_sel.clone())
            .with_chain(&resolved_sel);
        tracing::debug!(target: "chain_trace", selector = %resolved_sel, chain_depth = env.selector_chain.len(), "eval_body_chain");
        let (css, new_env) = Self::eval_nodes(body, env)?;

        // 使用 RuleBuilder 累积输出——使用 resolved_sel（已展开 &），防止字面 & 泄漏到 CSS
        let mut builder = RuleBuilder::new(resolved_sel.clone());
        builder.set_resolved_self(resolved_sel.clone());
        for node in css {
            builder.push(node);
        }
        let result = builder.build();

        // 退出子作用域——恢复父 scope，传播 !global 和新增 mixin/function
        let return_env = new_env.exit_scope();

        // ## 恢复优化：Vector 直接移动（无 format!/split 开销）
        let return_env =
            return_env.restore_parent_context(parent_selector, parent_chain);

        Ok((result, return_env))
    }

    /// 递归检测 CssNode 中是否存在包含字面 `&` 的选择器。
    /// 用于判断 @at-root 子规则是否需要父选择器展开。
    pub(super) fn selector_contains_ampersand(node: &CssNode) -> bool {
        match node {
            CssNode::Rule { selector, children, .. } => {
                selector.contains('&') || children.iter().any(Self::selector_contains_ampersand)
            }
            CssNode::AtRule { children, .. } => {
                children.iter().any(Self::selector_contains_ampersand)
            }
            CssNode::AtRoot(nodes, _) => {
                nodes.iter().any(Self::selector_contains_ampersand)
            }
            CssNode::AtRootDirect(inner) => {
                Self::selector_contains_ampersand(inner)
            }
            _ => false,
        }
    }
}
