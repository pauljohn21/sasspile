use super::*;
use crate::css::node::CssNode;
use crate::error::Result;

/// 判断 child selector 是否已经以 parent 作为 compound 前缀。
/// 用于 outer Rule arm combine：当 inner eval 输出已包含 parent 语义时跳过 descendant combine。
fn starts_with_compound_prefix(parent: &str, child: &str) -> bool {
    if parent.is_empty() || child == parent {
        return !parent.is_empty();
    }
    let separators: &[u8] = b".:#[>+~";
    child.len() > parent.len()
        && child.starts_with(parent)
        && child.as_bytes().get(parent.len()).is_some_and(|c| separators.contains(c))
}

/// 规则构建器——封装 `eval_rule` 的累积器状态。
///
/// `result` 是最终输出节点列表，`current_decls` 是当前累积的声明，
/// `root_nodes` 是 @at-root 提升的节点。
/// `resolved_self` 是 selector 展开 `&` 后的值，用于 compound 前缀检测。
struct RuleBuilder {
    selector: String,
    resolved_self: String,
    result: Vec<CssNode>,
    current_decls: Vec<CssNode>,
    root_nodes: Vec<CssNode>,
}

impl RuleBuilder {
    fn new(selector: String) -> Self {
        Self {
            resolved_self: selector.clone(),
            selector,
            result: Vec::new(),
            current_decls: Vec::new(),
            root_nodes: Vec::new(),
        }
    }

    /// flush 当前累积的声明为一条 Rule 节点。
    fn flush_decls(&mut self) {
        match !self.current_decls.is_empty() {
            true => {
                let decls = std::mem::take(&mut self.current_decls);
                self.result.push(CssNode::Rule {
                    selector: self.selector.clone(),
                    declarations: decls,
                    children: vec![],
                });
            }
            false => {}
        }
    }

