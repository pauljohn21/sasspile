//! DirectiveBlock — 多行指令的状态化分块 (Flux bufferUntil)
//!
//! scan_map 累积行 → emit DirectiveBlock
//! @if/@else 一体化 (状态机跨行跟踪 branches)
//! flat_map 消费 DirectiveBlock → emit Vec<String> (展开后的 CSS 行)

use super::eval::TokenKind;
use super::parse::{parse_each_sig, parse_for_sig, parse_include_sig, parse_mixin_sig};
use super::state::CompileState;
use super::ops::process_block;
use tracing::debug_span;

// ─── 分块产物 ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum DirectiveBlock {
    Lines(Vec<String>),
    For { var_name: String, values: Vec<String>, body: Vec<String> },
    Each { var_name: String, items: Vec<String>, body: Vec<String> },
    If { branches: Vec<(Option<String>, Vec<String>)> },
    MixinDef { name: String, params: Vec<(String, Option<String>)>, body: Vec<String> },
    /// %placeholder 定义 — 存入 state 后不输出
    PlaceholderDef { name: String, body: Vec<String> },
    While { cond: String, body: Vec<String> },
    Include { name: String, args: Vec<String>, using: Vec<String>, body: Vec<String> },
}

// ─── 分块状态机 (scan_map Acc) ──────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct BlockAccumulator {
    current_lines: Vec<String>,
    building: Option<Building>,
    /// 等待后续 @else/@else if 的 Branches (已完成的 @if 分支合并)
    pending_branches: Option<Vec<(Option<String>, Vec<String>)>>,
}

#[derive(Debug)]
enum Building {
    For { var_name: String, values: Vec<String>, body: Vec<String>, brace_depth: i32 },
    Each { var_name: String, items: Vec<String>, body: Vec<String>, brace_depth: i32 },
    If {
        branches: Vec<(Option<String>, Vec<String>)>,
        current_cond: Option<String>,
        current_body: Vec<String>,
    },
    /// 多行规则积累: selector { ... } (含 extend/普通声明)
    Rule { selector: String, body: Vec<String>, brace_depth: i32 },
    MixinDef { name: String, params: Vec<(String, Option<String>)>, body: Vec<String>, brace_depth: i32 },
    PlaceholderDef { name: String, body: Vec<String>, brace_depth: i32 },
    While { cond: String, body: Vec<String>, brace_depth: i32 },
    Include { name: String, args: Vec<String>, using: Vec<String>, body: Vec<String>, brace_depth: i32 },
}

// ─── scan_map reducer: 累积行 → Vec<DirectiveBlock> ──────────────────────────

/// 计算行内 { 和 } 的净增深度
#[inline]
fn count_brace_depth(line: &str) -> i32 {
    line.chars().fold(0, |d, c| match c {
        '{' => d + 1,
        '}' => d - 1,
        _ => d,
    })
}

