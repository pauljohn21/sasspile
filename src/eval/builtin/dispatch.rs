//! 内建函数注册统一入口——纯转发，调用各子模块的注册函数。
//!
//! 替代旧的 `module_dispatch.rs`（宏生成版本），不依赖 proc-macro。

use crate::error::Result;
use crate::eval::Env;
use crate::parse::ast::Value;
use std::collections::HashMap;

// ─── math ─────────────────────────────────────────────────

/// math 模块限定名 → 全局名。
pub(crate) fn math_builtin_name(name: &str) -> Option<&'static str> {
    match name {
        "math.abs" => Some("abs"),
        "math.div" => Some("div"),
        "math.ceil" => Some("ceil"),
        "math.floor" => Some("floor"),
        "math.round" => Some("round"),
        "math.max" => Some("max"),
        "math.min" => Some("min"),
        "math.percentage" => Some("percentage"),
        "math.pow" => Some("pow"),
        "math.sqrt" => Some("sqrt"),
        "math.sin" => Some("sin"),
        "math.cos" => Some("cos"),
        "math.tan" => Some("tan"),
        "math.log" => Some("log"),
        "math.hypot" => Some("hypot"),
        "math.atan2" => Some("atan2"),
        "math.asin" => Some("asin"),
        "math.acos" => Some("acos"),
        "math.atan" => Some("atan"),
        "math.random" => Some("random"),
        "math.clamp" => Some("clamp"),
        "math.unit" => Some("unit"),
        "math.is-unitless" => Some("is_unitless"),
        "math.compatible" => Some("compatible"),
        "math.comparable" => Some("comparable"),
        _ => None,
    }
}

pub(crate) fn math_is_known(name: &str) -> bool {
    matches!(
        name,
        "abs" | "div" | "ceil" | "floor" | "round" | "max" | "min"
        | "percentage" | "pow" | "sqrt" | "sin" | "cos" | "tan" | "log"
        | "hypot" | "atan2" | "asin" | "acos" | "atan" | "random"
        | "clamp" | "unit" | "is_unitless" | "is-unitless" | "compatible" | "comparable"
        | "math.abs" | "math.div" | "math.ceil" | "math.floor" | "math.round"
        | "math.max" | "math.min" | "math.percentage" | "math.pow" | "math.sqrt"
        | "math.sin" | "math.cos" | "math.tan" | "math.log" | "math.hypot"
        | "math.atan2" | "math.asin" | "math.acos" | "math.atan" | "math.random"
        | "math.clamp" | "math.unit" | "math.is-unitless" | "math.compatible" | "math.comparable"
    )
}

pub(crate) fn math_dispatch(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    _env: &Env,
) -> Option<Result<Value>> {
    match name {
        "abs" | "div" | "ceil" | "floor" | "round" | "max" | "min"
        | "percentage" | "pow" | "sqrt" | "sin" | "cos" | "tan" | "log"
        | "hypot" | "atan2" | "asin" | "acos" | "atan" | "random"
        | "clamp" | "unit" | "is_unitless" | "is-unitless" | "compatible" | "comparable"
        => match super::math::call(name, pos_args, kw_args) {
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        },
        _ => None,
    }
}

// ─── string ───────────────────────────────────────────────

pub(crate) fn string_builtin_name(name: &str) -> Option<&'static str> {
    match name {
        "string.length" => Some("str-length"),
        "string.index" => Some("str-index"),
        "string.slice" => Some("str-slice"),
        "string.to-upper-case" => Some("to-upper-case"),
        "string.to-lower-case" => Some("to-lower-case"),
        "string.insert" => Some("str-insert"),
        "string.quote" => Some("quote"),
        "string.unquote" => Some("unquote"),
        "string.split" => Some("str-split"),
        "string.unique-id" => Some("unique-id"),
        _ => None,
    }
}

pub(crate) fn string_is_known(name: &str) -> bool {
    matches!(
        name,
        "str-length" | "str-index" | "str-slice" | "to-upper-case" | "to-lower-case"
        | "str-insert" | "quote" | "unquote" | "str-split" | "unique-id"
        | "string.length" | "string.index" | "string.slice" | "string.to-upper-case"
        | "string.to-lower-case" | "string.insert" | "string.quote" | "string.unquote"
        | "string.split" | "string.unique-id"
    )
}