    /// push 一个 CSS 节点到构建器（&mut self 模式——Group 1 重构关键前置）。
    #[tracing::instrument(skip(self, node), fields(sel = %self.selector, node = ?std::mem::discriminant(&node)))]
    fn push(&mut self, node: CssNode) {
        match node {
            CssNode::Declaration { .. } => {
                self.current_decls.push(node);
            }
            CssNode::AtRoot(nodes, query) => {
                // 解析 @at-root query 语义（官方文档）
                // without: media/supports → 脱离 @media 但保留父选择器
                // without: rule → 脱离父选择器（默认行为）
                // without: all → 脱离所有包裹
                // with: rule → 只保留 style rules，排除所有 at-rules
                // 无 query → 脱离父选择器（默认行为）
                let without_media = query
                    .as_ref()
                    .is_some_and(|q| q.contains("without: media") || q.contains("without:media"));
                let without_supports = query.as_ref().is_some_and(|q| {
                    q.contains("without: supports") || q.contains("without:supports")
                });
                let without_all = query
                    .as_ref()
                    .is_some_and(|q| q.contains("without: all") || q.contains("without:all"));
                let with_rule = query
                    .as_ref()
                    .is_some_and(|q| q.contains("with: rule") || q.contains("with:rule"));
                // `&` 父选择器检测：即使无 query，含 `&` 的规则也需父选择器展开
                let has_parent_ref = nodes.iter().any(Evaluator::selector_contains_ampersand);
                match without_media || without_supports || without_all || with_rule || has_parent_ref {
                    true => {
                        let nested = Evaluator::nest_rule_in_children(&self.selector, nodes);
                        self.flush_decls();
                        self.result.push(CssNode::AtRoot(nested, query));
                    }
                    false => {
                        self.root_nodes.extend(nodes);
                    }
                }
            }
            // AtRootDirect：来自 mixin at-rule，selector 可能含字面 &（如 when() mixin 的 &.disabled）。
            // 含 & 时需结合 self.selector 展开（combine_selectors）；
            // 不含 & 时也需与 self.selector 嵌套——内层 @at-root 需继承外层 EP BEM 链前缀。
            CssNode::AtRootDirect(inner) => {
                self.flush_decls();
                self.push_atroot_direct(*inner);
            }
            CssNode::Rule {
                selector: child_sel,
                declarations: child_decls,
                children: child_kids,
            } => {
                self.flush_decls();
                // Group 3 修复：用 resolved_self（& 已展开为完整 compound）检测 inner eval 输出
                // 是否已包含 self 语义前缀。对 `&.--vertical { &.--underline {} }` 链式嵌套，
                // inner Rule arm combine 时 self.selector 仍是 "&.--vertical"（字面）、
                // 此时 child_sel 已是 ".el-anchor.el-anchor--vertical.el-anchor--underline"（已展开），
                // 用 resolved_self 才能正确识别 → 跳过 double-prefix 的 descendant combine。
                let has_prefix = starts_with_compound_prefix(&self.resolved_self, &child_sel);
                let combined = if has_prefix {
                    child_sel.clone()
                } else {
                    Evaluator::combine_selectors(&self.selector, &child_sel)
                };
                match !child_decls.is_empty() {
                    true => {
                        self.result.push(CssNode::Rule {
                            selector: combined.clone(),
                            declarations: child_decls,
                            children: vec![],
                        });
                    }
                    false => {}
                }
                for kid in child_kids {
                    if let CssNode::Rule {
                        selector: kid_sel,
                        declarations: kid_decls,
                        ..
                    } = kid
                    {
                        let kid_combined = Evaluator::combine_selectors(&combined, &kid_sel);
                        match !kid_decls.is_empty() {
                            true => {
                                self.result.push(CssNode::Rule {
                                    selector: kid_combined,
                                    declarations: kid_decls,
                                    children: vec![],
                                });
                            }
                            false => {}
                        }
                    } else if let CssNode::AtRootDirect(inner) = kid {
                        // BEM AtRootDirect 嵌套修复：当 AtRootDirect 出现在 child_kids 中时，
                        // 需使用当前 Rule 的 combined selector 作为父上下文，
                        // 避免丢失中间 BEM 层级（如 .el-anchor__list → .el-anchor__item 链）。
                        let orig_selector = self.selector.clone();
                        let orig_resolved = self.resolved_self.clone();
                        self.selector = combined.clone();
                        self.resolved_self = combined.clone();
                        self.push_atroot_direct(*inner);
                        self.selector = orig_selector;
                        self.resolved_self = orig_resolved;
                    } else {
                        // Group 4 修复：递归 dispatch，让其他节点进入 push
                        self.push(kid);
                    }
                }
            }
            other => {
                self.flush_decls();
                let other = match other {
                    CssNode::AtRule {
                        name,
                        params,
                        children,
                        has_body: true,
                    } => {
                        let is_kf = name == "keyframes"
                            || name == "-webkit-keyframes"
                            || name == "-moz-keyframes";
                        let ch = if is_kf {
                            children
                        } else {
                            Evaluator::nest_rule_in_children(&self.selector, children)
                        };
                        CssNode::AtRule {
                            name,
                            params,
                            children: ch,
                            has_body: true,
                        }
                    }
                    CssNode::AtRule {
                        name,
                        params,
                        children: _,
                        has_body: false,
                    } => CssNode::Rule {
                        selector: self.selector.clone(),
                        declarations: vec![],
                        children: vec![CssNode::AtRule {
                            name,
                            params,
                            children: vec![],
                            has_body: false,
                        }],
                    },
                    other => other,
                };
                self.result.push(other);
            }
        }
    }

