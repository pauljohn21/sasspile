//! Pipeline — internal only, no pub items.
//!
//! rxrust 4-stage chain:
//! - tokenize   : scan_map + flat_map + filter
//! - parse      : scan_map + flat_map
//! - evaluate   : scan + flat_map — see evaluate_dst.rs
//! - serialize  : map + flat_map + finalize — see serialize_dst.rs
//!
//! Exposed to the crate root via `pub(super) fn build`.
//!
//! Each stage's logic has been moved to its own module:
//!   - `tokenize_dst.rs` — ScannerState (char → Token)
//!   - `parse_dst.rs`    — AstBuilder (Token → Node)
//!   - `evaluate_dst.rs` — Evaluator (Node → Result<Vec<CssNode>, _>)
//!   - `serialize_dst.rs` — Serializer (Result<CssNode, _> → char)
//!
//! Pipeline.rs now only ORCHESTRATES the stage adapters with `.pipe()` + `.box_it()`.

use rxrust::prelude::*;
use std::convert::Infallible;
use std::time::Instant;

use std::path::Path;

use crate::ast::CssNode;
use crate::ast::Node;
use crate::error::CompileError;
use crate::parse_dst::AstBuilder;
use crate::tokenize_dst::ScannerState;

/// Build the internal char-stream Observable (crate-private entry).
pub(super) fn build(input: &str) -> LocalBoxedObservable<'static, char, Infallible> {
    build_with_base(input, None)
}

/// Build with a base directory for resolving relative @import/@use paths.
pub(super) fn build_with_base(
    input: &str,
    base_path: Option<&Path>,
) -> LocalBoxedObservable<'static, char, Infallible> {
    let start = Instant::now();
    let chars: Vec<char> = input.chars().collect();

    // Stage 1: tokenize — char → Token
    let tokens = Local::from_iter(chars)
        .box_it()
        .scan_map(ScannerState::new(), |state, ch| state.feed(ch))
        .flat_map(|toks| Local::from_iter(toks))
        .tap(|tok| tracing::debug!(?tok, stage = "tokenize", "token out"))
        .box_it();

    // Stage 2: parse — Token → Node
    let nodes = tokens
        .scan_map(AstBuilder::new(), |builder, token| builder.feed(token))
        .flat_map(|ns| Local::from_iter(ns))
        .tap(|node| tracing::info!(?node, stage = "parse", "node out"))
        .box_it();

    // Stage 3: evaluate — Node → Result<Vec<CssNode>, CompileError>
    //
    // Operator mapping:
    //   - scan      : thread CompilerContext state across items → variable scope
    //   - flat_map  : 1 Node expands to 0..N CssNode's (@for, @include, @import)
    //   - distinct_until_changed : suppress duplicate errors
    let mut initial_ctx = crate::shared::context::CompilerContext::new();
    if let Some(bp) = base_path {
        initial_ctx.path_stack.push(bp.to_path_buf());
    }
    let evaluated = crate::evaluate_dst::attach_with_ctx(nodes, initial_ctx);

    // Stage 4: serialize — flattens inner Vec<CssNode> and emits char stream.
    //
    // Adapter:  fn(map(|outcome| → Vec<Result<CssNode, _>>))
    //           .flat_map(each Item to stream)
    //           .pipe(serialize_dst::attach)
    type SerItem = Result<CssNode, CompileError>;

    let ser_input = evaluated
        .tap(|outcome| tracing::info!(?outcome, stage = "evaluate", "item out"))
        .flat_map(|outcome| {
            let items: Vec<SerItem> = match outcome {
                Ok(nodes) => nodes.into_iter().map(Ok).collect(),
                Err(e) => vec![Err(e)],
            };
            Local::from_iter(items)
        })
        .tap(|item| tracing::info!(?item, stage = "serialize", "ser input"))
        .box_it();

    // Stage 4: serialize — SerItem → char
    crate::serialize_dst::attach(ser_input, start)
}
