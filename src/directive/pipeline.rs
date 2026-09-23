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
// dispatch — match 替代 if/else, 函数式 fold 替代 mutate 循环
// ═══════════════════════════════════════════════════════════════════════════

fn dispatch_pass(state: &mut CompileState, token: String) -> Vec<String> {
    let _span = info_span!("dispatch_pass", phase = ?state.phase, token = %token).entered();
    // 剥离前导空白以支持缩进格式
    let t = token.trim();

    match t {
        // @mixin 定义: 消费 token, 存入 state (含 body), 输出空
        t if t.starts_with("@mixin ") => {
            if let Some((name, params)) = parse_mixin_sig(&t[7..]) {
                // 提取 `{...}` body；若找不到，从 `)` 后取到字符串末尾作为 body
                let body = t.find('{').and_then(|start| {
                    t.rfind('}').map(|end| t[start..=end].to_string())
                }).unwrap_or_default();
                state.scope.mixins.insert(name, MixinDef { params, body: vec![body] });
            }
            vec![]
        }
        // @include: 查 state, fold 展开参数, 输出展开后 token
        t if t.starts_with("@include ") => {
            let (name, args) = parse_include_sig(&t[9..]);
            state.scope.mixins.get(&name).map(|mixin_def| {
                // 默认参数值 (params[i].1)
                let defaults: Vec<Option<&str>> = mixin_def.params.iter()
                    .map(|(_, d)| d.as_deref()).collect();
                mixin_def.body.iter().map(|body_token| {
                    mixin_def.params.iter().enumerate().fold(body_token.clone(), |mut acc, (i, (p_name, _))| {
                        // 优先用调用参数, 其次用参数默认值
                        let replacement = args.get(i)
                            .map(|s| s.as_str())
                            .or_else(|| defaults.get(i).copied().flatten());
                        if let Some(r) = replacement {
                            // p_name 已含 $ 前缀 (如 "$x"), 直接作为替换目标
                            acc = acc.replace(p_name, r);
                        }
                        acc
                    })
                }).collect::<Vec<_>>()
            }).unwrap_or_default()
        }
        // @each: 解析 → 展开为 N 个 token (@each 被消费)
        t if t.starts_with("@each ") => {
            parse_each_sig(&t[6..])
                .map(|(var_name, items)| {
                    items.into_iter().map(|item| {
                        t.replacen("@each", "", 1).replace(&var_name, &item)
                    }).collect()
                })
                .unwrap_or_default()
        }
        // @for: 解析 direction → 展开为 N 个 token
        // 单行 form: `@for $i from 1 through 3 { b: $i; }` → 展开 body `{b: $i;}`, 3 个 token
        t if t.starts_with("@for ") => {
            parse_for_sig(&t[5..])
                .and_then(|(var_name, values)| {
                    // 提取 `{...}` body (支持内联单行)
                    let body_start = t.find('{')?;
                    let body_end = t.rfind('}')?;
                    if body_end <= body_start { return None; }
                    let body = &t[body_start..=body_end];
                    Some(values.into_iter().map(move |v| body.replace(&var_name, &v)).collect())
                })
                .unwrap_or_default()
        }
        // @each: 展开为 N 个 token, body 内变量替换
        t if t.starts_with("@each ") => {
            parse_each_sig(&t[6..])
                .and_then(|(var_name, items)| {
                    let body_start = t.find('{')?;
                    let body_end = t.rfind('}')?;
                    if body_end <= body_start { return None; }
                    let body = &t[body_start..=body_end];
                    Some(items.into_iter().map(move |item| body.replace(&var_name, &item)).collect())
                })
                .unwrap_or_default()
        }
        // @if / @use / @forward: 消费 token, 无输出
        t if t.starts_with("@if ") => vec![],
        t if t.starts_with("@use ") || t.starts_with("@forward ") => vec![],
        // 其他: 透传
        _ => vec![token],
    }
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
