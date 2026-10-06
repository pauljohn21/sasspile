//! Parser — 将 Token 流转为 SassAstNode（解析树）流
//!
//! 纯响应式架构：
//! 1. `scan_map(ParserState)` 增量解析 → 输出 Vec<SassAstNode>
//! 2. `flat_map` 展平 Vec 为单个节点
//! 3. `subscribe(closure)` 转发节点（FnMutObserver::complete 为 no-op）
//! 4. subscribe 返回（Local 流同步结束）时检查未闭合分隔符 → error()/complete()

mod ast;
mod state;
mod syntax;

pub use ast::SassAstNode;
pub use syntax::InputSyntax;
pub use state::ParserState;

use rxrust::prelude::*;
use std::cell::RefCell;
use std::convert::Infallible;
use std::rc::Rc;
use crate::lexer::{Token, TokenKind};
use tracing::debug_span;
use crate::Error;

/// SassAstNode stream type (with error propagation)
pub type SassAstStream = LocalBoxedObservableClone<'static, SassAstNode, Error>;

/// 语法分析入口：将 Token 流转为 SassAstNode 流
///
/// 纯响应式管道 + completion 错误检测：
/// - scan_map 增量解析 Token，维护 ParserState，输出 Vec<SassAstNode>
/// - flat_map 展平为单个节点流
/// - subscribe 闭包转发节点（不 move subscriber，仅 borrow）
/// - Local 流同步完成后，检查未闭合分隔符并调用 error()/complete()
///
/// 错误传播：当检测到未闭合括号（如 `(key: value`）时，
/// 通过 rxrust 的 `on_error` 通道报告 `crate::Error`。
pub fn parse(input: LocalBoxedObservableClone<'static, Token, Infallible>) -> SassAstStream {
    let span = debug_span!("parser");
    let _guard = span.enter();

    Local::create(move |subscriber| {
        let state: Rc<RefCell<ParserState>> = Rc::new(RefCell::new(ParserState::new()));
        let state_for_scan = state.clone();

        // ── 响应式解析管道：scan_map + flat_map → collect ──
        // scan_map: &mut ParserState 增量解析，输出 Vec<SassAstNode>
        // flat_map: 展平 Vec 为单个节点流
        // collect: 收集所有节点到单个 Vec 中（流式结束发出）
        let collected = input
            .scan_map(ParserState::new(), move |s: &mut ParserState, tok| {
                s.push_token(tok.clone());
                let mut nodes = Vec::new();
                while let Some(node) = try_parse_statement(s) {
                    nodes.push(node);
                }
                // 同步内部状态到共享引用（供 completion 检查）
                *state_for_scan.borrow_mut() = s.clone();
                nodes
            })
            .flat_map(|nodes| Local::from_iter(nodes))
            .collect::<Vec<_>>();

        // 收集结果：subscribe 闭包通过 Rc 共享数据（move 而非 borrow 栈变量）
        let all_nodes = Rc::new(RefCell::new(Vec::<SassAstNode>::new()));
        let nodes_ref = all_nodes.clone();
        collected.subscribe(move |v| *nodes_ref.borrow_mut() = v);

        // ── 转发所有节点到 subscriber ──
        let nodes = std::mem::take(&mut *all_nodes.borrow_mut());
        for node in nodes {
            subscriber.next(node);
        }

        // ── 流已结束：检查未闭合分隔符 ──
        if let Some(err) = check_unclosed_delimiters(&state.borrow()) {
            subscriber.error(err);
        } else {
            subscriber.complete();
        }
    })
    .box_it_clone()
}

