//! Serialize Stage (Stage 4)
//!
//! `Observable<Result<CssNode, CompileError>, Infallible>`
//!     → `Observable<char, Infallible>`
//!
//! Pipeline fragment:
//!   .map(render_or_error_banner)
//!   .flat_map(chars_of)
//!   .finalize(trace elapsed)
//!
//! Side-effects (tracing elapsed_us) are isolated in `.finalize()` —
//! this is the ONLY side-effect site in this stage.

use rxrust::prelude::*;
use std::convert::Infallible;
use std::time::Instant;

use crate::ast::CssNode;
use crate::error::CompileError;

/// Type alias for the input to this stage: Result-wrapped CssNode per Item.
pub type EvalItem = Result<CssNode, CompileError>;

/// Build the serialize stage closure — returns a function that attaches the
/// serialize operators to an upstream `Observable<EvalItem, Infallible>`.
///
/// This is the rxrust "stage adapter" pattern: each stage is a function that
/// takes an upstream Observable + a tick (Instant for finalize tracing) and
/// returns a downstream Observable, with `.box_it()` at the boundary.
pub fn attach(
    upstream: LocalBoxedObservable<'static, EvalItem, Infallible>,
    start: Instant,
) -> LocalBoxedObservable<'static, char, Infallible> {
    let _span = tracing::info_span!("serialize.attach").entered();

    upstream
        .map(render_item)
        .flat_map(chars_of)
        .finalize(move || {
            tracing::info!(elapsed_us = start.elapsed().as_micros() as u64, "serialize finalize")
        })
        .tap(|ch| tracing::trace!(?ch, stage = "serialize", "char out"))
        .box_it()
}

/// Map a `Result<CssNode, CompileError>` to a String.
/// - `Ok(css_node)` → rendered CSS fragment
/// - `Err(err)`     → `/* COMPILE ERROR: ... */` banner (error-as-value)
#[inline]
fn render_item(item: EvalItem) -> String {
    match item {
        Ok(css_node) => css_node.render(),
        Err(err) => format!("/* COMPILE ERROR: {err} */"),
    }
}

/// Convert a String into a char-stream Observable.
#[inline]
fn chars_of(s: String) -> LocalBoxedObservable<'static, char, Infallible> {
    Local::from_iter(s.chars().collect::<Vec<_>>()).box_it()
}