pub fn accumulate_block(acc: &mut BlockAccumulator, line: String) -> Vec<DirectiveBlock> {
    let trimmed = line.trim();
    let kind = TokenKind::classify(trimmed);

    // 等待 @else 合并: @else if / @else → 继续收集
    if let Some(branches) = acc.pending_branches.take() {
        match kind {
            TokenKind::AtElseIf => {
                let cond = extract_at_else_if_cond(trimmed);
                acc.building = Some(Building::If {
                    branches,
                    current_cond: Some(cond),
                    current_body: vec![],
                });
                return vec![];
            }
            TokenKind::AtElse => {
                acc.building = Some(Building::If {
                    branches,
                    current_cond: None,
                    current_body: vec![],
                });
                return vec![];
            }
            _ => {
                // 非 @else: flush pending_if, 继续正常处理
                let mut result =
                    vec![DirectiveBlock::If { branches }];
                result.extend(accumulate_block(acc, line));
                return result;
            }
        }
    }

    if acc.building.is_some() {
        return append_and_maybe_close(acc, trimmed, kind);
    }

    match kind {
        TokenKind::AtForSingle => try_emit_block(&mut acc.current_lines, parse_single_for(trimmed)),
        TokenKind::AtForMulti => {
            if let Some((var, values)) = trimmed.strip_prefix("@for ").or_else(|| trimmed.strip_prefix("@for")).and_then(|s| parse_for_sig(s)) {
                let brace_depth = count_brace_depth(trimmed);
                acc.building = Some(Building::For { var_name: var, values, body: vec![], brace_depth });
            }
            vec![]
        }
        TokenKind::AtEachSingle => {
            try_emit_block(&mut acc.current_lines, parse_single_each(trimmed))
        }
        TokenKind::AtEachMulti => {
            if let Some((var, items)) = trimmed.strip_prefix("@each ").or_else(|| trimmed.strip_prefix("@each")).and_then(|s| parse_each_sig(s)) {
                let brace_depth = count_brace_depth(trimmed);
                acc.building = Some(Building::Each { var_name: var, items, body: vec![], brace_depth });
            }
            vec![]
        }
        TokenKind::AtIfStart => {
            let cond = extract_at_if_cond(trimmed);
            acc.building = Some(Building::If {
                branches: vec![],
                current_cond: Some(cond),
                current_body: vec![],
            });
            vec![]
        }
        TokenKind::AtWhile => {
            let cond = extract_at_while_cond(trimmed);
            let brace_depth = count_brace_depth(trimmed);
            acc.building = Some(Building::While { cond, body: vec![], brace_depth });
            vec![]
        }
        TokenKind::AtIncludeMulti => {
            // strip @include 前缀后去掉尾部 { 再 parse, 避免 { 混入 args
            let after_include = trimmed.strip_prefix("@include ").unwrap_or(trimmed);
            let sig = after_include.trim_end_matches('{').trim();
            let (name, args) = parse_include_sig(sig);
            // brace_depth = 当前行中 { 的数量 - } 的数量
            let brace_depth = trimmed.chars().fold(0i32, |d, c| match c {
                '{' => d + 1,
                '}' => d - 1,
                _ => d,
            });
            acc.building = Some(Building::Include { name, args, using: vec![], body: vec![], brace_depth });
            vec![]
        }
        TokenKind::PlaceholderDef => {
            if let Some((name, s, e)) = parse_placeholder_sig(trimmed) {
                if e != usize::MAX && e > s {
                    // 单行: %foo { ... }
                    return try_emit_block(
                        &mut acc.current_lines,
                        Some(DirectiveBlock::PlaceholderDef {
                            name,
                            body: vec![trimmed[s..=e].to_string()],
                        }),
                    );
                }
                // 多行: %foo { ... \n ... \n }
                let brace_depth = count_brace_depth(trimmed);
                let after_brace = trimmed[s + 1..].trim();
                let initial_body = if after_brace.is_empty() {
                    vec![]
                } else {
                    vec![after_brace.to_string()]
                };
                acc.building = Some(Building::PlaceholderDef { name, body: initial_body, brace_depth });
            }
            vec![]
        }
        TokenKind::RuleStart => {
            let _span = debug_span!("block.rule_start", selector = %trimmed).entered();
            // 多行规则: "selector {" 或 "selector { @extend" → 积累 body 直到匹配 "}"
            let brace_pos = trimmed.find('{').unwrap_or(0);
            let selector = trimmed[..brace_pos].trim();
            if !selector.is_empty() && !selector.starts_with('@') {
                let brace_depth = count_brace_depth(trimmed);
                // { 后面可能跟了内容 (如 "d {@extend"),需要把这部分放入 body
                let after_brace = trimmed[brace_pos + 1..].trim();
                let initial_body = if after_brace.is_empty() {
                    vec![]
                } else {
                    vec![after_brace.to_string()]
                };
                acc.building = Some(Building::Rule {
                    selector: selector.to_string(),
                    body: initial_body,
                    brace_depth,
                });
                return vec![];
            }
            // 不符合积累条件: 当作普通行
            acc.current_lines.push(line);
            vec![DirectiveBlock::Lines(std::mem::take(&mut acc.current_lines))]
        }
        TokenKind::AtMixinDef => {
            if trimmed.len() > 7 {
                if let (Some((name, params)), Some(s), Some(e)) =
                    (parse_mixin_sig(&trimmed[7..]), trimmed.find('{'), trimmed.rfind('}'))
                {
                    if e > s {
                        return try_emit_block(
                            &mut acc.current_lines,
                            Some(DirectiveBlock::MixinDef {
                                name,
                                params,
                                body: vec![trimmed[s..=e].to_string()],
                            }),
                        );
                    }
                }
                if let Some((name, params)) = parse_mixin_sig(&trimmed[7..]) {
                    let brace_depth = count_brace_depth(trimmed);
                    acc.building = Some(Building::MixinDef { name, params, body: vec![], brace_depth });
                }
            }
            vec![]
        }
        _ => {
            acc.current_lines.push(line);
            vec![DirectiveBlock::Lines(std::mem::take(&mut acc.current_lines))]
        }
    }
}

