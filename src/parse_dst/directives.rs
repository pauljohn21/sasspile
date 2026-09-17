//! Directive parsing helpers — 纯函数,原语迭代器,无 for+push 累积结果流

use crate::ast::Token;

/// 查找第一个 Ident token 作为 @指令名 — 迭代器 find_map
#[allow(dead_code)]
pub fn parse_at_rule_name(tokens: &[Token]) -> Option<String> {
    tokens.iter().find_map(|t| match t {
        Token::Ident(s) => Some(s.clone()),
        _ => None,
    })
}

/// 拼接 args 为空格分隔字符串 — 迭代器 filter_map + join
#[allow(dead_code)]
pub fn parse_at_rule_args(tokens: &[Token]) -> String {
    tokens
        .iter()
        .filter_map(|t| match t {
            Token::Ident(s) | Token::Number(s) | Token::String(s) | Token::Interpolation(s) => {
                Some(s.clone())
            }
            Token::Dollar => Some("$".to_string()),
            Token::Hash => Some("#".to_string()),
            Token::LParen => Some("(".to_string()),
            Token::RParen => Some(")".to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 顶层分割 tokens — scan 追踪 depth,结果为 owned Vec<Vec<Token>>
#[allow(dead_code)]
pub fn split_top_level_tokens(tokens: &[Token], delimiter: Token) -> Vec<Vec<Token>> {
    #[derive(Clone)]
    struct SplitState {
        depth: usize,
        segments: Vec<Vec<Token>>,
        current: Vec<Token>,
    }

    let final_state = tokens.iter().cloned().scan(
        SplitState {
            depth: 0,
            segments: Vec::new(),
            current: Vec::new(),
        },
        |state, tok| match tok {
            Token::LBrace | Token::LBracket | Token::LParen => {
                state.depth += 1;
                state.current.push(tok);
                Some(state.clone())
            }
            Token::RBrace | Token::RBracket | Token::RParen => {
                state.depth = state.depth.saturating_sub(1);
                state.current.push(tok);
                Some(state.clone())
            }
            ref t if state.depth == 0 && *t == delimiter => {
                if !state.current.is_empty() {
                    state.segments.push(state.current.clone());
                }
                state.current = Vec::new();
                Some(state.clone())
            }
            _ => {
                state.current.push(tok);
                Some(state.clone())
            }
        },
    );

    let last_state = final_state.last();
    match last_state {
        Some(s) if !s.current.is_empty() => {
            let mut segs = s.segments;
            segs.push(s.current);
            segs
        }
        Some(s) => s.segments,
        None => Vec::new(),
    }
}
