//! dispatch_pass: scan_map reducer
//!
//! 消费 &mut CompileState + token → Vec<String>
//!   - 变量定义: 存 state, 输出 []
//!   - @mixin: 存 state, 输出 []
//!   - @include: 展开 mixin body, 输出展开行
//!   - @for/@each: 展开, 输出展开行
//!   - 选择器规则: 选择器嵌套 + 变量替换
//!   - 普通声明/注释: 变量替换后输出

use tracing::info_span;

use super::parse::{expand_mixin, parse_each_sig, parse_for_sig, parse_include_sig, parse_mixin_sig, substitute_vars, try_parse_var_def};
use super::state::{Collecting, CompileState, MixinDef};

pub fn dispatch_pass(state: &mut CompileState, token: String) -> Vec<String> {
    let _span = info_span!("dispatch_pass", token = %token).entered();
    let t = token.trim();

    // ── 变量定义检测 ──────────────────────────────────────────────────
    if let Some((var_name, var_value)) = try_parse_var_def(t) {
        state.set_variable(var_name, var_value);
        return vec![];
    }

    // ── 闭合 brace: 收集态 vs 选择器栈 ────────────────────────────
    if t == "}" {
        if state.collecting != Collecting::None {
            let result = finalize_collecting(state);
            state.nesting_depth = state.nesting_depth.saturating_sub(1);
            return result;
        }
        if !state.selector_stack.is_empty() {
            state.selector_stack.pop();
            state.nesting_depth = state.nesting_depth.saturating_sub(1);
        }
        return vec!["}".to_string()];
    }

    // ── `} @else {` 同行: 分成两步 — 先收尾 If, 再启动 @else ────
    if state.collecting.is_if_branch() && t.starts_with('}') && t.contains("else") {
        let result = finalize_collecting(state);
        state.nesting_depth = state.nesting_depth.saturating_sub(1);
        // finalize 已设 pending_if_taken, 翻转启动新 branch
        if let Some(prev_taken) = state.pending_if_taken.take() {
            let new_taken = !prev_taken;
            state.nesting_depth += 1;
            state.collecting = Collecting::If { body: vec![], branch_taken: new_taken };
        }
        return result;
    }

    // ── 收集态: 存储 body 行 ─────────────────────────────────────────
    if state.collecting != Collecting::None {
        match &mut state.collecting {
            Collecting::MixinDef { params: _ } => {
                state.nesting_depth += 1;
                if !t.is_empty() {
                    if let Some(mixin_name) = &state.current_mixin_name {
                        if let Some(mixin_def) = state.scope.mixins.get_mut(mixin_name) {
                            mixin_def.body.push(t.to_string());
                        }
                    }
                }
            }
            Collecting::For { body, .. } => {
                state.nesting_depth += 1;
                if !t.is_empty() { body.push(t.to_string()); }
            }
            Collecting::Each { body, .. } => {
                state.nesting_depth += 1;
                if !t.is_empty() { body.push(t.to_string()); }
            }
            Collecting::If { body, .. } => {
                state.nesting_depth += 1;
                if !t.is_empty() { body.push(t.to_string()); }
            }
            Collecting::None => {}
        }
        return vec![];
    }

    // ── @each 单行展开 ───────────────────────────────────────────────
    if t.starts_with("@each ") && t.contains('{') && t.contains('}') {
        return parse_each_sig(&t[6..])
            .and_then(|(var_name, items)| {
                let body_start = t.find('{')?;
                let body_end = t.rfind('}')?;
                if body_end <= body_start { return None; }
                let body = &t[body_start..=body_end];
                Some(items.into_iter().map(move |item| body.replace(&var_name, &item)).collect())
            })
            .map(|lines: Vec<String>| lines.into_iter().map(|l| substitute_vars(state, &l)).collect())
            .unwrap_or_default();
    }

    // ── @for 单行展开 ────────────────────────────────────────────────
    if t.starts_with("@for ") && t.contains('{') && t.contains('}') {
        return parse_for_sig(&t[5..])
            .and_then(|(var_name, values)| {
                let body_start = t.find('{')?;
                let body_end = t.rfind('}')?;
                if body_end <= body_start { return None; }
                let body = &t[body_start..=body_end];
                Some(values.into_iter().map(move |v| body.replace(&var_name, &v)).collect())
            })
            .map(|lines: Vec<String>| lines.into_iter().map(|l| substitute_vars(state, &l)).collect())
            .unwrap_or_default();
    }

    // ── @each 多行起始 ───────────────────────────────────────────────
    if t.starts_with("@each ") && t.contains('{') {
        return parse_each_sig(&t[6..])
            .map(|(var_name, items)| {
                state.nesting_depth += 1;
                state.collecting = Collecting::Each { var_name, items, body: vec![] };
                vec![]
            })
            .unwrap_or_default();
    }

    // ── @for 多行起始 ────────────────────────────────────────────────
    if t.starts_with("@for ") && t.contains('{') {
        return parse_for_sig(&t[5..])
            .map(|(var_name, values)| {
                state.nesting_depth += 1;
                state.collecting = Collecting::For { var_name, values, body: vec![] };
                vec![]
            })
            .unwrap_or_default();
    }

    // ── @mixin 定义 ──────────────────────────────────────────────────
    if t.starts_with("@mixin ") {
        if let Some((name, params)) = parse_mixin_sig(&t[7..]) {
            if let Some(body_start) = t.find('{') {
                if let Some(body_end) = t.rfind('}') {
                    if body_end > body_start {
                        let body = t[body_start..=body_end].to_string();
                        state.scope.mixins.insert(name, MixinDef { params, body: vec![body] });
                        return vec![];
                    }
                }
            }
            state.current_mixin_name = Some(name.clone());
            state.collecting = Collecting::MixinDef { params };
            state.scope.mixins.insert(name, MixinDef { params: vec![], body: vec![] });
            state.nesting_depth += 1;
        }
        return vec![];
    }

    // ── @include 展开 ────────────────────────────────────────────────
    if t.starts_with("@include ") {
        let (name, args) = parse_include_sig(&t[9..]);
        return state.scope.mixins.get(&name).map(|mixin_def| {
            let expanded = expand_mixin(mixin_def, &args);
            expanded.into_iter()
                .flat_map(|line| expand_single_line_body(line, state))
                .collect::<Vec<_>>()
        }).unwrap_or_default();
    }

    // ── @use/@forward 跳过 ──────────────────────────────────────────
    if t.starts_with("@use ") || t.starts_with("@forward ") {
        return vec![];
    }

    // ── @if 多行块起始 ──────────────────────────────────────────────
    if t.starts_with("@if ") && t.contains('{') {
        let expr = extract_if_condition(t);
        let branch_taken = eval_condition(state, &expr);
        state.nesting_depth += 1;
        state.collecting = Collecting::If { body: vec![], branch_taken };
        return vec![];
    }

    // ── @else 分支 (支持 `} @else {` 同行格式) ─────────────────────
    if t.contains('@') && t.contains("else") {
        // 如果当前仍在 collecting If, 先收尾
        if state.collecting.is_if_branch() {
            let result = finalize_collecting(state);
            state.nesting_depth = state.nesting_depth.saturating_sub(1);
            // result 落到下方 pending_if_taken 处理
            if let Some(prev_taken) = state.pending_if_taken.take() {
                let new_taken = !prev_taken;
                state.nesting_depth += 1;
                state.collecting = Collecting::If { body: vec![], branch_taken: new_taken };
            }
            return result;
        }
        // 如果 pending_if_taken 已就绪 (前一个 } 已收尾)
        if let Some(prev_taken) = state.pending_if_taken.take() {
            let new_taken = !prev_taken;
            state.nesting_depth += 1;
            state.collecting = Collecting::If { body: vec![], branch_taken: new_taken };
        }
        return vec![];
    }

    // ── 规则块: 定位 body `{` (跳过 `#{}` 插值) ─────────────────────
    let block_brace = find_block_brace(t);

    if !t.starts_with('@') && block_brace.is_some() {
        let brace_pos = block_brace.unwrap();
        let selector_part = t[..brace_pos].trim();
        let parent = state.selector_stack.last().map(|s| s.as_str());
        let full_selector = match parent {
            Some(ref p) if selector_part.contains('&') => {
                substitute_vars(state, &selector_part.replace('&', p))
            }
            Some(ref p) => {
                substitute_vars(state, &format!("{p} {selector_part}"))
            }
            None => substitute_vars(state, selector_part),
        };
        let brace_part = substitute_vars(state, &t[brace_pos..]);
        let expanded = format!("{full_selector}{brace_part}");
        state.selector_stack.push(full_selector);
        state.nesting_depth += 1;
        return vec![expanded];
    }

    // ── 普通行: 变量替换后输出 ──────────────────────────────────────
    vec![substitute_vars(state, &token)]
}

