//! Built-in Sass modules — sass:color, sass:math, sass:list,
//! sass:map, sass:string, sass:meta.
//!
//! Each module exposes a set of functions accessible via the
//! `@use "sass:<name>"` namespace mechanism.

pub mod sass_color;
pub mod sass_list;
pub mod sass_map;
pub mod sass_math;
pub mod sass_meta;
pub mod sass_string;

use crate::eval::error::EvalError;
use crate::parser::Expr;
use crate::value::Value;

/// Result type for built-in function calls.
pub type BuiltinResult = Result<Value, EvalError>;

/// Dispatch a built-in function call.
///
/// Supports fully-qualified names like `color.adjust-hue` or
/// `math.sin` (the `sass:` prefix is stripped before calling).
pub fn dispatch(
    name: &str,
    args: &[Expr],
    ctx: &mut crate::eval::EvalContext<'_>,
) -> Result<Option<Value>, EvalError> {
    // Parse module.function format.
    let (module, func) = if let Some(dot) = name.find('.') {
        (&name[..dot], &name[dot + 1..])
    } else {
        // No namespace — try core fallback via _.
        return Ok(None);
    };

    match module {
        "color" => sass_color::call(func, args, ctx),
        "math" => sass_math::call(func, args, ctx),
        "list" => sass_list::call(func, args, ctx),
        "map" => sass_map::call(func, args, ctx),
        "string" => sass_string::call(func, args, ctx),
        "meta" => sass_meta::call(func, args, ctx),
        _ => Ok(None),
    }
}

/// All registered module names (for documentation/introspection).
pub const MODULE_NAMES: &[&str] = &[
    "color", "math", "list", "map", "string", "meta",
];