pub(crate) fn string_dispatch(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    _env: &Env,
) -> Option<Result<Value>> {
    match name {
        "str-length" | "str-index" | "str-slice" | "to-upper-case" | "to-lower-case"
        | "str-insert" | "quote" | "unquote" | "str-split" | "unique-id"
        => match super::Evaluator::call_string_builtin(name, pos_args, kw_args) {
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        },
        _ => None,
    }
}

// ─── map ──────────────────────────────────────────────────

pub(crate) fn map_builtin_name(name: &str) -> Option<&'static str> {
    match name {
        "map.get" => Some("map-get"),
        "map.merge" => Some("map-merge"),
        "map.remove" => Some("map-remove"),
        "map.keys" => Some("map-keys"),
        "map.values" => Some("map-values"),
        "map.has-key" => Some("map-has-key"),
        "map.deep-remove" => Some("map-deep-remove"),
        "map.deep-merge" => Some("map-deep-merge"),
        "map.set" => Some("map-set"),
        _ => None,
    }
}

pub(crate) fn map_is_known(name: &str) -> bool {
    matches!(
        name,
        "map-get" | "map-merge" | "map-remove" | "map-keys" | "map-values"
        | "map-has-key" | "map-deep-remove" | "map-deep-merge" | "map-set"
        | "map.get" | "map.merge" | "map.remove" | "map.keys" | "map.values"
        | "map.has-key" | "map.deep-remove" | "map.deep-merge" | "map.set"
    )
}

pub(crate) fn map_dispatch(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    _env: &Env,
) -> Option<Result<Value>> {
    match name {
        "map-get" | "map-merge" | "map-remove" | "map-keys" | "map-values"
        | "map-has-key" | "map-deep-remove" | "map-deep-merge" | "map-set"
        => {
            let combined = super::merge_map_args(pos_args, kw_args, name);
            match super::Evaluator::call_map_builtin(name, &combined, _env) {
                Ok(Some(v)) => Some(Ok(v)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        }
        _ => None,
    }
}

// ─── list ─────────────────────────────────────────────────

pub(crate) fn list_builtin_name(name: &str) -> Option<&'static str> {
    match name {
        "list.length" => Some("length"),
        "list.nth" => Some("nth"),
        "list.append" => Some("append"),
        "list.join" => Some("join"),
        "list.index" => Some("index"),
        "list.separator" => Some("list-separator"),
        "list.set-nth" => Some("set-nth"),
        "list.is-bracketed" => Some("is-bracketed"),
        "list.slash" => Some("list-slash"),
        "list.zip" => Some("zip"),
        _ => None,
    }
}

pub(crate) fn list_is_known(name: &str) -> bool {
    matches!(
        name,
        "length" | "list-length" | "nth" | "append" | "join" | "index"
        | "list-separator" | "separator" | "set-nth" | "is-bracketed"
        | "list-slash" | "zip"
        | "list.length" | "list.nth" | "list.append" | "list.join" | "list.index"
        | "list.separator" | "list.set-nth" | "list.is-bracketed" | "list.slash" | "list.zip"
    )
}

pub(crate) fn list_dispatch(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    _env: &Env,
) -> Option<Result<Value>> {
    match name {
        "length" | "list-length" | "nth" | "append" | "join" | "index"
        | "list-separator" | "separator" | "set-nth" | "is-bracketed"
        | "list-slash" | "zip"
        => match super::list::call(name, pos_args, kw_args) {
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        },
        _ => None,
    }
}

// ─── color ────────────────────────────────────────────────

pub(crate) fn color_builtin_name(name: &str) -> Option<&'static str> {
    match name {
        "color.adjust" | "color.adjust-color" => Some("adjust-color"),
        "color.change" | "color.change-color" => Some("change-color"),
        "color.scale" | "color.scale-color" => Some("scale-color"),
        "color.ie-hex-str" => Some("ie-hex-str"),
        "color.invert" => Some("invert"),
        "color.grayscale" => Some("grayscale"),
        "color.complement" => Some("complement"),
        "color.adjust-hue" => Some("adjust-hue"),
        "color.saturate" => Some("saturate"),
        "color.desaturate" => Some("desaturate"),
        "color.transparentize" => Some("transparentize"),
        "color.fade-out" => Some("fade-out"),
        "color.opacify" => Some("opacify"),
        "color.fade-in" => Some("fade-in"),
        "color.alpha" => Some("alpha"),
        "color.opacity" => Some("opacity"),
        "color.red" => Some("red"),
        "color.green" => Some("green"),
        "color.blue" => Some("blue"),
        "color.hue" => Some("hue"),
        "color.saturation" => Some("saturation"),
        "color.lightness" => Some("lightness"),
        "color.whiteness" => Some("whiteness"),
        "color.blackness" => Some("blackness"),
        "color.is-powerless" => Some("is-powerless"),
        "color.is-missing" => Some("is-missing"),
        "color.is-in-gamut" => Some("is-in-gamut"),
        "color.is-legacy" => Some("is-legacy"),
        "color.channel" => Some("channel"),
        "color.to-space" => Some("to-space"),
        "color.to-gamut" => Some("to-gamut"),
        "color.space" => Some("space"),
        "color.same" => Some("same"),
        "color.hwb" => Some("hwb"),
        "color.hsl" => Some("hsl"),
        "color.hsla" => Some("hsla"),
        "color.rgba" => Some("rgba"),
        "color.rgb" => Some("rgb"),
        "color.darken" => Some("darken"),
        "color.lighten" => Some("lighten"),
        "color.mix" => Some("mix"),
        _ => None,
    }
}