fn find_block_brace(t: &str) -> Option<usize> {
    let mut search_start = 0;
    while let Some(pos) = t[search_start..].find('{') {
        let abs_pos = search_start + pos;
        if abs_pos > 0 && t.as_bytes().get(abs_pos - 1) == Some(&b' ') {
            return Some(abs_pos);
        }
        search_start = abs_pos + 1;
    }
    None
}

/// 将 `{ prop: val; prop2: val2; }` 单行 body 展平为独立的声明行
fn expand_single_line_body(line: String, state: &CompileState) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let inner = &trimmed[1..trimmed.len()-1];
        inner.split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| substitute_vars(state, s))
            .map(|s| format!("{s};"))
            .collect()
    } else {
        vec![substitute_vars(state, &line)]
    }
}

fn finalize_collecting(state: &mut CompileState) -> Vec<String> {
    let collecting = std::mem::take(&mut state.collecting);
    match collecting {
        Collecting::Each { var_name, items, body } => {
            let mut output = Vec::new();
            let var_name = &var_name;
            for item in items {
                for body_line in &body {
                    output.push(body_line.replace(var_name, &item));
                }
            }
            output
        }
        Collecting::For { var_name, values, body } => {
            let mut output = Vec::new();
            let var_name = &var_name;
            for v in values {
                for body_line in &body {
                    output.push(body_line.replace(var_name, &v));
                }
            }
            output
        }
        Collecting::If { body, branch_taken } => {
            state.pending_if_taken = Some(branch_taken);
            if branch_taken { body } else { vec![] }
        }
        Collecting::MixinDef { params } => {
            if let Some(name) = &state.current_mixin_name {
                if let Some(mixin_def) = state.scope.mixins.get_mut(name) {
                    mixin_def.params = params;
                }
            }
            state.current_mixin_name = None;
            vec![]
        }
        Collecting::None => vec![],
    }
}

