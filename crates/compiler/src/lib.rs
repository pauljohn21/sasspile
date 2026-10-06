//! Lightforger / 铸光者 — Reactive Sass compiler built with rxrust.
//!
//! Compiler pipeline: Lexer → Parser → Evaluator → Serializer
//! All stages communicate via Observable streams and a multicast CompilerBus.

#![warn(clippy::all, clippy::cargo, clippy::dbg_macro)]
#![deny(missing_debug_implementations)]

pub mod error;
pub mod reactive;
pub mod lexer;
pub mod parser;
pub mod lowering;
pub mod builtin;

pub use error::{Error, Result};

/// Scope stub — 模块系统占位
#[derive(Debug, Default)]
pub struct Scope {
    // TODO: implement scope
}
