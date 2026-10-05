//! —— 选择器组合纯函数 ——
//!
//! 将 `&` 替换、descendant combine、compound 前缀检测等纯函数集中在此，
//! 供 `rule.rs` / `rule_builder.rs` / `mixin.rs` 共享。

use crate::css::node::CssNode;
use crate::parse::at_rule_kinds::CssAtRule;

/// 判断 child selector 是否已经以 parent 作为 compound 前缀。
///
/// 检测 `parent=".a"` + `child=".a__b"/".a--b"/".a.b"` 等形式（无分隔空格）。
///
/// **separator 精确化**（EP 修复）：仅 `--` 和 `__` 双字符触发 compound 匹配。
/// 单 `-` / 单 `_` 不再视为 BEM 分隔符——这些由 `#{& + '-suffix'}` interpolation
/// 产出的选择器不是 compound 变体，应走 descendant combine。
/// `.` `:` `#` `[` `>` `+` `~` 视为 structural selector 分隔符（单字符即可）。
pub fn starts_with_compound_prefix(parent: &str, child: &str) -> bool {
    if parent.is_empty() || child == parent {
        return !parent.is_empty();
    }
    if child.len() <= parent.len() || !child.starts_with(parent) {
        return false;
    }
    let bytes = child.as_bytes();
    let sep_pos = parent.len();
    let next_char = bytes[sep_pos];

    // Structural 分隔符（单字符）：.:#[>+~
    if matches!(next_char, b'.' | b':' | b'#' | b'[' | b'>' | b'+' | b'~') {
        return true;
    }

    // BEM 分隔符（需双字符精确匹配）：-- 和 __
    if sep_pos + 1 < bytes.len() {
        if next_char == b'-' && bytes[sep_pos + 1] == b'-' {
            return true;
        }
        if next_char == b'_' && bytes[sep_pos + 1] == b'_' {
            return true;
        }
    }

    false
}

/// 判断 child 是否已包含 parent 前缀（compound 或 descendant 两种模式）。
///
/// 1. compound: `parent=".a"` + `child=".a--b"` — `starts_with_compound_prefix`
/// 2. descendant: `parent=".a--x"` + `child=".a--x .b—y"` — 空格分隔前缀
/// 3. 嵌套链包含: `parent=".a__b"` + `child=".x .a__b:first-child"` —
///    parent 出现在 child 的 descendant chain 中间（如 @content 内 eval 展开后）。
///
/// BOUNDARY FIX：contains 匹配必须验证 parent 后面紧跟合法边界字符
///（空格/>/+～/: 或 .[ 或字符串结尾），防止 `.el-select-dropdown` 被误判为
/// `.el-scrollbar.is-empty .el-select-dropdown__list` 的前缀（后者 __list 后缀表明
/// `.el-select-dropdown` 是一个更长类名的前缀，而非同一选择器）。
///
/// MULTI-PARENT FIX：当 parent 含逗号（多个独立选择器，如 `.a, .b`）时，
/// child 可能每个段都已经包含对应 parent（如 `.a > .x, .b > .x` 包含 `.a, .b`）。
/// 逐段检查防止 Cartesian 重复展开。
pub fn has_descendant_prefix(parent: &str, child: &str) -> bool {
    if parent.is_empty() || child.is_empty() {
        return false;
    }

    // 合法后继字符——selector 段边界。
    let valid_suffix = |c: char| matches!(c, ' ' | '>' | '+' | '~' | ',' | ':' | '.' | '[' | ']');

    // MULTI-PARENT：拆分 parent 段，逐段检查是否都在 child 中
    let parent_parts = split_selectors_respecting_parens(parent);
    if parent_parts.len() > 1 {
        // 检查每个 parent 段是否都出现在 child 中（带边界检查）
        let all_present = parent_parts.iter().all(|&pp| {
            if pp.is_empty() {
                return true;
            }
            // child starts with parent segment (compound)或 child 包含 parent 作为独立段
            child_starts_with_boundary(child, pp, valid_suffix)
                || child_contains_boundary(child, pp, valid_suffix)
        });
        if all_present {
            return true;
        }
    }

    if starts_with_compound_prefix(parent, child) {
        return true;
    }
    if child.starts_with(&format!("{parent} ")) {
        return true;
    }

    // 检查 " {parent}" 模式（descendant 链中间出现）
    for (offset, _) in child.match_indices(&format!(" {parent}")) {
        let after_parent = offset + 1 + parent.len(); // 1 是 leading space
        if after_parent >= child.len() {
            return true; // parent 在 child 尾部边界
        }
        if let Some(next_char) = child[after_parent..].chars().next() {
            if valid_suffix(next_char) {
                return true;
            }
        }
    }

    // 检查 "{parent}:" 和 "{parent}." 模式
    for suffix in [":", "."] {
        let formatted = format!("{parent}{suffix}");
        if let Some(offset) = child.find(&formatted) {
            let after_parent = offset + formatted.len();
            if after_parent >= child.len() || child[after_parent..].chars().next().is_some_and(valid_suffix) {
                return true;
            }
        }
    }
    false
}

