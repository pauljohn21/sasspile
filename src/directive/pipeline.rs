//! sa
//! 统一响应式管线 — rxrust 1.0.0-rc.5
//!
//! 核心洞察: SCSS 编译是多线程有序组合 (composition)
//!   flat_map + Shared::from_stream = 多线程有序组合
//!
//! 管线 = 算子链, 每个算子消费上游、产出下游:
//!   from_stream → scan_map → flat_map → scan_map → flat_map → collect → last → subscribe

use super::state::{Collecting, CompileState, MixinDef};
use crate::css::{CssBuilder, render_node};
use rxrust::prelude::*;
use tracing::info_span;

// ═══════════════════════════════════════════════════════════════════════════
// 解析辅助 (纯函数, 无副作用)
// ═══════════════════════════════════════════════════════════════════════════

fn parse_mixin_sig(s: &str) -> Option<(String, Vec<(String, Option<String>)>)> {
    let (name, rest) = s.split_once('(')?;
    let name = name.trim().to_string();
    let params_end = rest.find(')')?;
    let params_str = &rest[..params_end].trim();
    let params = if params_str.is_empty() {
        vec![]
    } else {
        params_str.split(',')
            .map(|param| {
                let p = param.trim();
                p.split_once(':')
                    .map(|(n, d)| (n.trim().to_string(), Some(d.trim().to_string())))
                    .unwrap_or_else(|| (p.to_string(), None))
            })
            .collect()
    };
    Some((name, params))
}

fn parse_for_sig(s: &str) -> Option<(String, Vec<String>)> {
    let s = s.trim();
    let (_, rest) = s.split_once('$')?;
    let rest = rest.trim();
    let (var_name, rest) = rest.split_once(' ')?;
    let var_name = format!("${}", var_name.trim());
    let rest = rest.trim().strip_prefix("from")?.trim();
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() < 3 { return None; }

    let from: i64 = parts[0].trim_end_matches(|c: char| c == '{' || c == '}' || c == ';' || c == ')').parse().ok()?;
    let to: i64 = parts[2].trim_end_matches(|c: char| c == '{' || c == '}' || c == ';' || c == ')').parse().ok()?;

    let inclusive = parts[1] == "through";
    Some((var_name, generate_range(from, to, inclusive)))
}

fn generate_range(from: i64, to: i64, inclusive: bool) -> Vec<String> {
    let mut values = Vec::new();
    if from <= to {
        let end = if inclusive { to + 1 } else { to };
        for i in from..end {
            values.push(i.to_string());
        }
    } else {
        let end = if inclusive { to - 1 } else { to };
        let mut i = from;
        while i > end {
            values.push(i.to_string());
            i -= 1;
        }
    }
    values
}

fn parse_each_sig(s: &str) -> Option<(String, Vec<String>)> {
    let s = s.trim();
    let (_, rest) = s.split_once('$')?;
    let rest = rest.trim();
    let (var_name, rest) = rest.split_once(' ')?;
    let var_name = format!("${}", var_name.trim());
    let rest = rest.trim().strip_prefix("in")?.trim();
    let rest = rest
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(|c: char| c == ')' || c == '{' || c == '}')
        .trim();
    let items: Vec<String> = rest
        .split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect();
    if items.is_empty() {
        return None;
    }
    Some((var_name, items))
}

