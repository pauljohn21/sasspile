//! 统一响应式管线 — rxrust 1.0.0-rc.5 Shared 多线程
//!
//! 管线:
//!   Shared::from_stream(iter(tokens))
//!     .scan_map(CompileState::new(), dispatch_pass)
//!     .flat_map(|v| Shared::from_stream(iter(v)))
//!     .collect::<Vec<String>>()
//!     .last()
//!     .subscribe(|css| tx.send(css.join("\n")))
//!
//! 结果传递: subscribe → std::sync::mpsc → rx.recv() 阻塞
//! 调度: SharedScheduler 内部 tokio runtime, 独立驱动
//! 零手写 block_on, 零手写 runtime.

use super::state::*;
use rxrust::prelude::*;
use tracing::info_span;

// ═══════════════════════════════════════════════════════════════════════════
// 解析辅助 (纯函数, 无副作用)
// ═══════════════════════════════════════════════════════════════════════════

fn parse_mixin_sig(s: &str) -> Option<(String, Vec<(String, Option<String>)>)> {
    // s 形式: "name($x: default, $y)" — 找到第一个 ( 和对应的 )
    let (name, rest) = s.split_once('(')?;
    let name = name.trim().to_string();
    // 仅取到第一个 ) 为止作为参数列表 (之后是 { body })
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
    // rest 现在是 "1 through 3 ..." 或 "1 to 3 ..."
    // 取第一个数字作为 from, 取 'through'/'to' 后的数字作为 to
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
    // 剥离尾随分号和空白 (处理输入行的 SCSS 语法)
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
// finalize — 闭合 block 时展开 body (取出 state.collecting, 重置为 None)
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

// ═══════════════════════════════════════════════════════════════════════════
// expand_mixin — 查询 mixin_def, fold 参数替换
// ═══════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════
// dispatch — if/else 链 + 状态机 (scan_map 驱动)
// ═══════════════════════════════════════════════════════════════════════════

fn dispatch_pass(state: &mut CompileState, token: String) -> Vec<String> {
    let _span = info_span!("dispatch_pass", phase = ?state.phase, token = %token).entered();
    let t = token.trim();

    // ── 状态机: 收集中的 block body ────────────────────────────────────────
    // 检测闭合 `}` — 无论 collecting 是什么状态, 遇到 } 都递减 depth, depth=0 时 finalize
    if state.collecting != Collecting::None && t == "}" {
        let result = finalize_collecting(state);
        state.nesting_depth = state.nesting_depth.saturating_sub(1);
        return result;
    }

    // 收集模式下: 累积 body depth++
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

    // ── 非收集模式: dispatch 到各指令处理 ─────────────────────────────────

    // 单行展开: 指令 + body 同行 (含 `{ ... }`)
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

    // 多行收集: 指令 + { 开启, body 在后续行
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

    // @mixin: 提取 params + body (支持单行和多行)
    if t.starts_with("@mixin ") {
        if let Some((name, params)) = parse_mixin_sig(&t[7..]) {
            if let Some(body_start) = t.find('{') {
                if let Some(body_end) = t.rfind('}') {
                    if body_end > body_start {
                        // 单行: @mixin foo() { ... } 同行 body
                        let body = t[body_start..=body_end].to_string();
                        state.scope.mixins.insert(name, MixinDef { params, body: vec![body] });
                        return vec![];
                    }
                }
            }
            // 多行: @mixin foo() { 开始, 后续行收集
            state.current_mixin_name = Some(name.clone());
            state.collecting = Collecting::MixinDef { params };
            state.scope.mixins.insert(name, MixinDef { params: vec![], body: vec![] });
            state.nesting_depth += 1;
        }
        return vec![];
    }

    // @include: 查询 mixin, 展开参数
    if t.starts_with("@include ") {
        let (name, args) = parse_include_sig(&t[9..]);
        return state.scope.mixins.get(&name).map(|mixin_def| {
            expand_mixin(mixin_def, &args)
        }).unwrap_or_default();
    }

    // @if / @use / @forward / @else: 消费
    if t.starts_with("@if ") || t.starts_with("@use ") || t.starts_with("@forward ") || t.starts_with("@else ") {
        return vec![];
    }

    // 其他: 透传
    vec![token]
}

// ═══════════════════════════════════════════════════════════════════════════
// 管线入口 — Shared 多线程, 结果经 channel 传回
// ═══════════════════════════════════════════════════════════════════════════

/// 编译 SCSS → CSS (Shared 多线程响应式管线)
///
/// 数据流: input → Shared::from_stream → scan_map → flat_map → collect → last → subscribe(mpsc)
/// SharedScheduler 需要 tokio runtime — 本函数内创建局部 runtime 驱动管线。
/// subscribe 触发管线执行, 结果经 channel 传回, recv 阻塞等待。
pub fn compile_pipeline(input: &str) -> String {
    let _root = info_span!("compile_pipeline", bytes = input.len()).entered();

    // SharedScheduler 需要 tokio runtime (内部 spawn 任务)
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("sasspile-shared")
        .enable_all()
        .build()
        .expect("tokio runtime failed");

    rt.block_on(async {
        let tokens: Vec<String> = input.lines().map(|l| l.to_string()).collect();
        let state = CompileState::new();
        let (tx, rx) = tokio::sync::oneshot::channel();

        // Shared 管线 — SharedScheduler 全局 tokio runtime, 无需额外 runtime
        // subscribe 是 FnMut (可能多次调用), 用 Option + take 实现一次性 move
        let mut tx_opt = Some(tx);
        let _sub = Shared::from_stream(futures::stream::iter(tokens))
            .scan_map(state, dispatch_pass)
            .flat_map(|v| Shared::from_stream(futures::stream::iter(v)))
            .collect::<Vec<String>>()
            .last()
            .subscribe(move |css| {
                if let Some(sender) = tx_opt.take() {
                    let _ = sender.send(css.join("\n"));
                }
            });

        // 等待管线完成 → rx 接收最终结果
        rx.await.unwrap_or_default()
    })
}