pub(crate) fn color_is_known(name: &str) -> bool {
    matches!(
        name,
        "adjust-color" | "change-color" | "scale-color" | "ie-hex-str"
        | "invert" | "grayscale" | "complement" | "adjust-hue"
        | "saturate" | "desaturate" | "transparentize" | "fade-out"
        | "opacify" | "fade-in" | "alpha" | "opacity" | "red" | "green" | "blue"
        | "hue" | "saturation" | "lightness" | "whiteness" | "blackness"
        | "is-powerless" | "is-missing" | "is-in-gamut" | "is-legacy" | "channel"
        | "to-space" | "to-gamut" | "space" | "same" | "hwb" | "hsl" | "hsla"
        | "color_channel"
        | "rgba" | "rgb" | "darken" | "lighten" | "mix"
        | "color.adjust" | "color.adjust-color" | "color.change" | "color.change-color"
        | "color.scale" | "color.scale-color" | "color.ie-hex-str" | "color.invert"
        | "color.grayscale" | "color.complement" | "color.adjust-hue" | "color.saturate"
        | "color.desaturate" | "color.transparentize" | "color.fade-out" | "color.opacify"
        | "color.fade-in" | "color.alpha" | "color.opacity" | "color.red" | "color.green"
        | "color.blue" | "color.hue" | "color.saturation" | "color.lightness" | "color.whiteness"
        | "color.blackness" | "color.is-powerless" | "color.is-missing" | "color.is-in-gamut"
        | "color.is-legacy" | "color.channel" | "color.to-space" | "color.to-gamut"
        | "color.space" | "color.same" | "color.hwb" | "color.hsl" | "color.hsla"
        | "color.rgba" | "color.rgb" | "color.darken" | "color.lighten" | "color.mix"
    )
}

pub(crate) fn color_dispatch(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    _env: &Env,
) -> Option<Result<Value>> {
    match name {
        "adjust-color" | "change-color" | "scale-color" | "ie-hex-str"
        | "invert" | "grayscale" | "complement" | "adjust-hue"
        | "saturate" | "desaturate" | "transparentize" | "fade-out"
        | "opacify" | "fade-in" | "alpha" | "opacity" | "red" | "green" | "blue"
        | "hue" | "saturation" | "lightness" | "whiteness" | "blackness"
        | "is-powerless" | "is-missing" | "is-in-gamut" | "is-legacy" | "channel"
        | "to-space" | "to-gamut" | "space" | "same" | "hwb" | "hsl" | "hsla" | "color_channel"
        => match super::color::call(name, pos_args, kw_args) {
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        },
        _ => None,
    }
}

