//! Lightforger / 铸光者 — Reactive Sass compiler built with rxrust.
//!
//! Compiler pipeline: Lexer → Parser → Evaluator → Serializer
//! All stages communicate via Observable streams and a multicast CompilerBus.

#![warn(clippy::all, clippy::cargo, clippy::dbg_macro)]
#![deny(missing_debug_implementations)]

pub mod error;
pub mod reactive;

pub use error::{Error, Result};
