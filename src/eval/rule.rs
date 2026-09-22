use super::*;
use crate::css::node::CssNode;
use crate::error::Result;

/// 规则构建器——封装 `eval_rule` 的 3 个累积器状态。
///
/// `result` 是最终输出节点列表，`current_decls` 是当前累积的声明，
/// `root_nodes` 是 @at-root 提升的节点。
struct RuleBuilder {
    selector: String,
    result: Vec<CssNode>,
    current_decls: Vec<CssNode>,
    root_nodes: Vec<CssNode>,
}

impl RuleBuilder {
    fn new(selector: String) -> Self {
        Self {
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

    /// push 一个 CSS 节点到构建器。
    fn push(mut self, node: CssNode) -> Self {
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
            // 不含 & 时直接 push 不组合，但子节点需与 self.selector 嵌套（EP BEM 链 m()→e() 语义）。
            CssNode::AtRootDirect(inner) => {
                self.flush_decls();
                if let CssNode::Rule { selector, declarations, children } = *inner {
                    let resolved_selector = if selector.contains('&') {
                        Evaluator::combine_selectors(&self.selector, &selector)
                    } else {
                        selector
                    };
                    // 子节点需与 resolved_selector 嵌套——combine child selectors with parent
                    let nested_children: Vec<CssNode> = children
                        .into_iter()
                        .map(|child| match child {
                            CssNode::Rule {
                                selector: ref kid_sel,
                                declarations: ref kid_decls,
                                children: ref kid_kids,
                            } => {
                                let kid_combined = Evaluator::combine_selectors(&resolved_selector, kid_sel);
                                CssNode::Rule {
                                    selector: kid_combined,
                                    declarations: kid_decls.clone(),
                                    children: kid_kids.clone(),
                                }
                            }
                            other => other,
                        })
                        .collect();
                    self.result.push(CssNode::Rule {
                        selector: resolved_selector,
                        declarations,
                        children: nested_children,
                    });
                } else {
                    self.result.push(*inner);
                }
            }
            CssNode::Rule {
                selector: child_sel,
                declarations: child_decls,
                children: child_kids,
            } => {
                self.flush_decls();
                let combined = Evaluator::combine_selectors(&self.selector, &child_sel);
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
                    } else {
                        self.result.push(kid);
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
        self
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

        // 进入子作用域——零 clone，parent 指向当前 scope
        let env = env.enter_scope().with_selector(selector.clone());
        let (css, new_env) = Self::eval_nodes(body, env)?;

        // 使用 RuleBuilder + fold 处理嵌套规则
        let result = css
            .into_iter()
            .fold(RuleBuilder::new(selector), RuleBuilder::push)
            .build();

        // 退出子作用域——恢复父 scope，传播 !global 和新增 mixin/function
        let return_env = new_env.exit_scope();

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

    /// 组合选择器——处理 & 替换和逗号分隔选择器。
    pub(crate) fn combine_selectors(parent: &str, child: &str) -> String {
        let parents: Vec<&str> = parent
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        let children: Vec<&str> = child
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

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
                    let combined = Self::combine_selectors(parent, &selector);
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
