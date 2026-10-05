//! CSS 序列化核心逻辑——节点展平、合并、序列化输出。
//!
//! 从 `mod.rs` 拆分出来，包含：
//! - `merge_at_rules` — 合并相同 query 的 @media/@supports 块
//! - `flatten_nodes` / `flatten_children` — 嵌套规则展平
//! - `serialize_expanded` / `serialize_compressed` — 两种输出格式
//! - `write_node_expanded` / `write_node_compressed` — 单节点写入

use crate::css::node::CssNode;

use super::Serializer;

impl Serializer {
    /// 合并相邻的 @media/@supports 块（相同 query）。
    pub(super) fn merge_at_rules(nodes: Vec<(CssNode, usize)>) -> Vec<(CssNode, usize)> {
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
                    match should_merge {
                        true => match result.pop() {
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
                        },
                        false => result.push((node, gid)),
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
    pub(super) fn flatten_nodes(nodes: &[CssNode], start_group: usize) -> Vec<(CssNode, usize)> {
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
                    match !declarations.is_empty() {
                        true => out.push((
                            CssNode::Rule {
                                selector: selector.clone(),
                                declarations: declarations.clone(),
                                children: vec![],
                            },
                            gid,
                        )),
                        false => {}
                    }
                    let has_non_rule_children =
                        children.iter().any(|c| !matches!(c, CssNode::Rule { .. }));
                    crate::__tracing::debug!(
                        target: "sasspile::flatten",
                        selector = %selector,
                        n_children = children.len(),
                        has_non_rule = has_non_rule_children,
                        child_types = ?children.iter().map(|c| std::mem::discriminant(c)).collect::<Vec<_>>(),
                        "process_node Rule"
                    );
                    let flat = Serializer::flatten_children(selector, children, gid);
                    match has_non_rule_children {
                        true => {
                            let (rule_kids, other_kids): (
                                Vec<(CssNode, usize)>,
                                Vec<(CssNode, usize)>,
                            ) = flat
                                .into_iter()
                                .partition(|(k, _)| matches!(k, CssNode::Rule { .. }));
                            out.extend(rule_kids);
                            match !other_kids.is_empty() {
                                true => out.push((
                                    CssNode::Rule {
                                        selector: selector.clone(),
                                        declarations: vec![],
                                        children: other_kids.into_iter().map(|(n, _)| n).collect(),
                                    },
                                    gid,
                                )),
                                false => {}
                            }
                        }
                        false => out.extend(flat),
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
                // AtRootDirect：来自 mixin @at-root，应作为其内部节点直接展平
                // 保持源码位置，不受父选择器组合影响
                CssNode::AtRootDirect(inner) => {
                    return process_node(inner, state);
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

    pub(super) fn flatten_children(
        parent: &str,
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
                // AtRootDirect：嵌套在 Rule children 中时需展开内部节点并组合父选择器
                // 场景：@at-root mixin (m/e 系列) 生成的 AtRootDirect 嵌套在父 Rule 下
                CssNode::AtRootDirect(inner) => {
                    if let CssNode::Rule { selector, declarations, children: nested } = inner.as_ref() {
                        // 组合父选择器与 AtRootDirect 内部选择器
                        let combined = if selector.contains('&') {
                            selector.replace('&', parent)
                        } else if parent.is_empty() || selector.starts_with(':') {
                            // 空 parent 或以伪类开头 → 直接拼接
                            format!("{parent}{selector}")
                        } else {
                            format!("{parent} {selector}")
                        };
                        let combined = crate::css::selector::sanitize_selector(&combined);
                        crate::__tracing::debug!(
                            target: "sasspile::flatten",
                            parent = %parent,
                            inner_sel = %selector,
                            combined = %combined,
                            "AtRootDirect nested → combined"
                        );
                        let decls: Vec<(CssNode, usize)> = if declarations.is_empty() {
                            Vec::new()
                        } else {
                            vec![(
                                CssNode::Rule {
                                    selector: combined.clone(),
                                    declarations: declarations.clone(),
                                    children: vec![],
                                },
                                group_id,
                            )]
                        };
                        let nested_flat = Self::flatten_children(&combined, nested, group_id);
                        decls.into_iter().chain(nested_flat).collect::<Vec<_>>()
                    } else {
                        // 非 Rule 内部节点：直接传递
                        vec![(child.clone(), group_id)]
                    }
                }
                other => vec![(other.clone(), group_id)],
            })
            .collect()
    }

    pub(super) fn serialize_expanded(nodes: &[(CssNode, usize)], depth: usize) -> String {
        let indent = "  ".repeat(depth);
        let mut result = nodes.iter().enumerate().fold(String::new(), |mut acc, (i, (n, gid))| {
            match i > 0 {
                true => {
                    acc.push('\n');
                    match depth == 0 {
                        true => {
                            let (prev_n, prev_gid) = &nodes[i - 1];
                            let prev_is_import = matches!(prev_n, CssNode::AtRule { name, has_body: false, .. } if name == "import");
                            let curr_is_import = matches!(n, CssNode::AtRule { name, has_body: false, .. } if name == "import");
                            let prev_is_comment = matches!(prev_n, CssNode::Comment(_));
                            let same_group = prev_gid == gid;
                            let same_origin = !same_group && Self::is_same_origin(prev_n, n);
                            match !prev_is_import
                                && !curr_is_import
                                && !prev_is_comment
                                && !same_group
                                && !same_origin {
                                true => acc.push('\n'),
                                false => {}
                            }
                        }
                        false => {}
                    }
                }
                false => {}
            }
            Self::write_node_expanded(&mut acc, n, &indent, depth);
            acc
        });
        match depth == 0 {
            true => result.push('\n'),
            false => {}
        }
        result
    }

    /// 启发式：判断两个顶层兄弟 Rule 是否来自同一 `eval_rule` 输出。
    /// 仅在 `group_id` 不同时使用——检查选择器后代关系（非完全相同）。
    pub(super) fn is_same_origin(prev: &CssNode, curr: &CssNode) -> bool {
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
                match prev_sel == curr_sel {
                    true => return false,
                    false => {}
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

    pub(super) fn serialize_compressed(nodes: &[(CssNode, usize)]) -> String {
        nodes.iter().fold(String::new(), |mut acc, (n, _)| {
            Self::write_node_compressed(&mut acc, n);
            acc
        })
    }

    /// CSS 输出后处理——规范化三种模式：
    /// 1. var() null fallback: `var(--x, null)` → `var(--x, )`（compressed: `var(--x,)`）
    /// 2. appearance 厂商前缀: `appearance: none` → 前插 `-moz-appearance: none`（去重）
    /// 3. :not() 多参包装: `:not(.a, .b)` → `:not(:is(.a, .b))`
    pub(crate) fn normalize_css(css: &str) -> String {
        let is_compressed = !css.contains("\n  ") && !css.contains(": ");
        let css = Self::normalize_var_null(css, is_compressed);
        let css = Self::normalize_moz_appearance(&css);
        Self::normalize_not_multi_arg(&css)
    }

    /// 处理 var() 内的 null fallback：`var(--name, null)` → `var(--name, )`。
    /// 只移除字面量 `null`，保留逗号以维持合法 CSS。
    /// compressed 模式下不包含空格（`var(--name,)`）。
    fn normalize_var_null(css: &str, is_compressed: bool) -> String {
        let mut result = String::with_capacity(css.len());
        let chars: Vec<char> = css.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            // 检测 "var(" 起始 — 快速路径:直接切片 chars 而非分配 String
            if chars[i..].starts_with(&['v', 'a', 'r', '(']) {
                // 找到匹配的闭合括号
                let paren_start = i + 3; // '(' 的位置
                let mut depth = 1;
                let mut j = paren_start + 1;
                while j < chars.len() && depth > 0 {
                    match chars[j] {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }
                // chars[paren_start+1..j-1] 是 var() 内部内容（不含外层括号）
                let inner_start = paren_start + 1;
                let inner_end = j - 1; // exclusive: position of ')'
                let inner = &css[inner_start..inner_end];

                // 在 inner 中查找顶层的 ", null" 并移除
                match Self::remove_null_arg(inner, is_compressed) {
                    Some(new_inner) => {
                        result.push_str("var(");
                        result.push_str(&new_inner);
                        result.push(')');
                        i = j; // 已处理整个 var(...) 表达式
                    }
                    None => {
                        result.push(chars[i]);
                        i += 1;
                    }
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    }

    /// 在 var() 内部查找并移除顶层的 null 参数。
    /// 返回 Some(new_inner) 表示有修改，None 表示无 null 参数。
    fn remove_null_arg(inner: &str, is_compressed: bool) -> Option<String> {
        // 找到顶层逗号分隔的参数中是否有 "null"
        let mut depth = 0;
        let mut arg_start = 0;
        let mut found = false;
        let chars: Vec<char> = inner.chars().collect();

        for (idx, ch) in chars.iter().enumerate() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                ',' if depth == 0 => {
                    let arg = inner[arg_start..idx].trim();
                    if arg == "null" {
                        found = true;
                        break;
                    }
                    arg_start = idx + 1;
                }
                _ => {}
            }
        }
        if !found {
            // 检查最后一个参数
            let arg = inner[arg_start..].trim();
            if arg == "null" {
                found = true;
            }
        }
        if !found {
            return None;
        }

        // 重建 inner，移除 null 参数
        Self::rebuild_without_null(inner, is_compressed)
    }

    /// 重建 inner 字符串，移除所有顶层 null 参数。
    /// compressed 模式下不包含空格（`var(--x,)`）。
    fn rebuild_without_null(inner: &str, is_compressed: bool) -> Option<String> {
        let sep = if is_compressed { "," } else { ", " };
        let mut args: Vec<String> = Vec::new();
        let mut depth = 0;
        let mut start = 0;
        let chars: Vec<char> = inner.chars().collect();

        for (idx, ch) in chars.iter().enumerate() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                ',' if depth == 0 => {
                    let arg = inner[start..idx].trim();
                    if arg != "null" {
                        args.push(arg.to_string());
                    }
                    start = idx + 1;
                }
                _ => {}
            }
        }
        let last = inner[start..].trim();
        if last != "null" && !last.is_empty() {
            args.push(last.to_string());
        }

        // 统计 null 数量
        let null_count = inner.split(|c: char| c == ',').filter(|s| s.trim() == "null").count();
        if null_count == 0 {
            return None;
        }

        // 如果所有参数都是 null，返回空 inner
        if args.is_empty() {
            return Some(String::new());
        }

        // 检查 null 是否位于末尾（最后一个顶层逗号后的内容是 "null"）
        let mut last_comma_before_end = None;
        let mut d = 0;
        for (idx, ch) in chars.iter().enumerate().rev() {
            match ch {
                ')' => d += 1,
                '(' => {
                    if d > 0 { d -= 1; } else { break; }
                }
                ',' if d == 0 => {
                    last_comma_before_end = Some(idx);
                    break;
                }
                _ => {}
            }
        }

        if let Some(comma_idx) = last_comma_before_end {
            let after_comma = inner[comma_idx + 1..].trim();
            if after_comma == "null" {
                // null 在末尾：保留末尾逗号表示 fallback 存在
                // expanded: "args, "  compressed: "args,"
                if is_compressed {
                    Some(format!("{},", args.join(sep)))
                } else {
                    Some(format!("{}, ", args.join(sep)))
                }
            } else {
                Some(args.join(sep))
            }
        } else {
            Some(args.join(sep))
        }
    }

    /// 处理 appearance 厂商前缀：
    /// 当遇到 `appearance: none;`（compressed: `appearance:none;`）时，
    /// 在同一规则块内插入 `-moz-appearance: none;`（如果不存在）。
    fn normalize_moz_appearance(css: &str) -> String {
        let is_compressed = !css.contains("\n  ") && !css.contains(": ");
        let pattern = if is_compressed { "appearance:none" } else { "appearance: none" };
        let injection_full = if is_compressed { "-moz-appearance:none;" } else { "-moz-appearance: none;" };
        let moz_pattern = if is_compressed { "-moz-appearance:none" } else { "-moz-appearance: none" };

        if !css.contains(pattern) || css.contains(moz_pattern) {
            return css.to_string();
        }

        if is_compressed {
            Self::inject_moz_appearance_compressed(css, pattern, injection_full, moz_pattern)
        } else {
            Self::inject_moz_appearance_expanded(css, pattern, injection_full, moz_pattern)
        }
    }

    /// compressed 模式：在花括号块内找到 appearance:none 并前插 -moz-appearance:none;
    fn inject_moz_appearance_compressed(css: &str, pattern: &str, injection: &str, moz_pat: &str) -> String {
        let pat_chars: Vec<char> = pattern.chars().collect();
        let mut result = String::with_capacity(css.len() + 30);
        let chars: Vec<char> = css.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if i + pat_chars.len() <= chars.len() && chars[i..i + pat_chars.len()] == pat_chars[..] {
                // 检查花括号块内是否已有 -moz-appearance
                let block_start = css[..i].rfind('{').unwrap_or(0);
                let block = &css[block_start..i];
                if !block.contains(moz_pat) {
                    result.push_str(injection);
                }
                result.push_str(pattern);
                i += pat_chars.len();
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    }

    /// expanded 模式：逐行查找 appearance: none; 并前插带缩进的 -moz-appearance: none;
    fn inject_moz_appearance_expanded(css: &str, _pattern: &str, injection: &str, _moz_pat: &str) -> String {
        let mut result = String::with_capacity(css.len() + 50);
        let lines: Vec<&str> = css.lines().collect();
        let trailing_newline = css.ends_with('\n');

        for (idx, line) in lines.iter().enumerate() {
            if line.trim() == "appearance: none;" {
                // 提取缩进
                let indent_len = line.len() - line.trim_start_matches(' ').len();
                let indent = &line[..indent_len];
                result.push_str(&format!("{indent}{injection}\n"));
            }
            result.push_str(line);
            if idx < lines.len() - 1 {
                result.push('\n');
            } else if trailing_newline {
                result.push('\n');
            }
        }
        result
    }

    /// 处理 :not() 多参数包装：
    /// `:not(.a, .b)` → `:not(:is(.a, .b))`（>=2 个顶层逗号分隔参数时触发）。
    fn normalize_not_multi_arg(css: &str) -> String {
        let mut result = String::with_capacity(css.len());
        // 使用 (byte_offset) 追踪每个 char 的字节位置，避免 UTF-8 char/byte 索引混淆
        let char_byte: Vec<usize> = css.char_indices().map(|(off, _)| off).collect();
        let chars: Vec<char> = css.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // 快速路径:直接切片 chars 而非每字符分配 O(n) String
            if chars[i..].starts_with(&[':', 'n', 'o', 't', '(']) {
                // 使用 char 索引追踪括号匹配
                let paren_char_start = i + 4; // '(' 位置(按 char 计)
                let mut depth = 1;
                let mut j = paren_char_start + 1;
                while j < chars.len() && depth > 0 {
                    match chars[j] {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }
                // 将 char 索引转为字节偏移后再切片
                let byte_inner_start = char_byte[paren_char_start + 1];
                let byte_inner_end = char_byte[j - 1];
                let inner = &css[byte_inner_start..byte_inner_end];

                // 统计顶层逗号数量
                let top_level_commas = {
                    let mut count = 0;
                    let mut d = 0;
                    for ch in inner.chars() {
                        match ch {
                            '(' => d += 1,
                            ')' => d -= 1,
                            ',' if d == 0 => count += 1,
                            _ => {}
                        }
                    }
                    count
                };

                if top_level_commas >= 1 && !inner.contains(":is(") {
                    // 包装为 :is(...)
                    result.push_str(":not(:is(");
                    result.push_str(inner);
                    result.push_str("))");
                } else {
                    result.push_str(":not(");
                    result.push_str(inner);
                    result.push(')');
                }
                i = j;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    }
}
