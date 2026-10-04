//! Reactive compilation pipeline entry points.

pub mod compile;

pub use compile::{compile_ast, from_string, from_string_ast};