// ─── selector ─────────────────────────────────────────────

pub(crate) fn selector_builtin_name(name: &str) -> Option<&'static str> {
    match name {
        "selector.append" => Some("selector-append"),
        "selector.nest" => Some("selector-nest"),
        "selector.is-superselector" => Some("selector-is-superselector"),
        "selector.parse" => Some("selector-parse"),
        "selector.simple-selectors" => Some("selector-simple-selectors"),
        "selector.unify" => Some("selector-unify"),
        "selector.extend" => Some("selector-extend"),
        "selector.replace" => Some("selector-replace"),
        _ => None,
    }
}

pub(crate) fn selector_is_known(name: &str) -> bool {
    matches!(
        name,
        "selector-append" | "selector-is-super" | "selector-nest"
        | "selector-is-superselector" | "selector-parse" | "selector-simple-selectors"
        | "selector-unify" | "selector-extend" | "selector-replace"
        | "selector.append" | "selector.nest" | "selector.is-superselector"
        | "selector.parse" | "selector.simple-selectors" | "selector.unify"
        | "selector.extend" | "selector.replace"
    )
}

pub(crate) fn selector_dispatch(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    _env: &Env,
) -> Option<Result<Value>> {
    match name {
        "selector-append" | "selector-is-super" | "selector-nest"
        | "selector-is-superselector" | "selector-parse" | "selector-simple-selectors"
        | "selector-unify" | "selector-extend" | "selector-replace"
        => match super::selector::call(name, pos_args, kw_args) {
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        },
        _ => None,
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
    match name {
        "meta.type-of" => Some("type-of"),
        "meta.inspect" => Some("inspect"),
        "meta.keywords" => Some("keywords"),
        "meta.get-function" => Some("get-function"),
        "meta.call" => Some("call"),
        "meta.feature-exists" => Some("feature-exists"),
        "meta.content-exists" => Some("content-exists"),
        "meta.mixin-exists" => Some("mixin-exists"),
        "meta.function-exists" => Some("function-exists"),
        "meta.global-variable-exists" => Some("global-variable-exists"),
        "meta.variable-exists" => Some("variable-exists"),
        "meta.calc-args" => Some("calc-args"),
        "meta.calc-name" => Some("calc-name"),
        "meta.get-mixin" => Some("get-mixin"),
        "meta.module-functions" => Some("module-functions"),
        "meta.module-mixins" => Some("module-mixins"),
        "meta.module-variables" => Some("module-variables"),
        "meta.accepts-content" => Some("accepts-content"),
        _ => None,
    }
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
        || matches!(name, "calc" | "env" | "var")
}

/// meta 模块函数名检查（dispatch = "none"：只参与名称映射，不分派）。
fn meta_is_known(name: &str) -> bool {
    matches!(
        name,
        "type-of" | "type_of" | "inspect" | "keywords"
        | "get-function" | "get_function" | "call"
        | "feature-exists" | "feature_exists"
        | "content-exists" | "content_exists"
        | "mixin-exists" | "mixin_exists"
        | "function-exists" | "function_exists"
        | "global-variable-exists" | "global_variable_exists"
        | "variable-exists" | "variable_exists"
        | "calc-args" | "calc_args" | "calc-name" | "calc_name"
        | "get-mixin" | "get_mixin"
        | "module-functions" | "module_functions"
        | "module-mixins" | "module_mixins"
        | "module-variables" | "module_variables"
        | "accepts-content" | "accepts_content"
        | "meta.type-of" | "meta.inspect" | "meta.keywords"
        | "meta.get-function" | "meta.call" | "meta.feature-exists"
        | "meta.content-exists" | "meta.mixin-exists" | "meta.function-exists"
        | "meta.global-variable-exists" | "meta.variable-exists"
        | "meta.calc-args" | "meta.calc-name" | "meta.get-mixin"
        | "meta.module-functions" | "meta.module-mixins" | "meta.module-variables"
        | "meta.accepts-content"
    )
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
