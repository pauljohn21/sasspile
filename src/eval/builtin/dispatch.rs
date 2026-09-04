//! 内建函数注册统一入口——纯转发，调用各子模块的注册函数。
//!
//! 替代旧的 `module_dispatch.rs`（宏生成版本），不依赖 proc-macro。

use crate::error::Result;
use crate::eval::Env;
use crate::parse::ast::Value;
use std::collections::HashMap;

/// 内建函数映射条目：`(模块限定名, 全局名)`。
///
/// 每个模块的 const 数组集中管理，消除 `builtin_name` / `is_known` / `dispatch` 三处重复。
const MATH_NAMES: &[(&str, &str)] = &[
    ("math.abs", "abs"),
    ("math.div", "div"),
    ("math.ceil", "ceil"),
    ("math.floor", "floor"),
    ("math.round", "round"),
    ("math.max", "max"),
    ("math.min", "min"),
    ("math.percentage", "percentage"),
    ("math.pow", "pow"),
    ("math.sqrt", "sqrt"),
    ("math.sin", "sin"),
    ("math.cos", "cos"),
    ("math.tan", "tan"),
    ("math.log", "log"),
    ("math.hypot", "hypot"),
    ("math.atan2", "atan2"),
    ("math.asin", "asin"),
    ("math.acos", "acos"),
    ("math.atan", "atan"),
    ("math.random", "random"),
    ("math.clamp", "clamp"),
    ("math.unit", "unit"),
    ("math.is-unitless", "is-unitless"),
    ("math.compatible", "compatible"),
    ("math.comparable", "comparable"),
    ("math.mod", "mod"),
    ("math.rem", "rem"),
];

const STRING_NAMES: &[(&str, &str)] = &[
    ("string.length", "str-length"),
    ("string.index", "str-index"),
    ("string.slice", "str-slice"),
    ("string.insert", "str-insert"),
    ("string.split", "str-split"),
    ("string.to-upper-case", "to-upper-case"),
    ("string.to-lower-case", "to-lower-case"),
    ("string.quote", "quote"),
    ("string.unquote", "unquote"),
    ("string.unique-id", "unique-id"),
];

const MAP_NAMES: &[(&str, &str)] = &[
    ("map.get", "map-get"),
    ("map.merge", "map-merge"),
    ("map.remove", "map-remove"),
    ("map.keys", "map-keys"),
    ("map.values", "map-values"),
    ("map.has-key", "map-has-key"),
    ("map.deep-remove", "map-deep-remove"),
    ("map.deep-merge", "map-deep-merge"),
    ("map.set", "map-set"),
];

const LIST_NAMES: &[(&str, &str)] = &[
    ("list.length", "length"),
    ("list.nth", "nth"),
    ("list.append", "append"),
    ("list.join", "join"),
    ("list.index", "index"),
    ("list.separator", "list-separator"),
    ("list.set-nth", "set-nth"),
    ("list.is-bracketed", "is-bracketed"),
    ("list.slash", "list-slash"),
    ("list.zip", "zip"),
];

const COLOR_NAMES: &[(&str, &str)] = &[
    ("color.adjust", "adjust-color"),
    ("color.change", "change-color"),
    ("color.scale", "scale-color"),
    ("color.ie-hex-str", "ie-hex-str"),
    ("color.invert", "invert"),
    ("color.grayscale", "grayscale"),
    ("color.complement", "complement"),
    ("color.adjust-hue", "adjust-hue"),
    ("color.saturate", "saturate"),
    ("color.desaturate", "desaturate"),
    ("color.transparentize", "transparentize"),
    ("color.fade-out", "fade-out"),
    ("color.opacify", "opacify"),
    ("color.fade-in", "fade-in"),
    ("color.alpha", "alpha"),
    ("color.opacity", "opacity"),
    ("color.red", "red"),
    ("color.green", "green"),
    ("color.blue", "blue"),
    ("color.hue", "hue"),
    ("color.saturation", "saturation"),
    ("color.lightness", "lightness"),
    ("color.whiteness", "whiteness"),
    ("color.blackness", "blackness"),
    ("color.is-powerless", "is-powerless"),
    ("color.is-missing", "is-missing"),
    ("color.is-in-gamut", "is-in-gamut"),
    ("color.is-legacy", "is-legacy"),
    ("color.channel", "channel"),
    ("color.to-space", "to-space"),
    ("color.to-gamut", "to-gamut"),
    ("color.space", "space"),
    ("color.same", "same"),
    ("color.hwb", "hwb"),
    ("color.hsl", "hsl"),
    ("color.hsla", "hsla"),
    ("color.rgba", "rgba"),
    ("color.rgb", "rgb"),
    ("color.darken", "darken"),
    ("color.lighten", "lighten"),
    ("color.mix", "mix"),
];

