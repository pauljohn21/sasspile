//! DirectiveBlock — 多行指令的状态化分块 (Flux bufferUntil)
//!
//! scan_map 累积行 → emit DirectiveBlock
//! @if/@else 一体化 (状态机跨行跟踪 branches)
//! flat_map 消费 DirectiveBlock → emit Vec<String> (展开后的 CSS 行)

use super::eval::TokenKind;
use super::parse::{parse_each_sig, parse_for_sig, parse_mixin_sig, substitute_vars};
use super::state::CompileState;
use super::ops::process_block;

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
    For { var_name: String, values: Vec<String>, body: Vec<String> },
    Each { var_name: String, items: Vec<String>, body: Vec<String> },
    If {
        branches: Vec<(Option<String>, Vec<String>)>,
        current_cond: Option<String>,
        current_body: Vec<String>,
    },
    MixinDef { name: String, params: Vec<(String, Option<String>)>, body: Vec<String> },
    PlaceholderDef { name: String, body: Vec<String> },
}

// ─── scan_map reducer: 累积行 → Vec<DirectiveBlock> ──────────────────────────

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
            if let Some((var, values)) = parse_for_sig(&trimmed[5..]) {
                acc.building = Some(Building::For { var_name: var, values, body: vec![] });
            }
            vec![]
        }
        TokenKind::AtEachSingle => {
            try_emit_block(&mut acc.current_lines, parse_single_each(trimmed))
        }
        TokenKind::AtEachMulti => {
            if let Some((var, items)) = parse_each_sig(&trimmed[6..]) {
                acc.building = Some(Building::Each { var_name: var, items, body: vec![] });
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
        TokenKind::PlaceholderDef => {
            // 单行: "%foo { color: red; }"
            if let Some((name, s, e)) = parse_placeholder_sig(trimmed) {
                if e > s {
                    return try_emit_block(
                        &mut acc.current_lines,
                        Some(DirectiveBlock::PlaceholderDef {
                            name,
                            body: vec![trimmed[s..=e].to_string()],
                        }),
                    );
                }
                // 多行: "%foo {" — 开启收集
                acc.building = Some(Building::PlaceholderDef { name, body: vec![] });
            }
            vec![]
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
                    acc.building = Some(Building::MixinDef { name, params, body: vec![] });
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

    let result = match &mut building {
        Building::For { body, .. }
        | Building::Each { body, .. }
        | Building::MixinDef { body, .. }
        | Building::PlaceholderDef { body, .. } => {
            if t == "}" {
                block_to_directive(building)
            } else if !t.is_empty() {
                body.push(t.to_string());
                acc.building = Some(building);
                vec![]
            } else {
                acc.building = Some(building);
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
                // 暂存 branches, 等待可能的 @else
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
        Building::For { var_name, values, body } => {
            vec![DirectiveBlock::For { var_name, values, body }]
        }
        Building::Each { var_name, items, body } => {
            vec![DirectiveBlock::Each { var_name, items, body }]
        }
        Building::If { branches, .. } => vec![DirectiveBlock::If { branches }],
        Building::MixinDef { name, params, body } => {
            vec![DirectiveBlock::MixinDef { name, params, body }]
        }
        Building::PlaceholderDef { name, body } => {
            vec![DirectiveBlock::PlaceholderDef { name, body }]
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
fn parse_placeholder_sig(line: &str) -> Option<(String, usize, usize)> {
    let s = line.strip_prefix('%').unwrap_or(line).trim_start_matches('%');
    let name = s.split(|c: char| c == '{' || c == '}' || c == ';').next()?.trim().to_string();
    if name.is_empty() { return None; }
    match (line.find('{'), line.rfind('}')) {
        (Some(o), Some(c)) => Some((name, o, c)),
        _ => None,
    }
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
    let after = line[4..].trim();
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