fn append_and_maybe_close(
    acc: &mut BlockAccumulator,
    t: &str,
    kind: TokenKind,
) -> Vec<DirectiveBlock> {
    let mut building = match acc.building.take() {
        Some(b) => b,
        None => return vec![],
    };

    // 所有累积型 block 共用 brace_depth 逻辑: depth=0 时遇到 } 才关闭
    let depth_delta = count_brace_depth(t);
    let _span = debug_span!("block.append", body = %t, delta = depth_delta).entered();
    let closes = |d: i32| d + depth_delta <= 0 && (t.contains('}') || t.ends_with('}'));

    let result = match &mut building {
        Building::For { body, brace_depth, .. }
        | Building::Each { body, brace_depth, .. }
        | Building::MixinDef { body, brace_depth, .. }
        | Building::PlaceholderDef { body, brace_depth, .. }
        | Building::While { body, brace_depth, .. }
        | Building::Rule { body, brace_depth, .. } => {
            if closes(*brace_depth) {
                // 关闭前提取 } 之前的内容 (如 "a}" → "a")
                if !t.is_empty() && t != "}" {
                    let cleaned = t.trim().trim_end_matches('}').trim();
                    if !cleaned.is_empty() {
                        body.push(cleaned.to_string());
                    }
                }
                block_to_directive(building)
            } else {
                *brace_depth += depth_delta;
                if !t.is_empty() {
                    body.push(t.to_string());
                }
                acc.building = Some(building);
                vec![]
            }
        }
        Building::Include { name, args, using, body, brace_depth } => {
            let new_depth = *brace_depth + depth_delta;
            if closes(*brace_depth) {
                let mut final_body = body.clone();
                if !t.is_empty() && t != "}" {
                    let cleaned = t.trim().trim_end_matches('}').trim();
                    if !cleaned.is_empty() {
                        final_body.push(cleaned.to_string());
                    }
                }
                block_to_directive(Building::Include { name: name.clone(), args: args.clone(), using: using.clone(), body: final_body, brace_depth: 0 })
            } else {
                let mut new_using = using.clone();
                let mut done_using = false;
                if t.trim().starts_with("using") && !t.trim().starts_with("@content") {
                    if let Some(start) = t.find('(') {
                        if let Some(end) = t.rfind(')') {
                            let params = &t[start + 1..end];
                            let params: Vec<String> = params.split(',').map(str::trim).filter(|s| !s.is_empty()).map(String::from).collect();
                            new_using = params;
                            done_using = true;
                        }
                    }
                }
                if !done_using && !t.is_empty() {
                    body.push(t.to_string());
                }
                acc.building = Some(Building::Include { name: name.clone(), args: args.clone(), using: new_using, body: body.clone(), brace_depth: new_depth });
                vec![]
            }
        }
        Building::If {
            branches,
            current_cond,
            current_body,
        } => {
            if t == "}" {
                branches.push((current_cond.take(), std::mem::take(current_body)));
                acc.pending_branches = Some(std::mem::take(branches));
                vec![]
            } else if matches!(kind, TokenKind::AtElseIf) {
                branches.push((current_cond.take(), std::mem::take(current_body)));
                current_cond.replace(extract_at_else_if_cond(t));
                acc.building = Some(building);
                vec![]
            } else if matches!(kind, TokenKind::AtElse) {
                branches.push((current_cond.take(), std::mem::take(current_body)));
                *current_cond = None;
                acc.building = Some(building);
                vec![]
            } else if !t.is_empty() {
                current_body.push(t.to_string());
                acc.building = Some(building);
                vec![]
            } else {
                acc.building = Some(building);
                vec![]
            }
        }
    };
    result
}