fn parse_include_sig(s: &str) -> (String, Vec<String>) {
    let s = s.trim().trim_end_matches(';').trim();
    match s.split_once('(') {
        Some((name, rest)) => {
            let name = name.trim().to_string();
            let rest = rest.trim().trim_end_matches(|c: char| c == ')' || c == ';' || c == '}').trim();
            let args = if rest.is_empty() {
                vec![]
            } else {
                rest.split(',').map(|a| a.trim().to_string()).collect()
            };
            (name, args)
        }
        None => (s.trim().to_string(), vec![]),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// finalize — 闭合 block 时展开 body
// ═══════════════════════════════════════════════════════════════════════════

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

fn expand_mixin(mixin_def: &MixinDef, args: &[String]) -> Vec<String> {
    let defaults: Vec<Option<&str>> = mixin_def.params.iter()
        .map(|(_, d)| d.as_deref()).collect();
    mixin_def.body.iter().map(|body_token| {
        mixin_def.params.iter().enumerate().fold(body_token.clone(), |mut acc, (i, (p_name, _))| {
            let replacement = args.get(i)
                .map(|s| s.as_str())
                .or_else(|| defaults.get(i).copied().flatten());
            if let Some(r) = replacement {
                acc = acc.replace(p_name, r);
            }
            acc
        })
    }).collect()
}

fn dispatch_pass(state: &mut CompileState, token: String) -> Vec<String> {
    let _span = info_span!("dispatch_pass", phase = ?state.phase, token = %token).entered();
    let t = token.trim();

    if t == "}" && state.collecting == Collecting::None && !state.selector_stack.is_empty() {
        state.selector_stack.pop();
        state.nesting_depth = state.nesting_depth.saturating_sub(1);
        return vec!["}".to_string()];
    }

    if state.collecting != Collecting::None && t == "}" {
        let result = finalize_collecting(state);
        state.nesting_depth = state.nesting_depth.saturating_sub(1);
        return result;
    }

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

    if t.starts_with("@each ") && t.contains('{') && t.contains('}') {
        return parse_each_sig(&t[6..])
            .and_then(|(var_name, items)| {
                let body_start = t.find('{')?;
                let body_end = t.rfind('}')?;
                if body_end <= body_start { return None; }
                let body = &t[body_start..=body_end];
                Some(items.into_iter().map(move |item| body.replace(&var_name, &item)).collect())
            })
            .unwrap_or_default();
    }

    if t.starts_with("@for ") && t.contains('{') && t.contains('}') {
        return parse_for_sig(&t[5..])
            .and_then(|(var_name, values)| {
                let body_start = t.find('{')?;
                let body_end = t.rfind('}')?;
                if body_end <= body_start { return None; }
                let body = &t[body_start..=body_end];
                Some(values.into_iter().map(move |v| body.replace(&var_name, &v)).collect())
            })
            .unwrap_or_default();
    }

    if t.starts_with("@each ") && t.contains('{') {
        return parse_each_sig(&t[6..])
            .map(|(var_name, items)| {
                state.nesting_depth += 1;
                state.collecting = Collecting::Each { var_name, items, body: vec![] };
                vec![]
            })
            .unwrap_or_default();
    }

    if t.starts_with("@for ") && t.contains('{') {
        return parse_for_sig(&t[5..])
            .map(|(var_name, values)| {
                state.nesting_depth += 1;
                state.collecting = Collecting::For { var_name, values, body: vec![] };
                vec![]
            })
            .unwrap_or_default();
    }

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

    if t.starts_with("@include ") {
        let (name, args) = parse_include_sig(&t[9..]);
        return state.scope.mixins.get(&name).map(|mixin_def| {
            expand_mixin(mixin_def, &args)
        }).unwrap_or_default();
    }

    if t.starts_with("@if ") || t.starts_with("@use ") || t.starts_with("@forward ") || t.starts_with("@else ") {
        return vec![];
    }

    if !t.starts_with('@') && t.contains('{') {
        if let Some(brace_pos) = t.find('{') {
            let selector_part = t[..brace_pos].trim();
            let parent = state.selector_stack.last().map(|s| s.as_str());
            let full_selector = match parent {
                Some(ref p) if selector_part.contains('&') => {
                    selector_part.replace('&', p)
                }
                Some(ref p) => {
                    format!("{p} {selector_part}")
                }
                None => selector_part.to_string(),
            };
            let expanded = format!("{full_selector}{}", &t[brace_pos..]);
            state.selector_stack.push(full_selector);
            state.nesting_depth += 1;
            return vec![expanded];
        }
    }

    vec![token]
}

// ═══════════════════════════════════════════════════════════════════════════
// 管线入口 — 算子链 = 工厂模式
// ═══════════════════════════════════════════════════════════════════════════

/// 编译 SCSS → CSS (rxrust Shared 多线程响应式管线)
///
/// 所有权流转:
///   input.lines() → Shared::from_stream 多线程分发
///   scan_map(CompileState) 消费指令, 就地 mutate &mut state
///   flat_map 展开 Vec → 独立事件 (有序组合)
///   scan_map(CssBuilder) 构建 AST, 就地 mutate &mut builder
///   flat_map 展开 CssNode
///   render_node(&node) 借用渲染
///   collect/last 汇聚最终结果
///   subscribe 消费 TaskHandle, String 所有权返回调用者
///
/// 零 Arc<Mutex>, 零外部共享状态 — scan_map 算子即状态机
pub fn compile_pipeline(input: &str) -> String {
    let _root = info_span!("compile_pipeline", bytes = input.len()).entered();

    // Source: input.lines() 是 &str, Shared 跨线程需要 owned String
    // 这里 clone 是不可避免的 (Send + 'static 边界), 后续算子内部全部用借用
    let lines: Vec<String> = input.lines().map(|l| l.to_string()).collect();

    // 终端结果回收: oneshot channel = 单次值传递 (非共享可变状态)
    // tx 包裹在 Option 中, 闭包 take() 实现一次性 move (collect/last 保证最多一次调用)
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let mut tx_opt = Some(tx);

    // 管线: scan_map 持有状态所有权, flat_map 有序组合展开
    let handle = Shared::from_stream(futures::stream::iter(lines))
        // Phase 1: 指令展开 — CompileState 由 scan_map 算子内部管理
        .scan_map(CompileState::new(), dispatch_pass)
        // flat_map 有序组合展开 Vec<Vec<String>> 为独立 String 事件
        .flat_map(|v| Shared::from_stream(futures::stream::iter(v)))
        // Phase 2: AST 构建 — CssBuilder 由 scan_map 算子内部管理
        .scan_map(CssBuilder::new(), |builder, line: String| builder.feed(&line))
        // flat_map 有序组合展开 Vec<CssNode> 为独立 CssNode 事件
        .flat_map(|v| Shared::from_stream(futures::stream::iter(v)))
        // Phase 3: 渲染 — render_node 借用 &CssNode, 产出 owned String
        .map(|node| render_node(&node))
        // 汇聚所有 CSS 行 → Option<Vec<String>>
        .collect::<Vec<String>>()
        .last()
        // 终端: take() 取出 tx 发送结果 (make illegal state unrepresentable)
        .subscribe(move |css_vec: Vec<String>| {
            if let Some(tx) = tx_opt.take() {
                let _ = tx.send(css_vec.join("\n"));
            }
        });

    // 驱动 Shared 管线的 TaskHandle 在当前线程完成
    // collect/last 在 Shared 上下文中返回 SourceWithDynamicSubs<SourceWithDynamicSubs<TaskHandle>>
    // 需要深入 .source.source 拿到真正的 TaskHandle (Future) 才能 block_on
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(handle.source.source);
    });

    // 阻塞接收管线产物, channel 消费后自动释放 — 零 Arc, 零 Mutex, 零 clone
    rx.blocking_recv().unwrap_or_default()
}