/// 检查 child 是否以 parent segment 开头（段边界匹配）。
fn child_starts_with_boundary(child: &str, parent: &str, valid_suffix: fn(char) -> bool) -> bool {
    if child.len() < parent.len() {
        return false;
    }
    // 提取 child 的第一个逗号分隔段（括号安全）
    let child_first = split_selectors_respecting_parens(child)
        .first()
        .copied()
        .unwrap_or(child);
    if child_first.starts_with(parent) {
        if child_first.len() == parent.len() {
            return true; // 精确匹配
        }
        return child_first[parent.len()..]
            .chars()
            .next()
            .is_some_and(valid_suffix);
    }
    false
}

/// 检查 child 是否包含 parent segment 作为独立段（前后都是合法边界）。
fn child_contains_boundary(child: &str, parent: &str, valid_suffix: fn(char) -> bool) -> bool {
    let search = format!(" {parent}");
    for (offset, _) in child.match_indices(&search) {
        let after = offset + search.len();
        if after >= child.len() || child[after..].chars().next().is_some_and(valid_suffix) {
            return true;
        }
    }
    false
}

/// 按逗号分割选择器——但**不**分割括号内的逗号（`:not(.a, .b)` 内部逗号不是分隔符）。
///
/// 跟踪圆括号嵌套深度，深度 > 0 时的逗号属于函数参数（如 `:not()`、`:is()`、`:where()`）。
pub fn split_selectors_respecting_parens(s: &str) -> Vec<&str> {
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
///
/// 对 parent/child 的每个逗号分隔段做笛卡尔积：
/// - child 含 `&` → 用 parent 替换 `&`
/// - child 不含 `&` 且 parent 为空 → 使用 child
/// - child 不含 `&` 且 parent 非空 → descendant combine（"{parent} {child}"）
pub fn combine_selectors(parent: &str, child: &str) -> String {
    let parents = split_selectors_respecting_parens(parent);
    let children = split_selectors_respecting_parens(child);

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
            children.iter().map(move |c| match (c.contains('&'), p.is_empty()) {
                (true, _) => c.replace('&', p),
                (false, true) => c.to_string(),
                (false, false) => format!("{p} {c}"),
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
                selector: combine_selectors(parent, selector),
                declarations: declarations.clone(),
                children: children.clone(),
            },
            CssNode::AtRoot(inner, q) => {
                CssNode::AtRoot(resolve_ampersand_in_nodes(parent, inner), q.clone())
            }
            CssNode::AtRootDirect(inner) => CssNode::AtRootDirect(Box::new(
                resolve_ampersand_in_nodes(parent, std::slice::from_ref(inner))
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
                if !current_decls.is_empty() {
                    result.push(CssNode::Rule {
                        selector: parent.to_string(),
                        declarations: std::mem::take(&mut current_decls),
                        children: vec![],
                    });
                }
                // DOUBLE-PREFIX FIX：检测 child selector 是否已包含 parent 前缀。
                let already_has_prefix = has_descendant_prefix(parent, &selector);
                let combined = if already_has_prefix {
                    selector.clone()
                } else {
                    combine_selectors(parent, &selector)
                };
                // 递归处理子节点：对仍含 `&` 的子选择器继续展开
                let processed_kids = nest_rule_in_children(&combined, rule_children);
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
                if !current_decls.is_empty() {
                    result.push(CssNode::Rule {
                        selector: parent.to_string(),
                        declarations: std::mem::take(&mut current_decls),
                        children: vec![],
                    });
                }
                let ch = match CssAtRule::is_keyframes(&name) {
                    true => atrule_children,
                    false => nest_rule_in_children(parent, atrule_children),
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
                if !current_decls.is_empty() {
                    result.push(CssNode::Rule {
                        selector: parent.to_string(),
                        declarations: std::mem::take(&mut current_decls),
                        children: vec![],
                    });
                }
                let nested = nest_rule_in_children(parent, atroot_nodes);
                result.push(CssNode::AtRoot(nested, query));
                (result, current_decls)
            }
            // AtRootDirect：直接放入结果，不参与 parent 选择器组合
            CssNode::AtRootDirect(inner) => {
                if !current_decls.is_empty() {
                    result.push(CssNode::Rule {
                        selector: parent.to_string(),
                        declarations: std::mem::take(&mut current_decls),
                        children: vec![],
                    });
                }
                let processed = nest_rule_in_children(parent, vec![*inner]);
                for node in processed {
                    result.push(node);
                }
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
