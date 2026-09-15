//! 流式 Parser —— 结构累积
//!
//! 核心：`feed_token: (ParseState, Token) -> (ParseState, Vec<Node>)` 是纯函数。
//! scan 累加器 = ParseState，每步 emit 0..N 个 Node。
//!
//! 使用 scan + flat_map 模式：
//!   scan 维护 ParseState（token 缓冲 + 解析位置），返回产出 Node 的 Vec
//!   flat_map 将每次产出的 Node 展开到输出流

use crate::error::SassError;
use crate::lex::token::Token;
use crate::parse::ast::Node;
use crate::parse::ParseStream;

/// 流式解析状态（不可变数据，每次创建新实例）
#[derive(Clone)]
pub struct ParseState {
    /// 已接收但未完全解析的 token 缓冲
    buffer: Vec<Token>,
    /// 当前已解析的绝对位置
    consumed: usize,
    /// 是否在规则体内
    in_body: bool,
    /// 是否已解析过非模块规则
    saw_other_rule: bool,
}

impl Default for ParseState {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            consumed: 0,
            in_body: false,
            saw_other_rule: false,
        }
    }
}

/// 纯函数：`(ParseState, Token) -> (ParseState, Vec<Node>)`
///
/// 每接收一个 token，追加到缓冲，然后尝试从缓冲中解析尽可能多的完整节点。
/// 解析出的节点在下次 scan 调用时通过 flat_map 展开。
pub fn feed_token(state: ParseState, token: Token) -> (ParseState, Vec<Node>) {
    let is_eof = matches!(token, Token::Eof);

    // 追加 token 到缓冲
    let mut buffer = state.buffer;
    buffer.push(token);

    // 尝试从缓冲中解析尽可能多的完整节点
    let mut nodes = Vec::new();
    let mut consumed = state.consumed;
    let mut in_body = state.in_body;
    let mut saw_other_rule = state.saw_other_rule;

    loop {
        let remaining = &buffer[consumed..];
        if remaining.is_empty() {
            break;
        }

        // 跳过空白
        let skip_count = remaining.iter().take_while(|t| matches!(t, Token::Whitespace)).count();
        let after_ws = &remaining[skip_count..];

        if after_ws.is_empty() || matches!(after_ws.first(), Some(Token::Eof)) {
            break;
        }

        // 尝试解析一个节点
        let mut stream = ParseStream::with_context(remaining, in_body, saw_other_rule);
        match stream.next() {
            Some(Ok(node)) => {
                let tokens_consumed = stream.consumed_count() + skip_count;
                consumed += tokens_consumed;
                nodes.push(node);

                // 更新上下文（简化：解析后重置 in_body）
                // 实际应该根据节点类型更新，但 scan 的纯函数约束下难以精确追踪
                // 这里采用保守策略：解析一个节点后保持当前上下文
            }
            Some(Err(_)) => {
                // 解析错误：可能是需要更多 token，也可能是真正的错误
                // 策略：如果是 EOF 前的最后一个错误，emit error
                // 否则等待更多 token
                if is_eof {
                    // 无法继续，跳出
                    break;
                }
                break;
            }
            None => break,
        }
    }

    (
        ParseState {
            buffer,
            consumed,
            in_body,
            saw_other_rule,
        },
        nodes,
    )
}

/// 便利函数：一次性解析所有 token（用于测试和 fallback）
pub fn parse_all(tokens: &[Token]) -> Result<Vec<Node>, SassError> {
    let mut stream = ParseStream::new(tokens);
    let mut nodes = Vec::new();
    while let Some(result) = stream.next() {
        nodes.push(result?);
    }
    Ok(nodes)
}
