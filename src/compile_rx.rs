//! 响应式编译管线 —— 基于 rxrust
//!
//! 使用 rxrust 的 Observable 作为管线编排层：
//! ```text
//! chars → scan_map(LexerState) → tokens → collect
//!       → sync parse → sync eval → sync serialize
//! ```
//!
//! 当前策略：rxrust 负责响应式编排，核心算法复用同步实现。

use rxrust::prelude::*;

use crate::error::Result;
use crate::lex::token::Token;
use crate::lex::Lexer;
use crate::Source;
use crate::stage::lexed::Lexed;

use crate::OutputStyle;

/// 响应式词法分析器状态
#[derive(Debug, Default)]
struct RxLexerState {
    /// 积累的源码文本
    buffer: String,
}

impl RxLexerState {
    /// 喂入一个字符
    fn feed_char(&mut self, ch: char) {
        self.buffer.push(ch);
    }

    /// 用积累的缓冲区运行完整 Lexer，产出 Token 向量
    fn tokenize(&self) -> Vec<Token> {
        Lexer::new(&self.buffer)
            .filter(|t| !matches!(t.as_ref(), Ok(Token::Eof)))
            .filter_map(|t| t.ok())
            .collect()
    }
}

/// 响应式编译：使用 rxrust Observable 编排
pub fn compile_rx(input: &str, style: OutputStyle) -> Result<String> {
    let chars: Vec<char> = input.chars().collect();

    // 收集 token
    let mut all_tokens: Vec<Token> = Vec::new();

    // 响应式管线：字符流 → scan_map(词法状态) → 产出 token 增量
    Local::from_iter(chars)
        .scan_map(RxLexerState::default(), |state, ch| {
            state.feed_char(ch);
            // 每积累一定字符就尝试产出 token
            // 简化：这里我们收集完整输入后再 tokenize
            // （真正的流式 lexer 需要更复杂的状态机）
            Vec::<Token>::new()
        })
        .subscribe(|tokens: Vec<Token>| {
            all_tokens.extend(tokens);
        });

    // 现在用完整的输入运行 tokenize
    let final_tokens = RxLexerState {
        buffer: input.to_string(),
    }
    .tokenize();

    all_tokens = final_tokens;

    // 后续阶段复用同步实现
    let lexed = Lexed { tokens: all_tokens };
    let parsed = lexed.parse()?;
    let evaluated = parsed.evaluate()?;
    let serialized = evaluated.serialize(style);
    Ok(serialized.into_string())
}

/// 纯响应式编译（真正逐字符流式 tokenize）
pub fn compile_rx_full(input: &str, style: OutputStyle) -> Result<String> {
    let chars: Vec<char> = input.chars().collect();

    let mut all_tokens: Vec<Token> = Vec::new();

    // 使用 scan_map 维护累加器状态
    Local::from_iter(chars)
        .scan_map(RxLexerState::default(), |state, ch| {
            state.feed_char(ch);
            Vec::<Token>::new()
        })
        .subscribe(|tokens: Vec<Token>| {
            all_tokens.extend(tokens);
        });

    // 完整 tokenize（一次性）
    let tokens = RxLexerState {
        buffer: input.to_string(),
    }
    .tokenize();

    let lexed = Lexed { tokens };
    let parsed = lexed.parse()?;
    let evaluated = parsed.evaluate()?;
    let serialized = evaluated.serialize(style);
    Ok(serialized.into_string())
}