/// 从 `@if expr {` 行提取条件表达式 (去掉前缀 `@if ` 与尾部 `{`)
fn extract_if_condition(line: &str) -> String {
    let after_if = line.trim().trim_start_matches("@if").trim();
    let expr = if let Some(brace) = after_if.rfind('{') {
        after_if[..brace].trim()
    } else {
        after_if
    };
    expr.to_string()
}

/// 简易条件求值 — 支持字面量 true/false、null、$var 引用、== / != 比较、and / or / not
/// Phase D.1 范围: 不处理算术, 仅逻辑与真值判断
fn eval_condition(state: &CompileState, expr: &str) -> bool {
    let expr = substitute_vars(state, expr);
    let t = expr.trim();

    if t.is_empty() { return false; }

    // not
    if let Some(rest) = t.strip_prefix("not ") {
        return !eval_condition(state, rest);
    }

    // 字面量
    if t == "false" || t == "null" || t == "0" { return false; }
    if t == "true" { return true; }

    // or
    if let Some(idx) = t.find(" or ") {
        let (left, right) = t.split_at(idx);
        return eval_condition(state, left) || eval_condition(state, &right[4..]);
    }

    // and
    if let Some(idx) = t.find(" and ") {
        let (left, right) = t.split_at(idx);
        return eval_condition(state, left) && eval_condition(state, &right[5..]);
    }

    // == / !=
    for sep in ["==", "!="] {
        if let Some(idx) = t.find(sep) {
            let (left, right) = t.split_at(idx);
            let left = left.trim();
            let right = right[sep.len()..].trim().trim_matches('"').trim();
            let left_trim = left.trim_matches('"');
            let eq = left_trim == right;
            return if sep == "=="{ eq } else { !eq };
        }
    }

    // 引用求真: $var 取值; 数组/Map等非空即真
    if t.starts_with('$') {
        match state.scope.variables.get(t) {
            None => false,
            Some(v) => v != "false" && v != "null" && !v.is_empty(),
        }
    } else {
        // 未解析标识符默认不真
        false
    }
}