/// 检查未闭合的分隔符：扫描 state 缓冲区中所有 token
/// 若存在未闭合的 `(` 或 `{`，返回对应 ParserError
fn check_unclosed_delimiters(state: &ParserState) -> Option<Error> {
    let mut paren_depth: i32 = 0;
    let mut brace_depth: i32 = 0;
    let mut paren_pos: Option<u32> = None;
    let mut brace_pos: Option<u32> = None;

    state.scan_tokens(|tok| {
        match tok.char_kind() {
            Some('(') => {
                if paren_depth == 0 {
                    paren_pos = Some(tok.pos);
                }
                paren_depth += 1;
            }
            Some(')') => {
                paren_depth -= 1;
            }
            Some('{') => {
                if brace_depth == 0 {
                    brace_pos = Some(tok.pos);
                }
                brace_depth += 1;
            }
            Some('}') => {
                brace_depth -= 1;
            }
            _ => {}
        }
    });

    if paren_depth > 0 {
        return Some(Error::parser(format!(
            "unclosed '(' at position {}",
            paren_pos.unwrap_or(0)
        )));
    }

    if brace_depth > 0 {
        return Some(Error::parser(format!(
            "unclosed '{{' at position {}",
            brace_pos.unwrap_or(0)
        )));
    }

    None
}

/// Try to parse one statement from the current state.
fn try_parse_statement(state: &mut ParserState) -> Option<SassAstNode> {
    if let Some((node, consumed)) = try_parse_self_contained(state) {
        for _ in 0..consumed {
            state.advance();
        }
        return Some(node);
    }

    let mut terminator = None;
    let len = state.tokens_len();
    for i in 0..len {
        if let Some(tok) = state.peek_n(i) {
            if tok.char_kind() == Some(';') || tok.char_kind() == Some('}') {
                terminator = Some(i);
                break;
            }
        }
    }

    let end_pos = terminator?;

    let mut stmt_tokens = Vec::new();
    for _ in 0..=end_pos {
        if let Some(tok) = state.peek() {
            stmt_tokens.push(tok.clone());
            state.advance();
        }
    }

    if stmt_tokens.is_empty() {
        return None;
    }

    if let Some(group_node) = try_parse_group_literal(&stmt_tokens) {
        return Some(group_node);
    }

    let is_brace_terminated = stmt_tokens
        .last()
        .and_then(|t| t.char_kind()) == Some('}');
    let is_semi_terminated = stmt_tokens
        .last()
        .and_then(|t| t.char_kind()) == Some(';');
    if !is_brace_terminated && (is_semi_terminated || !stmt_tokens.is_empty()) {
        if let Some(interp_node) = try_parse_interpolation(&stmt_tokens) {
            return Some(interp_node);
        }
    }

    if stmt_tokens.len() >= 3 && stmt_tokens[0].kind == TokenKind::Variable {
        if stmt_tokens[1].char_kind() == Some(':') {
            let name = stmt_tokens[0].text.clone();
            let value_part: Vec<Token> = stmt_tokens[2..]
                .iter()
                .filter(|t| t.char_kind() != Some(';'))
                .cloned()
                .collect();
            let value_text: String = value_part.iter().map(|t| t.text.as_str()).collect();

            let has_default = stmt_tokens.iter().any(|t| {
                t.kind == TokenKind::Ident && t.text == "default"
            }) || stmt_tokens.iter().any(|t| t.text.contains("default"));

            return Some(SassAstNode::VariableDecl {
                name,
                value: Box::new(SassAstNode::Raw(value_text.trim().to_string())),
                has_default,
            });
        }
    }

    if let Some(brace_pos) = stmt_tokens.iter().position(|t| t.char_kind() == Some('{')) {
        if brace_pos > 0 {
            let selector: String = stmt_tokens[..brace_pos]
                .iter()
                .map(|t| t.text.as_str())
                .collect();

            let inner_part: Vec<Token> = stmt_tokens[brace_pos + 1..]
                .iter()
                .filter(|t| t.char_kind() != Some('}') && t.char_kind() != Some(';'))
                .cloned()
                .collect();
            let inner_text: String = inner_part.iter().map(|t| t.text.as_str()).collect();

            let inner = if inner_text.trim().is_empty() {
                vec![]
            } else {
                vec![SassAstNode::Raw(inner_text)]
            };

            return Some(SassAstNode::Rule {
                selector: selector.trim().to_string(),
                inner,
            });
        }
    }

    if stmt_tokens.len() >= 3 {
        if let Some(colon_pos) = stmt_tokens.iter().position(|t| t.char_kind() == Some(':')) {
            if colon_pos > 0 {
                let prop: String = stmt_tokens[..colon_pos]
                    .iter()
                    .map(|t| t.text.as_str())
                    .collect();
                let value: String = stmt_tokens[colon_pos + 1..]
                    .iter()
                    .filter(|t| t.char_kind() != Some(';'))
                    .map(|t| t.text.as_str())
                    .collect();

                if !prop.trim().is_empty() {
                    return Some(SassAstNode::StyleDecl {
                        prop: prop.trim().to_string(),
                        value: value.trim().to_string(),
                    });
                }
            }
        }
    }

    let text: String = stmt_tokens.iter().map(|t| t.text.as_str()).collect();
    Some(SassAstNode::Raw(text))
}

