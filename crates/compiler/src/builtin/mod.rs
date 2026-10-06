//! Built-in Sass modules — sass:color, sass:math, sass:string, sass:list, sass:map, etc.

mod color;
mod math;
mod string;
mod list;
mod map;

use crate::Scope;

/// Register all built-in functions into the given scope
pub fn register_all(_scope: &mut Scope) {
    // TODO: register each module's functions
}