fn block_to_directive(building: Building) -> Vec<DirectiveBlock> {
    match building {
        Building::For { var_name, values, body, .. } => {
            vec![DirectiveBlock::For { var_name, values, body }]
        }
        Building::Each { var_name, items, body, .. } => {
            vec![DirectiveBlock::Each { var_name, items, body }]
        }
        Building::If { branches, .. } => vec![DirectiveBlock::If { branches }],
        Building::Rule { selector, body, .. } => {
            // 多行规则: selector + body + 末尾 } (CssBuilder 需要 } 关闭规则)
            let mut lines = vec![format!("{selector} {{")];
            lines.extend(body);
            lines.push("}".to_string());
            vec![DirectiveBlock::Lines(lines)]
        }
        Building::MixinDef { name, params, body, .. } => {
            vec![DirectiveBlock::MixinDef { name, params, body }]
        }
        Building::PlaceholderDef { name, body, .. } => {
            vec![DirectiveBlock::PlaceholderDef { name, body }]
        }
        Building::While { cond, body, .. } => {
            vec![DirectiveBlock::While { cond, body }]
        }
        Building::Include { name, args, using, body, .. } => {
            vec![DirectiveBlock::Include { name, args, using, body }]
        }
    }
}

fn try_emit_block(
    current_lines: &mut Vec<String>,
    block: Option<DirectiveBlock>,
) -> Vec<DirectiveBlock> {
    let mut result: Vec<DirectiveBlock> = std::mem::take(current_lines)
        .into_iter()
        .map(|l| DirectiveBlock::Lines(vec![l]))
        .collect();
    if let Some(b) = block {
        result.push(b);
    }
    result
}

/// 解析 placeholder 签名: "%foo {" / "%foo { color: red; }" → (name, open, close)
/// 支持多行: 有 { 但无 } 时返回 (name, open, usize::MAX) 表示需要后续积累
fn parse_placeholder_sig(line: &str) -> Option<(String, usize, usize)> {
    let s = line.strip_prefix('%').unwrap_or(line).trim_start_matches('%');
    let name = s.split(|c: char| c == '{' || c == '}' || c == ';').next()?.trim().to_string();
    if name.is_empty() { return None; }
    let open = line.find('{')?;
    let close = line.rfind('}').unwrap_or(usize::MAX);
    Some((name, open, close))
}

fn parse_single_for(line: &str) -> Option<DirectiveBlock> {
    let (var, values) = parse_for_sig(&line[5..])?;
    let (s, e) = match (line.find('{'), line.rfind('}')) {
        (Some(a), Some(b)) if b > a => (a, b),
        _ => return None,
    };
    Some(DirectiveBlock::For {
        var_name: var,
        values,
        body: vec![line[s..=e].to_string()],
    })
}

fn parse_single_each(line: &str) -> Option<DirectiveBlock> {
    let (var, items) = parse_each_sig(&line[6..])?;
    let (s, e) = match (line.find('{'), line.rfind('}')) {
        (Some(a), Some(b)) if b > a => (a, b),
        _ => return None,
    };
    Some(DirectiveBlock::Each {
        var_name: var,
        items,
        body: vec![line[s..=e].to_string()],
    })
}

fn extract_at_if_cond(line: &str) -> String {
    let after = line.strip_prefix("@if ").unwrap_or_else(|| line.strip_prefix("@if").unwrap_or("")).trim();
    match after.rfind('{') {
        Some(b) => after[..b].trim().to_string(),
        None => after.to_string(),
    }
}

fn extract_at_else_if_cond(line: &str) -> String {
    let after = line.trim_start_matches("@else").trim().trim_start_matches("if").trim();
    match after.rfind('{') {
        Some(b) => after[..b].trim().to_string(),
        None => after.to_string(),
    }
}

fn extract_at_while_cond(line: &str) -> String {
    let after = line.trim_start_matches("@while").trim();
    match after.rfind('{') {
        Some(b) => after[..b].trim().to_string(),
        None => after.to_string(),
    }
}

// ─── flat_map reducer: DirectiveBlock → Vec<String> ─────────────────────────

pub fn expand_block(state: &mut CompileState, block: DirectiveBlock) -> Vec<String> {
    process_block(block, state)
}

// ─── 管线级: 清理 pending_branches ──────────────────────────────────────────

pub fn flush_pending(acc: &BlockAccumulator) -> Option<DirectiveBlock> {
    acc.pending_branches
        .as_ref()
        .map(|branches| DirectiveBlock::If { branches: branches.clone() })
}