/// 尝试解析自封闭表达式
fn try_parse_self_contained(state: &ParserState) -> Option<(SassAstNode, usize)> {
    let first = state.peek()?;
    let len = state.tokens_len();

    match &first.kind {
        TokenKind::Interpolation => {
            let inner = extract_interp_inner(&first.text);
            Some((SassAstNode::Interpolated(inner), 1))
        }
        _ => {
            let mut has_terminator = false;
            let mut interp_pos = None;
            let mut paren_pos = None;

            for i in 0..len {
                if let Some(tok) = state.peek_n(i) {
                    match &tok.kind {
                        TokenKind::Char(c) if *c == ';' || *c == '}' => {
                            has_terminator = true;
                            break;
                        }
                        TokenKind::Interpolation if interp_pos.is_none() => {
                            interp_pos = Some(i);
                        }
                        TokenKind::Char('(') if paren_pos.is_none() => {
                            paren_pos = Some(i);
                        }
                        _ => {}
                    }
                }
            }

            if has_terminator {
                return None;
            }

            if let Some(pos) = interp_pos {
                let consumed_tokens: Vec<Token> = (0..=pos)
                    .filter_map(|j| state.peek_n(j).cloned())
                    .collect();
                if let Some(node) = try_parse_interpolation(&consumed_tokens) {
                    return Some((node, pos + 1));
                }
            }

            if let Some(pos) = paren_pos {
                let mut depth = 0usize;
                for i in pos..len {
                    if let Some(tok) = state.peek_n(i) {
                        match tok.char_kind() {
                            Some('(') => depth += 1,
                            Some(')') => {
                                depth -= 1;
                                if depth == 0 {
                                    let consumed_tokens: Vec<Token> = (0..=i)
                                        .filter_map(|j| state.peek_n(j).cloned())
                                        .collect();
                                    if let Some(node) = try_parse_group_literal(&consumed_tokens) {
                                        return Some((node, i + 1));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            None
        }
    }
}

/// 尝试解析插值表达式
fn try_parse_interpolation(tokens: &[Token]) -> Option<SassAstNode> {
    let has_interp = tokens.iter().any(|t| t.kind == TokenKind::Interpolation);

    if !has_interp {
        return None;
    }

    if tokens.len() == 1 && tokens[0].kind == TokenKind::Interpolation {
        let inner = extract_interp_inner(&tokens[0].text);
        return Some(SassAstNode::Interpolated(inner));
    }

    let mut result = String::new();
    for tok in tokens.iter() {
        match &tok.kind {
            TokenKind::Interpolation => {
                result.push_str(&tok.text);
            }
            TokenKind::Ident | TokenKind::Variable | TokenKind::Number | TokenKind::String => {
                result.push_str(&tok.text);
            }
            TokenKind::Char(c) => {
                if *c != ';' && *c != '}' {
                    result.push(*c);
                }
            }
            _ => {}
        }
    }

    if result.trim().is_empty() {
        None
    } else {
        Some(SassAstNode::Interpolated(result.trim().to_string()))
    }
}

/// 从 `#{$var}` 中提取内部变量表达式
fn extract_interp_inner(text: &str) -> String {
    text.trim_start_matches("#{")
        .trim_end_matches('}')
        .to_string()
}

/// 尝试解析分组字面量
fn try_parse_group_literal(tokens: &[Token]) -> Option<SassAstNode> {
    let first_non_ws = tokens.iter().find(|t| {
        !matches!(t.kind, TokenKind::Char(c) if c.is_whitespace())
    })?;
    let last_non_term = tokens.iter().rev().find(|t| {
        !matches!(t.char_kind(), Some(';') | Some('}'))
    })?;

    if first_non_ws.char_kind() != Some('(') || last_non_term.char_kind() != Some(')') {
        return None;
    }

    let inner_tokens: Vec<&Token> = tokens
        .iter()
        .skip_while(|t| {
            matches!(t.kind, TokenKind::Char(c) if c.is_whitespace())
        })
        .skip(1)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .skip_while(|t| {
            matches!(t.kind, TokenKind::Char(c) if c.is_whitespace())
        })
        .skip(1)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    if inner_tokens.is_empty() {
        return Some(SassAstNode::ListLiteral(vec![]));
    }

    let is_map = has_top_level_colon(&inner_tokens);

    if is_map {
        let entries = parse_map_entries(&inner_tokens);
        Some(SassAstNode::MapLiteral(entries))
    } else {
        let items = parse_list_items(&inner_tokens);
        Some(SassAstNode::ListLiteral(items))
    }
}

/// 检测顶层冒号
fn has_top_level_colon(tokens: &[&Token]) -> bool {
    let mut depth: i32 = 0;
    for tok in tokens {
        match tok.char_kind() {
            Some('(') | Some('{') | Some('[') => depth += 1,
            Some(')') | Some('}') | Some(']') => depth -= 1,
            Some(':') if depth == 0 => return true,
            _ => {}
        }
    }
    false
}

/// 解析 Map 键值对
fn parse_map_entries(tokens: &[&Token]) -> Vec<(SassAstNode, SassAstNode)> {
    let mut entries = Vec::new();
    let mut current_key = String::new();
    let mut current_value = String::new();
    let mut found_colon = false;
    let mut depth: i32 = 0;

    for tok in tokens {
        match tok.char_kind() {
            Some('(') | Some('{') | Some('[') => {
                depth += 1;
                if found_colon {
                    current_value.push_str(&tok.text);
                } else {
                    current_key.push_str(&tok.text);
                }
            }
            Some(')') | Some('}') | Some(']') => {
                depth -= 1;
                if found_colon {
                    current_value.push_str(&tok.text);
                } else {
                    current_key.push_str(&tok.text);
                }
            }
            Some(',') if depth == 0 => {
                if found_colon {
                    entries.push((make_value_node(&current_key), make_value_node(&current_value)));
                }
                current_key.clear();
                current_value.clear();
                found_colon = false;
            }
            Some(':') if depth == 0 => {
                found_colon = true;
            }
            _ => {
                if found_colon {
                    current_value.push_str(&tok.text);
                } else {
                    current_key.push_str(&tok.text);
                }
            }
        }
    }

    if !current_key.trim().is_empty() && found_colon {
        entries.push((make_value_node(&current_key), make_value_node(&current_value)));
    }

    entries
}

/// 解析 List 元素
fn parse_list_items(tokens: &[&Token]) -> Vec<SassAstNode> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut depth: i32 = 0;

    for tok in tokens {
        match tok.char_kind() {
            Some('(') | Some('{') | Some('[') => {
                depth += 1;
                current.push_str(&tok.text);
            }
            Some(')') | Some('}') | Some(']') => {
                depth -= 1;
                current.push_str(&tok.text);
            }
            Some(',') if depth == 0 => {
                if !current.trim().is_empty() {
                    items.push(make_value_node(&current));
                }
                current.clear();
            }
            _ => {
                current.push_str(&tok.text);
            }
        }
    }

    if !current.trim().is_empty() {
        items.push(make_value_node(&current));
    }

    items
}

/// 从文本创建值节点
fn make_value_node(text: &str) -> SassAstNode {
    SassAstNode::Raw(text.trim().to_string())
}

/// Created by script — helper re-export for integration tests
pub use crate::lexer::normalize_newlines;
