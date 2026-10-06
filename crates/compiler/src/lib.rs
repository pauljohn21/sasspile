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

use std::collections::HashMap;

/// Builtin function signature: takes a slice of args, returns a Value or error.
pub type BuiltinFn = fn(&[Value]) -> std::result::Result<Value, Error>;

/// Scope — holds registered builtin functions.
#[derive(Debug)]
pub struct Scope {
    functions: HashMap<String, BuiltinFn>,
}

impl Scope {
    /// Create a fresh scope with all builtins registered.
    pub fn new() -> Self {
        let mut scope = Self {
            functions: HashMap::new(),
        };
        builtin::register_all(&mut scope);
        scope
    }

    /// Register a builtin function under the given name.
    pub fn register(&mut self, name: impl Into<String>, f: BuiltinFn) {
        self.functions.insert(name.into(), f);
    }

    /// Call a builtin function by name with the given arguments.
    pub fn call(&self, name: &str, args: &[Value]) -> std::result::Result<Value, Error> {
        match self.functions.get(name) {
            Some(f) => f(args),
            None => Err(Error::eval(format!("unknown builtin function: {name}"))),
        }
    }

    /// Returns true if a function with the given name is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
}

pub use reactive::Value;