    /// 处理 AtRootDirect 节点：组合选择器 + 递归处理嵌套子节点。
    ///
    /// 核心语义：
    /// 1. AtRootDirect 的 selector 与 self.selector 组合（& 替换或嵌套）
    /// 2. 父 Rule 仅含自身声明，不含子节点
    /// 3. 子节点（含嵌套 AtRootDirect）生成独立 Rule，selector 与 resolved 组合
    fn push_atroot_direct(&mut self, inner: CssNode) {
        match inner {
            CssNode::Rule { selector, declarations, children } => {
                // 关键：trim 尾随逗号——SCSS @at-root mixin 的选择器可能有 "&--large," 形式
                let clean_sel = selector.trim().trim_end_matches(',').trim();
                let resolved_selector = if clean_sel.contains('&') {
                    let combined = Evaluator::combine_selectors(&self.selector, clean_sel);
                    combined.trim().trim_end_matches(',').trim().to_string()
                } else if clean_sel.starts_with(':') || clean_sel.starts_with('[') {
                    // 伪类/属性选择器：直接拼接（后缀型，需依附于父选择器）
                    format!("{}{}", self.selector, clean_sel)
                } else {
                    // 普通类选择器：已是完整路径（m() mixin 后缀输出或 e() mixin 字面输出），直接使用。
                    // 注意：此分支允许跨中间 wrapper 层级直接使用 bare selector。
                    // 嵌套 BEM 中间链（如 .el-anchor__list 内的 .el-anchor__item）须通过
                    // RuleBuilder::push 的 child_kids loop 中的 AtRootDirect 单独处理分支补全。
                    clean_sel.to_string()
                };
                crate::__tracing::debug!(
                    target: "sasspile::rule_builder",
                    self_sel = %self.selector,
                    atroot_sel = %selector,
                    resolved = %resolved_selector,
                    n_children = children.len(),
                    "AtRootDirect push"
                );
                // 组合后的父 Rule（仅含自身声明）
                self.result.push(CssNode::Rule {
                    selector: resolved_selector.clone(),
                    declarations,
                    children: vec![],
                });
                // 子节点：AtRootDirect 需展开并组合，Rule 直接嵌套
                for child in children {
                    match child {
                        CssNode::Rule { selector: kid_sel, declarations: kid_decls, children: kid_kids } => {
                            // kid_sel 也可能含尾随逗号——trim 后组合
                            let clean_kid = kid_sel.trim().trim_end_matches(',').trim();
                            let kid_combined = Evaluator::combine_selectors(&resolved_selector, clean_kid);
                            self.result.push(CssNode::Rule {
                                selector: kid_combined,
                                declarations: kid_decls,
                                children: kid_kids,
                            });
                        }
                        CssNode::AtRootDirect(nested_inner) => {
                            // 嵌套 AtRootDirect：用 resolved_selector 作为新的 self.selector 递归处理
                            let orig_selector = self.selector.clone();
                            self.selector = resolved_selector.clone();
                            self.push_atroot_direct(*nested_inner);
                            self.selector = orig_selector;
                        }
                        other => {
                            // 其他节点（Declaration 等）：直接 push
                            self.result.push(other);
                        }
                    }
                }
            }
            other => {
                // 非 Rule 内部节点：直接 push
                self.result.push(other);
            }
        }
    }