const SELECTOR_NAMES: &[(&str, &str)] = &[
    ("selector.is-superselector", "selector-is-superselector"),
    ("selector.parse", "selector-parse"),
    ("selector.simple-selectors", "selector-simple-selectors"),
    ("selector.unify", "selector-unify"),
    ("selector.extend", "selector-extend"),
    ("selector.replace", "selector-replace"),
    ("selector.append", "selector-append"),
    ("selector.nest", "selector-nest"),
];

const META_NAMES: &[(&str, &str)] = &[
    ("meta.type-of", "type-of"),
    ("meta.inspect", "inspect"),
    ("meta.keywords", "keywords"),
    ("meta.get-function", "get-function"),
    ("meta.call", "call"),
    ("meta.feature-exists", "feature-exists"),
    ("meta.content-exists", "content-exists"),
    ("meta.mixin-exists", "mixin-exists"),
    ("meta.function-exists", "function-exists"),
    ("meta.global-variable-exists", "global-variable-exists"),
    ("meta.variable-exists", "variable-exists"),
    ("meta.calc-args", "calc-args"),
    ("meta.calc-name", "calc-name"),
    ("meta.get-mixin", "get-mixin"),
    ("meta.module-functions", "module-functions"),
    ("meta.module-mixins", "module-mixins"),
    ("meta.module-variables", "module-variables"),
    ("meta.accepts-content", "accepts-content"),
];

/// CSS 通用函数名（无模块前缀变体）。
const CSS_FUNC_NAMES: &[&str] = &["calc", "env", "var"];

// ─── math ─────────────────────────────────────────────────

pub(crate) fn math_builtin_name(name: &str) -> Option<&'static str> {
    MATH_NAMES.iter().find(|(k, _)| *k == name).map(|(_, v)| *v)
}

pub(crate) fn math_is_known(name: &str) -> bool {
    MATH_NAMES.iter().any(|(k, v)| *k == name || *v == name)
}

pub(crate) fn math_dispatch(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    _env: &Env,
) -> Option<Result<Value>> {
    match math_is_known(name) {
        true => match super::math::call(name, pos_args, kw_args) {
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        },
        false => None,
    }
}

// ─── string ───────────────────────────────────────────────

pub(crate) fn string_builtin_name(name: &str) -> Option<&'static str> {
    STRING_NAMES
        .iter()
        .find(|(k, _)| *k == name)
        .map(|(_, v)| *v)
}

pub(crate) fn string_is_known(name: &str) -> bool {
    STRING_NAMES.iter().any(|(k, v)| *k == name || *v == name)
}

pub(crate) fn string_dispatch(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    _env: &Env,
) -> Option<Result<Value>> {
    match string_is_known(name) {
        true => match super::Evaluator::call_string_builtin(name, pos_args, kw_args) {
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        },
        false => None,
    }
}

// ─── map ──────────────────────────────────────────────────

pub(crate) fn map_builtin_name(name: &str) -> Option<&'static str> {
    MAP_NAMES.iter().find(|(k, _)| *k == name).map(|(_, v)| *v)
}

pub(crate) fn map_is_known(name: &str) -> bool {
    MAP_NAMES.iter().any(|(k, v)| *k == name || *v == name)
}

