//! Pipeline 组装 — char stream → Token → Node → CssNode → char stream
//!
//! build 是一个完整的 chain: 从 from_iter(chars) 经过 scan_map / flat_map
//! 串联四个阶段,最后 box_it 一次做 type erasure，得到
//! LocalBoxedObservable<char, Infallible>。
//!
//! 不拆分中间变量、不做中间 box_it，所有权随链转移，最终由 collect/last 消费。

use std::convert::Infallible;

use rxrust::prelude::*;

use crate::evaluate_dst;
use crate::parse_dst::AstBuilder;
use crate::serialize_dst;
use crate::tokenize_dst::Scanner;

/// 构建完整编译 pipeline: &str → LocalBoxedObservable<char, Infallible>
pub fn build(input: &str) -> LocalBoxedObservable<'static, char, Infallible> {
    let chars: Vec<char> = input.chars().collect();

    // Stage 1: char → token
    //   scan_map(Scanner):   char → Vec<Token>
    //   flat_map(from_iter): Vec<Token> → Token (逐个)
    // Stage 2: token → node
    //   scan_map(AstBuilder): Token → Vec<Node>
    //   flat_map(from_iter):  Vec<Node> → Node
    // Stage 3: node → CssNode (flat_map eval 展开)
    // Stage 4: CssNode → char (flat_map render 展开)
    // Final: box_it 做 type erasure,得到 LocalBoxedObservable<char>
    Local::from_iter(chars)
        .scan_map(Scanner::new(), |state, ch| state.feed(ch))
        .flat_map(|toks| Local::from_iter(toks))
        .tap(|tok| tracing::trace!(?tok, stage = "tokenize"))
        .scan_map(AstBuilder::new(), |builder, tok| builder.feed(tok))
        .flat_map(|node_vec| Local::from_iter(node_vec))
        .tap(|node| tracing::trace!(?node, stage = "parse"))
        .flat_map(|node| Local::from_iter(evaluate_dst::eval_node_vec(node)))
        .tap(|css| tracing::trace!(?css, stage = "evaluate"))
        .flat_map(|css_node| {
            Local::from_iter(serialize_dst::render_node_to_chars(css_node))
        })
        .tap(|ch| tracing::trace!(char = %ch, stage = "serialize"))
        .box_it()
}

/// 带 base_path 的 pipeline（用于 @import 解析,TODO: 实现完整的 base_path 传递）
pub fn build_with_base(
    input: &str,
    _base_path: Option<&std::path::Path>,
) -> LocalBoxedObservable<'static, char, Infallible> {
    build(input)
}