    /// 消费构建器，返回最终节点列表。
    ///
    /// 关键语义：@at-root 提升节点紧跟在父规则声明块之后、嵌套子规则之前——
    /// 与 dart-sass 一致（EP 的 BEM mixin 链依赖此顺序）。
    fn build(mut self) -> Vec<CssNode> {
        self.flush_decls();
        match self.result.is_empty() && self.root_nodes.is_empty() {
            true => {
                self.result.push(CssNode::Rule {
                    selector: self.selector,
                    declarations: vec![],
                    children: vec![],
                });
            }
            false => match self.root_nodes.is_empty() {
                true => {}
                false => {
                    // 将 root_nodes 插入到父声明块之后、嵌套子规则之前——
                    // 与 dart-sass 语义一致：@at-root 规则在父声明块后立即输出，
                    // 在所有非-@at-root 嵌套子规则之前。
                    //
                    // 当父规则无声明块时（纯 mixin  mixin 调用），
                    // root_nodes 插入到所有子节点之前。
                    let mut combined: Vec<CssNode> = Vec::new();
                    let mut hoisted = false;
                    for node in self.result {
                        if !hoisted {
                            if let CssNode::Rule { selector, .. } = &node {
                                if selector == &self.selector {
                                    combined.push(node);
                                    combined.extend(self.root_nodes.iter().cloned());
                                    hoisted = true;
                                    continue;
                                }
                            }
                        }
                        combined.push(node);
                    }
                    self.result = match hoisted {
                        true => combined,
                        false => {
                            // 父声明块不存在：root_nodes 插在所有子节点之前
                            let mut all = self.root_nodes;
                            all.extend(combined);
                            all
                        }
                    };
                }
            },
        }
        self.result
    }
}

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
        match is_top_level {
            true => {
                let trimmed = selector.trim_start();
                match trimmed.strip_prefix('&')
                    .and_then(|rest| rest.chars().next())
                {
                    Some(c) if c.is_alphanumeric() || c == '-' => {
                        return Err(SassError::Eval(
                            "A top-level selector may not contain a parent selector with a suffix.".into(),
                        ));
                    }
                    _ => {}
                }
            }
            false => {}
        }

        // & 位置检测：& 必须在 compound selector 开头
        // compound selector 由空格、>、+、~、, 分隔
        // 伪选择器括号内的 & 也是合法的（如 :is(&), :where(&)）
        match selector.contains('&') {
            true => {
                let chars: Vec<char> = selector.chars().collect();
                for (i, &c) in chars.iter().enumerate() {
                    match c == '&' && i > 0 {
                        true => {
                            let prev = chars[i - 1];
                            match prev != ' '
                                && prev != '>'
                                && prev != '+'
                                && prev != '~'
                                && prev != ','
                                && prev != '\t'
                                && prev != '\n'
                                && prev != '('
                            {
                                true => return Err(SassError::Eval(
                                    "\"&\" may only used at the beginning of a compound selector.".into(),
                                )),
                                false => {}
                            }
                        }
                        false => {}
                    }
                }
            }
            false => {}
        }

        // FIX: 解析 selector 中的字面 & 后再存入 env.current_selector，
        // 确保 mixin 内 $selector: & 读取到的是正确解析值（而非字面 &）。
        // RuleBuilder 仍使用原始 selector（保留 & 供后续 combine_selectors 处理）。
        // CRITICAL: trim 尾随逗号——EP 的 e() mixin 输出 "#{$currentSelector}" 含尾随逗号
        //（如 ".el-badge__content,"），如果 current_selector 保留逗号，
        // 后续 m() mixin 的 "$selector: &" 会捕获带逗号的值，
        // 导致 $currentSelector 变成 ".el-badge__content,--primary"（双逗号 Bug）。
        let resolved_sel = if selector.contains('&') {
            let parent_sel = env.get_selector().map(String::from).unwrap_or_default();
            Self::combine_selectors(&parent_sel, selector.as_str())
        } else {
            selector.clone()
        };
        // 归一化 current_selector：trim 尾逗号和空白
        let resolved_sel = resolved_sel.trim().trim_end_matches(',').trim().to_string();

        // FIX: 保存父级上下文（selector + chain），防止嵌套规则修改后泄漏到兄弟节点。
        // BEM NESTING FIX: with_chain 将 resolved_sel 追加到 selector_chain 末尾，
        // 使得 eval_content 捕获的 & 解析包含完整嵌套路径。
        let parent_selector = env.get_selector().map(String::from);
        let parent_chain = env.get_selector_chain().map(String::from);
        tracing::debug!(target: "chain_trace", selector = %resolved_sel, parent_sel = ?parent_selector, parent_chain = ?parent_chain, "eval_rule enter");
        let env = env
            .enter_scope()
            .with_selector(resolved_sel.clone())
            .with_chain(&resolved_sel);
        tracing::debug!(target: "chain_trace", selector = %resolved_sel, body_chain = ?env.get_selector_chain(), "eval_body_chain");
        let (css, new_env) = Self::eval_nodes(body, env)?;

        // Group 1 重构：for + &mut push 替代 fold（递归 dispatch AtRootDirect 需要 &mut self）
        let builder_selector = selector.trim().trim_end_matches(',').trim().to_string();
        let mut builder = RuleBuilder::new(builder_selector);
        builder.resolved_self = resolved_sel.trim().trim_end_matches(',').trim().to_string();
        for node in css {
            builder.push(node);
        }
        let result = builder.build();

        // 退出子作用域——恢复父 scope，传播 !global 和新增 mixin/function
        let return_env = new_env.exit_scope();

        // FIX: 恢复父级 selector 和 chain（嵌套规则内 eval_nodes 可能修改了它们）。
        let return_env =
            return_env.restore_parent_context(parent_selector.as_deref(), parent_chain.as_deref());

        Ok((result, return_env))
    }

    /// 递归检测 CssNode 中是否存在包含字面 `&` 的选择器。
    /// 用于判断 @at-root 子规则是否需要父选择器展开。
    fn selector_contains_ampersand(node: &CssNode) -> bool {
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

    /// 按逗号分割选择器——但**不**分割括号内的逗号（`:not(.a, .b)` 内部逗号不是分隔符）。
    ///
    /// 跟踪圆括号嵌套深度，深度 > 0 时的逗号属于函数参数（如 `:not()`、`:is()`、`:where()`）。
    fn split_selectors_respecting_parens(s: &str) -> Vec<&str> {
        let mut result = Vec::new();
        let mut start = 0;
        let mut depth: i32 = 0;
        for (i, b) in s.bytes().enumerate() {
            match b {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b',' if depth == 0 => {
                    let piece = &s[start..i];
                    let trimmed = piece.trim();
                    if !trimmed.is_empty() {
                        result.push(trimmed);
                    }
                    start = i + 1;
                }
                _ => {}
            }
        }
        let tail = &s[start..];
        let trimmed = tail.trim();
        if !trimmed.is_empty() {
            result.push(trimmed);
        }
        result
    }

    /// 组合选择器——处理 & 替换和逗号分隔选择器。
    pub(crate) fn combine_selectors(parent: &str, child: &str) -> String {
        let parents = Self::split_selectors_respecting_parens(parent);
        let children = Self::split_selectors_respecting_parens(child);

        // 空 parent 或空 child 时直接使用非空的一方
        match parents.is_empty() {
            true => return child.trim().to_string(),
            false => {}
        }
        match children.is_empty() {
            true => return parent.trim().to_string(),
            false => {}
        }

        // 迭代器笛卡尔积——flat_map 保持外层（parent）优先序
        parents
            .iter()
            .flat_map(|p| {
                children.iter().map(move |c| {
                    match (c.contains('&'), p.is_empty()) {
                        (true, _) => c.replace('&', p),
                        (false, true) => c.to_string(),
                        (false, false) => format!("{p} {c}"),
                    }
                })
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// 对顶层 CssNode 列表中仍含 `&` 的选择器做原地解析。
    ///
    /// 与 `nest_rule_in_children` 不同：后者对 ALL 子 Rule 做 descendant combine，
    /// 本函数仅替换选择器中残留的字面 `&`，不对不含 `&` 的选择器添加前缀。
    /// 这防止 e() mixin 生成的 `$selector` 规则（已含完整父路径）被重复前缀。
    pub(crate) fn resolve_ampersand_in_nodes(parent: &str, nodes: &[CssNode]) -> Vec<CssNode> {
        nodes
            .iter()
            .map(|node| match node {
                CssNode::Rule {
                    selector,
                    declarations,
                    children,
                } if selector.contains('&') => CssNode::Rule {
                    selector: Self::combine_selectors(parent, selector),
                    declarations: declarations.clone(),
                    children: children.clone(),
                },
                CssNode::AtRoot(inner, q) => CssNode::AtRoot(
                    Self::resolve_ampersand_in_nodes(parent, inner),
                    q.clone(),
                ),
                CssNode::AtRootDirect(inner) => CssNode::AtRootDirect(Box::new(
                    Self::resolve_ampersand_in_nodes(parent, std::slice::from_ref(inner))
                        .into_iter()
                        .next()
                        .expect("single inner node"),
                )),
                _ => node.clone(),
            })
            .collect()
    }

    /// 将父选择器传播到 children 内的 Rule 子节点——递归展开嵌套 `&`。
    ///
    /// 用于 `a { @at-root { &--x { ... } } }` 场景——`&` 需解析为实际父选择器。
    /// 递归处理 Rules、AtRules、AtRoots 及其子节点，确保所有嵌套层级 `&` 正确展开。
    pub(crate) fn nest_rule_in_children(parent: &str, children: Vec<CssNode>) -> Vec<CssNode> {
        let (result, current_decls) = children.into_iter().fold(
            (Vec::<CssNode>::new(), Vec::<CssNode>::new()),
            |(mut result, mut current_decls), child| match child {
                CssNode::Declaration { .. } => {
                    current_decls.push(child);
                    (result, current_decls)
                }
                CssNode::Rule {
                    selector,
                    declarations,
                    children: rule_children,
                } => {
                    match !current_decls.is_empty() {
                        true => {
                            result.push(CssNode::Rule {
                                selector: parent.to_string(),
                                declarations: std::mem::take(&mut current_decls),
                                children: vec![],
                            });
                        }
                        false => {}
                    }
                    // DOUBLE-PREFIX FIX：检测 child selector 是否已包含 parent 前缀。
                    // 两种场景：
                    // 1. compound 前缀（无空格，如 parent=".a" child=".a--b"）— starts_with_compound_prefix
                    // 2. descendant 前缀（有空格，如 parent=".a--x" child=".a--x .b—y"）— 自定义检测
                    // 场景 2 专门针对 e() mixin 内层 @content 已展开的嵌套链
                    let already_has_prefix = starts_with_compound_prefix(parent, &selector)
                        || selector.starts_with(&format!("{parent} "));
                    let combined = if already_has_prefix {
                        selector.clone()
                    } else {
                        Self::combine_selectors(parent, &selector)
                    };
                    // 递归处理子节点：对仍含 `&` 的子选择器继续展开
                    let processed_kids = Self::nest_rule_in_children(&combined, rule_children);
                    result.push(CssNode::Rule {
                        selector: combined,
                        declarations,
                        children: processed_kids,
                    });
                    (result, current_decls)
                }
                CssNode::AtRule {
                    name,
                    params,
                    children: atrule_children,
                    has_body: true,
                } => {
                    use crate::parse::at_rule_kinds::CssAtRule;
                    match !current_decls.is_empty() {
                        true => {
                            result.push(CssNode::Rule {
                                selector: parent.to_string(),
                                declarations: std::mem::take(&mut current_decls),
                                children: vec![],
                            });
                        }
                        false => {}
                    }
                    let ch = match CssAtRule::is_keyframes(&name) {
                        true => atrule_children,
                        false => Self::nest_rule_in_children(parent, atrule_children),
                    };
                    result.push(CssNode::AtRule {
                        name,
                        params,
                        children: ch,
                        has_body: true,
                    });
                    (result, current_decls)
                }
                // AtRoot：递归处理其内部节点，仍使用当前 parent 解析 `&`
                CssNode::AtRoot(atroot_nodes, query) => {
                    match !current_decls.is_empty() {
                        true => {
                            result.push(CssNode::Rule {
                                selector: parent.to_string(),
                                declarations: std::mem::take(&mut current_decls),
                                children: vec![],
                            });
                        }
                        false => {}
                    }
                    let nested = Self::nest_rule_in_children(parent, atroot_nodes);
                    result.push(CssNode::AtRoot(nested, query));
                    (result, current_decls)
                }
                // AtRootDirect：直接放入结果，不参与 parent 选择器组合
                CssNode::AtRootDirect(inner) => {
                    match !current_decls.is_empty() {
                        true => {
                            result.push(CssNode::Rule {
                                selector: parent.to_string(),
                                declarations: std::mem::take(&mut current_decls),
                                children: vec![],
                            });
                        }
                        false => {}
                    }
                    let processed = Self::nest_rule_in_children(parent, vec![*inner]);
                    for node in processed {
                        result.push(node);
                    }
                    (result, current_decls)
                }
                other => {
                    match !current_decls.is_empty() {
                        true => {
                            result.push(CssNode::Rule {
                                selector: parent.to_string(),
                                declarations: std::mem::take(&mut current_decls),
                                children: vec![],
                            });
                        }
                        false => {}
                    }
                    result.push(other);
                    (result, current_decls)
                }
            },
        );
        match current_decls.is_empty() {
            true => result,
            false => {
                let mut result = result;
                result.push(CssNode::Rule {
                    selector: parent.to_string(),
                    declarations: current_decls,
                    children: vec![],
                });
                result
            }
        }
    }
}