pub(crate) fn map_dispatch(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    env: &Env,
) -> Option<Result<Value>> {
    match map_is_known(name) {
        true => {
            let combined = super::merge_map_args(pos_args, kw_args, name);
            match super::Evaluator::call_map_builtin(name, &combined, env) {
                Ok(Some(v)) => Some(Ok(v)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        }
        false => None,
    }
}

// ─── list ─────────────────────────────────────────────────

pub(crate) fn list_builtin_name(name: &str) -> Option<&'static str> {
    LIST_NAMES.iter().find(|(k, _)| *k == name).map(|(_, v)| *v)
}

pub(crate) fn list_is_known(name: &str) -> bool {
    LIST_NAMES.iter().any(|(k, v)| *k == name || *v == name)
}

pub(crate) fn list_dispatch(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    _env: &Env,
) -> Option<Result<Value>> {
    match list_is_known(name) {
        true => match super::list::call(name, pos_args, kw_args) {
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        },
        false => None,
    }
}

// ─── color ────────────────────────────────────────────────

pub(crate) fn color_builtin_name(name: &str) -> Option<&'static str> {
    COLOR_NAMES
        .iter()
        .find(|(k, _)| *k == name)
        .map(|(_, v)| *v)
}

pub(crate) fn color_is_known(name: &str) -> bool {
    COLOR_NAMES.iter().any(|(k, v)| *k == name || *v == name) || name == "color_channel"
}

pub(crate) fn color_dispatch(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    _env: &Env,
) -> Option<Result<Value>> {
    match color_is_known(name) {
        true => match super::color::call(name, pos_args, kw_args) {
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        },
        false => None,
    }
}

// ─── selector ─────────────────────────────────────────────

pub(crate) fn selector_builtin_name(name: &str) -> Option<&'static str> {
    SELECTOR_NAMES
        .iter()
        .find(|(k, _)| *k == name)
        .map(|(_, v)| *v)
}

pub(crate) fn selector_is_known(name: &str) -> bool {
    SELECTOR_NAMES.iter().any(|(k, v)| *k == name || *v == name)
}

pub(crate) fn selector_dispatch(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    _env: &Env,
) -> Option<Result<Value>> {
    match selector_is_known(name) {
        true => match super::selector::call(name, pos_args, kw_args) {
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        },
        false => None,
    }
}

// ─── 统一入口函数 ─────────────────────────────────────────

/// 将模块限定名（如 `math.abs`）映射到内建函数名（如 `abs`）。
/// 未匹配的名称原样返回。
pub(crate) fn module_builtin_name(name: &str) -> &str {
    math_builtin_name(name)
        .or_else(|| string_builtin_name(name))
        .or_else(|| map_builtin_name(name))
        .or_else(|| list_builtin_name(name))
        .or_else(|| color_builtin_name(name))
        .or_else(|| selector_builtin_name(name))
        .or_else(|| meta_builtin_name(name))
        .unwrap_or(name)
}

/// meta 模块限定名 → 全局名（dispatch = "none"：只参与名称映射）。
fn meta_builtin_name(name: &str) -> Option<&'static str> {
    META_NAMES.iter().find(|(k, _)| *k == name).map(|(_, v)| *v)
}

/// 检查函数名是否为已知的 Sass 内置函数。
pub(crate) fn is_known_builtin(name: &str) -> bool {
    math_is_known(name)
        || string_is_known(name)
        || map_is_known(name)
        || list_is_known(name)
        || color_is_known(name)
        || selector_is_known(name)
        || meta_is_known(name)
        || CSS_FUNC_NAMES.contains(&name)
}

/// meta 模块函数名检查（dispatch = "none"：只参与名称映射，不分派）。
fn meta_is_known(name: &str) -> bool {
    META_NAMES.iter().any(|(k, v)| *k == name || *v == name)
        || META_NAMES.iter().any(|(_, v)| v.replace('-', "_") == name)
}

/// 按模块路由到子模块 call 函数。
/// 返回 `Some(Ok(value))` 表示已分派成功，
/// 返回 `Some(Err(...))` 表示分派目标已匹配但执行出错，
/// 返回 `None` 表示未匹配（调用方继续手工分派）。
pub(crate) fn dispatch_builtin_module(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    env: &Env,
) -> Option<Result<Value>> {
    math_dispatch(name, pos_args, kw_args, env)
        .or_else(|| string_dispatch(name, pos_args, kw_args, env))
        .or_else(|| map_dispatch(name, pos_args, kw_args, env))
        .or_else(|| list_dispatch(name, pos_args, kw_args, env))
        .or_else(|| color_dispatch(name, pos_args, kw_args, env))
        .or_else(|| selector_dispatch(name, pos_args, kw_args, env))
}
