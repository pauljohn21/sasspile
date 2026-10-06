//! Built-in Sass modules — sass:color, sass:math, sass:string, sass:list, sass:map, etc.

pub mod color;
pub mod math;
pub mod string;
pub mod list;
pub mod map;

// Re-export Scope so tests can access it via `builtin::Scope`
pub use crate::Scope;

/// Register all built-in functions into the given scope.
///
/// Covers:
/// - color: darken, lighten, mix, opacify, transparentize
/// - math: clamp, max, min, round, abs, percentage
/// - string: index, length, slice, to-upper-case, to-lower-case
/// - list: length, nth, append, index, join
/// - map: get, has-key, keys, values, merge, remove
pub fn register_all(scope: &mut Scope) {
    // sass:color
    scope.register("color.darken", color::darken);
    scope.register("color.lighten", color::lighten);
    scope.register("color.mix", color::mix);
    scope.register("color.opacify", color::opacify);
    scope.register("color.transparentize", color::transparentize);

    // sass:math
    scope.register("math.clamp", math::clamp);
    scope.register("math.max", math::max);
    scope.register("math.min", math::min);
    scope.register("math.round", math::round);
    scope.register("math.abs", math::abs);
    scope.register("math.percentage", math::percentage);

    // sass:string
    scope.register("string.index", string::index);
    scope.register("string.length", string::length);
    scope.register("string.slice", string::slice);
    scope.register("string.to-upper-case", string::to_upper_case);
    scope.register("string.to-lower-case", string::to_lower_case);

    // sass:list
    scope.register("list.length", list::length);
    scope.register("list.nth", list::nth);
    scope.register("list.append", list::append);
    scope.register("list.index", list::index);
    scope.register("list.join", list::join);

    // sass:map
    scope.register("map.get", map::get);
    scope.register("map.has-key", map::has_key);
    scope.register("map.keys", map::keys);
    scope.register("map.values", map::values);
    scope.register("map.merge", map::merge);
    scope.register("map.remove", map::remove);
}
