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
        if !self.current_decls.is_empty() {
            let decls = std::mem::take(&mut self.current_decls);
            self.result.push(CssNode::Rule {
                selector: self.selector.clone(),
                declarations: decls,
                children: vec![],
            });
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
                if without_media || without_supports || without_all || with_rule {
                    // 保留 AtRoot 但嵌套父选择器——由 eval_at_rule 分流提升到 @media 外面
                    let nested = Evaluator::nest_rule_in_children(&self.selector, nodes);
                    self.flush_decls();
                    self.result.push(CssNode::AtRoot(nested, query));
                } else {
                    // 默认行为——脱离父选择器，提升到 root
                    self.root_nodes.extend(nodes);
                }
            }
            CssNode::Rule {
                selector: child_sel,
                declarations: child_decls,
                children: child_kids,
            } => {
                self.flush_decls();
                let combined = Evaluator::combine_selectors(&self.selector, &child_sel);
                if !child_decls.is_empty() {
                    self.result.push(CssNode::Rule {
                        selector: combined.clone(),
                        declarations: child_decls,
                        children: vec![],
                    });
                }
                for kid in child_kids {
                    if let CssNode::Rule {
                        selector: kid_sel,
                        declarations: kid_decls,
                        ..
                    } = kid
                    {
                        let kid_combined = Evaluator::combine_selectors(&combined, &kid_sel);
                        if !kid_decls.is_empty() {
                            self.result.push(CssNode::Rule {
                                selector: kid_combined,
                                declarations: kid_decls,
                                children: vec![],
                            });
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
    fn build(mut self) -> Vec<CssNode> {
        self.flush_decls();
        if self.result.is_empty() && self.root_nodes.is_empty() {
            self.result.push(CssNode::Rule {
                selector: self.selector,
                declarations: vec![],
                children: vec![],
            });
        } else {
            self.result.extend(self.root_nodes);
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
        // 对选择器中的 #{...} 插值求值
        let selector = if selector.contains("#{") {
            crate::eval::value::eval_interp_str(selector, &env)
        } else {
            selector.to_string()
        };

        // 顶层父选择器后缀检测：&a 在无父选择器或 AtRule 上下文时非法
        let is_top_level = env.get_selector().is_none_or(|s| s.starts_with('@'));
        if is_top_level {
            let trimmed = selector.trim_start();
            if let Some(rest) = trimmed.strip_prefix('&')
                && let Some(c) = rest.chars().next()
                && (c.is_alphanumeric() || c == '-')
            {
                return Err(SassError::Eval(
                    "A top-level selector may not contain a parent selector with a suffix.".into(),
                ));
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

        // 检查当前规则是否是 top-level（无父选择器或父选择器是 @at-rule）
        // 用于 plain CSS 模式判断 @media 是否提升
        let is_top_level = env.get_selector().is_none_or(|s| s.starts_with('@'));

        // 进入子作用域——零 clone，parent 指向当前 scope
        let env = env.enter_scope().with_selector(selector.clone());
        let (css, new_env) = Self::eval_nodes(body, env)?;

        // plain CSS 模式——不合并选择器，保留嵌套结构
        if new_env.is_plain_css() {
            let (declarations, children, root_nodes) = css.into_iter().fold(
                (Vec::new(), Vec::new(), Vec::new()),
                |(mut decls, mut kids, mut root), node| {
                    match node {
                        decl @ CssNode::Declaration { .. } => decls.push(decl),
                        CssNode::AtRoot(nodes, _) => root.extend(nodes),
                        CssNode::AtRule {
                            name,
                            params,
                            children,
                            has_body,
                        } if is_top_level
                            && !crate::parse::at_rule_kinds::CssAtRule::is_keyframes(&name) =>
                        {
                            // 提升 AtRule 到外层，将 children 包装在 Rule { selector } 中
                            let wrapped = vec![CssNode::Rule {
                                selector: selector.clone(),
                                declarations: Vec::new(),
                                children,
                            }];
                            root.push(CssNode::AtRule {
                                name,
                                params,
                                children: wrapped,
                                has_body,
                            });
                        }
                        CssNode::AtRule {
                            name,
                            params,
                            children,
                            has_body: true,
                        } if !crate::parse::at_rule_kinds::CssAtRule::is_keyframes(&name) => {
                            // 非提升的 AtRule——plain CSS 中保持原始 children
                            kids.push(CssNode::AtRule {
                                name,
                                params,
                                children,
                                has_body: true,
                            });
                        }
                        other => kids.push(other),
                    }
                    (decls, kids, root)
                },
            );
            let mut result = Vec::new();
            if !declarations.is_empty() || !children.is_empty() {
                result.push(CssNode::Rule {
                    selector: selector.clone(),
                    declarations,
                    children,
                });
            }
            result.extend(root_nodes);
            return Ok((result, new_env));
        }

        // 使用 RuleBuilder + fold 处理嵌套规则
        let result = css
            .into_iter()
            .fold(RuleBuilder::new(selector), RuleBuilder::push)
            .build();

        // 退出子作用域——恢复父 scope，传播 !global 和新增 mixin/function
        let return_env = new_env.exit_scope();

        Ok((result, return_env))
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
        if parents.is_empty() {
            return child.trim().to_string();
        }
        if children.is_empty() {
            return parent.trim().to_string();
        }

        // 迭代器笛卡尔积——flat_map 保持外层（parent）优先序
        parents
            .iter()
            .flat_map(|p| {
                children.iter().map(move |c| {
                    if c.contains('&') {
                        c.replace('&', p)
                    } else if p.is_empty() {
                        c.to_string()
                    } else {
                        format!("{p} {c}")
                    }
                })
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// 将父选择器传播到 `AtRule` children 内的 Rule 子节点。
    ///
    /// 用于 `a {@import "other"}` 场景——被导入文件中的规则需要嵌套在父选择器 `a` 下。
    fn nest_rule_in_children(parent: &str, children: Vec<CssNode>) -> Vec<CssNode> {
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
                    children,
                } => {
                    if !current_decls.is_empty() {
                        result.push(CssNode::Rule {
                            selector: parent.to_string(),
                            declarations: std::mem::take(&mut current_decls),
                            children: vec![],
                        });
                    }
                    let combined = Self::combine_selectors(parent, &selector);
                    result.push(CssNode::Rule {
                        selector: combined,
                        declarations,
                        children,
                    });
                    (result, current_decls)
                }
                CssNode::AtRule {
                    name,
                    params,
                    children,
                    has_body: true,
                } => {
                    use crate::parse::at_rule_kinds::CssAtRule;
                    if !current_decls.is_empty() {
                        result.push(CssNode::Rule {
                            selector: parent.to_string(),
                            declarations: std::mem::take(&mut current_decls),
                            children: vec![],
                        });
                    }
                    let ch = if CssAtRule::is_keyframes(&name) {
                        children
                    } else {
                        Self::nest_rule_in_children(parent, children)
                    };
                    result.push(CssNode::AtRule {
                        name,
                        params,
                        children: ch,
                        has_body: true,
                    });
                    (result, current_decls)
                }
                other => {
                    if !current_decls.is_empty() {
                        result.push(CssNode::Rule {
                            selector: parent.to_string(),
                            declarations: std::mem::take(&mut current_decls),
                            children: vec![],
                        });
                    }
                    result.push(other);
                    (result, current_decls)
                }
            },
        );
        if current_decls.is_empty() {
            result
        } else {
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
