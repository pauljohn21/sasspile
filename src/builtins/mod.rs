//! Built-in function modules.
//!
//! Each Sass built-in module (sass:math, sass:string, etc.)
//! registers its functions here.

pub mod helpers;
pub mod math;
pub mod string;
pub mod list;
pub mod map;
pub mod color;
pub mod meta;
pub mod selector_builtins;

use crate::env::Env;

/// Register all builtin functions into the environment.
/// Called at the start of evaluation to make builtins available.
pub fn register_all(env: &mut Env) {
    let span = tracing::info_span!("register_builtins", stage = "init", module = "builtins");
    let _enter = span.enter();

    math::register(env);
    string::register(env);
    list::register(env);
    map::register(env);
    color::register(env);
    meta::register(env);
    selector_builtins::register(env);

    tracing::debug!(stage = "init", module = "builtins", "all builtin modules registered");
}
