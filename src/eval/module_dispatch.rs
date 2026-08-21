//! 模块限定函数名 → 内建函数名映射 + 内建函数注册表。
//!
//! 通过 `#[derive(BuiltinRegistry)]` 将函数名映射集中到结构体声明，
//! 宏自动生成 `__<struct>_module_builtin_name`、`__<struct>_is_known`、
//! `__<struct>_dispatch` 三个 `#[doc(hidden)]` 函数。
//!
//! 本文件中的三个统一函数依次调用各结构体的生成函数。

use crate::error::Result;
use crate::eval::Env;
use crate::parse::ast::Value;
use std::collections::HashMap;
use sasspile_macros::BuiltinRegistry;

// ─── math ─────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(BuiltinRegistry)]
#[builtin(module = "math", dispatch = "math")]
struct MathBuiltins {
    abs: (),
    #[builtin(alias = "math.div")]
    div: (),
    ceil: (),
    floor: (),
    round: (),
    max: (),
    min: (),
    percentage: (),
    pow: (),
    sqrt: (),
    sin: (),
    cos: (),
    tan: (),
    log: (),
    hypot: (),
    atan2: (),
    asin: (),
    acos: (),
    atan: (),
    random: (),
    clamp: (),
    unit: (),
    is_unitless: (),
    #[builtin(alias = "math.compatible")]
    compatible: (),
    comparable: (),
}

// ─── string ───────────────────────────────────────────────

#[allow(dead_code)]
#[derive(BuiltinRegistry)]
#[builtin(module = "string", dispatch = "string")]
struct StringBuiltins {
    #[builtin(alias = "string.length")]
    str_length: (),
    #[builtin(alias = "string.index")]
    str_index: (),
    #[builtin(alias = "string.slice")]
    str_slice: (),
    #[builtin(alias = "string.to-upper-case")]
    to_upper_case: (),
    #[builtin(alias = "string.to-lower-case")]
    to_lower_case: (),
    #[builtin(alias = "string.insert")]
    str_insert: (),
    #[builtin(alias = "string.quote")]
    quote: (),
    #[builtin(alias = "string.unquote")]
    unquote: (),
    #[builtin(alias = "string.split")]
    str_split: (),
    #[builtin(alias = "string.unique-id")]
    unique_id: (),
}

// ─── map ──────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(BuiltinRegistry)]
#[builtin(module = "map", dispatch = "map")]
struct MapBuiltins {
    #[builtin(alias = "map.get")]
    map_get: (),
    #[builtin(alias = "map.merge")]
    map_merge: (),
    #[builtin(alias = "map.remove")]
    map_remove: (),
    #[builtin(alias = "map.keys")]
    map_keys: (),
    #[builtin(alias = "map.values")]
    map_values: (),
    #[builtin(alias = "map.has-key")]
    map_has_key: (),
    #[builtin(alias = "map.deep-remove")]
    map_deep_remove: (),
    #[builtin(alias = "map.deep-merge")]
    map_deep_merge: (),
    #[builtin(alias = "map.set")]
    map_set: (),
}

// ─── list ─────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(BuiltinRegistry)]
#[builtin(module = "list", dispatch = "list")]
struct ListBuiltins {
    #[builtin(alias = "list.length")]
    length: (),
    list_length: (),
    #[builtin(alias = "list.nth")]
    nth: (),
    #[builtin(alias = "list.append")]
    append: (),
    #[builtin(alias = "list.join")]
    join: (),
    #[builtin(alias = "list.index")]
    index: (),
    #[builtin(alias = "list.separator")]
    list_separator: (),
    separator: (),
    #[builtin(alias = "list.set-nth")]
    set_nth: (),
    #[builtin(alias = "list.is-bracketed")]
    is_bracketed: (),
    #[builtin(alias = "list.slash")]
    list_slash: (),
    #[builtin(alias = "list.zip")]
    zip: (),
}

// ─── color ────────────────────────────────────────────────
// 排除 rgba/rgb/darken/lighten/mix（手工 arm，调用 Self::builtin_*）

#[allow(dead_code)]
#[derive(BuiltinRegistry)]
#[builtin(module = "color", dispatch = "color")]
struct ColorBuiltins {
    #[builtin(alias = "color.adjust", alias = "color.adjust-color")]
    adjust_color: (),
    #[builtin(alias = "color.change", alias = "color.change-color")]
    change_color: (),
    #[builtin(alias = "color.scale", alias = "color.scale-color")]
    scale_color: (),
    #[builtin(alias = "color.ie-hex-str")]
    ie_hex_str: (),
    #[builtin(alias = "color.invert")]
    invert: (),
    #[builtin(alias = "color.grayscale")]
    grayscale: (),
    #[builtin(alias = "color.complement")]
    complement: (),
    #[builtin(alias = "color.adjust-hue")]
    adjust_hue: (),
    #[builtin(alias = "color.saturate")]
    saturate: (),
    #[builtin(alias = "color.desaturate")]
    desaturate: (),
    #[builtin(alias = "color.transparentize")]
    transparentize: (),
    #[builtin(alias = "color.fade-out")]
    fade_out: (),
    #[builtin(alias = "color.opacify")]
    opacify: (),
    #[builtin(alias = "color.fade-in")]
    fade_in: (),
    #[builtin(alias = "color.alpha")]
    alpha: (),
    #[builtin(alias = "color.opacity")]
    opacity: (),
    #[builtin(alias = "color.red")]
    red: (),
    #[builtin(alias = "color.green")]
    green: (),
    #[builtin(alias = "color.blue")]
    blue: (),
    #[builtin(alias = "color.hue")]
    hue: (),
    #[builtin(alias = "color.saturation")]
    saturation: (),
    #[builtin(alias = "color.lightness")]
    lightness: (),
    #[builtin(alias = "color.whiteness")]
    whiteness: (),
    #[builtin(alias = "color.blackness")]
    blackness: (),
    #[builtin(alias = "color.is-powerless")]
    is_powerless: (),
    #[builtin(alias = "color.is-missing")]
    is_missing: (),
    #[builtin(alias = "color.is-in-gamut")]
    is_in_gamut: (),
    #[builtin(alias = "color.is-legacy")]
    is_legacy: (),
    #[builtin(alias = "color.channel")]
    channel: (),
    #[builtin(alias = "color.to-space")]
    to_space: (),
    #[builtin(alias = "color.to-gamut")]
    to_gamut: (),
    #[builtin(alias = "color.space")]
    space: (),
    #[builtin(alias = "color.same")]
    same: (),
    #[builtin(alias = "color.hwb")]
    hwb: (),
    #[builtin(alias = "color.hsl")]
    hsl: (),
    #[builtin(alias = "color.hsla")]
    hsla: (),
    color_channel: (),
}

// ─── meta ─────────────────────────────────────────────────
// dispatch = "none"：只参与名称映射和 is_known，不分派
// （meta 函数走 call_builtin 手工 arm）

#[allow(dead_code)]
#[derive(BuiltinRegistry)]
#[builtin(module = "meta", dispatch = "none")]
struct MetaBuiltins {
    #[builtin(alias = "meta.type-of")]
    type_of: (),
    #[builtin(alias = "meta.inspect")]
    inspect: (),
    #[builtin(alias = "meta.keywords")]
    keywords: (),
    #[builtin(alias = "meta.get-function")]
    get_function: (),
    #[builtin(alias = "meta.call")]
    call: (),
    #[builtin(alias = "meta.feature-exists")]
    feature_exists: (),
    #[builtin(alias = "meta.content-exists")]
    content_exists: (),
    #[builtin(alias = "meta.mixin-exists")]
    mixin_exists: (),
    #[builtin(alias = "meta.function-exists")]
    function_exists: (),
    #[builtin(alias = "meta.global-variable-exists")]
    global_variable_exists: (),
    #[builtin(alias = "meta.variable-exists")]
    variable_exists: (),
    #[builtin(alias = "meta.calc-args")]
    calc_args: (),
    #[builtin(alias = "meta.calc-name")]
    calc_name: (),
    #[builtin(alias = "meta.get-mixin")]
    get_mixin: (),
    #[builtin(alias = "meta.module-functions")]
    module_functions: (),
    #[builtin(alias = "meta.module-mixins")]
    module_mixins: (),
    #[builtin(alias = "meta.module-variables")]
    module_variables: (),
    #[builtin(alias = "meta.accepts-content")]
    accepts_content: (),
}

// ─── selector ─────────────────────────────────────────────

#[allow(dead_code)]
#[derive(BuiltinRegistry)]
#[builtin(module = "selector", dispatch = "selector")]
struct SelectorBuiltins {
    #[builtin(alias = "selector.append")]
    selector_append: (),
    #[builtin(alias = "selector.nest")]
    selector_nest: (),
    #[builtin(alias = "selector.is-superselector")]
    selector_is_superselector: (),
    selector_is_super: (),
    #[builtin(alias = "selector.parse")]
    selector_parse: (),
    #[builtin(alias = "selector.simple-selectors")]
    selector_simple_selectors: (),
    #[builtin(alias = "selector.unify")]
    selector_unify: (),
    #[builtin(alias = "selector.extend")]
    selector_extend: (),
    #[builtin(alias = "selector.replace")]
    selector_replace: (),
}

// ─── 统一入口函数 ─────────────────────────────────────────

/// 将模块限定名（如 `math.abs`）映射到内建函数名（如 `abs`）。
/// 未匹配的名称原样返回。
pub(crate) fn module_builtin_name(name: &str) -> &str {
    // ── 手工保留的模块限定名（不走宏分派）──
    if name == "color.rgba" { return "rgba"; }
    if name == "color.rgb" { return "rgb"; }
    if name == "color.darken" { return "darken"; }
    if name == "color.lighten" { return "lighten"; }
    if name == "color.mix" { return "mix"; }
    // ── 宏生成的模块限定名 ──
    if let Some(mapped) = __mathbuiltins_module_builtin_name(name) {
        return mapped;
    }
    if let Some(mapped) = __stringbuiltins_module_builtin_name(name) {
        return mapped;
    }
    if let Some(mapped) = __mapbuiltins_module_builtin_name(name) {
        return mapped;
    }
    if let Some(mapped) = __listbuiltins_module_builtin_name(name) {
        return mapped;
    }
    if let Some(mapped) = __colorbuiltins_module_builtin_name(name) {
        return mapped;
    }
    if let Some(mapped) = __metabuiltins_module_builtin_name(name) {
        return mapped;
    }
    if let Some(mapped) = __selectorbuiltins_module_builtin_name(name) {
        return mapped;
    }
    name
}

/// 检查函数名是否为已知的 Sass 内置函数。
pub(crate) fn is_known_builtin(name: &str) -> bool {
    __mathbuiltins_is_known(name)
        || __stringbuiltins_is_known(name)
        || __mapbuiltins_is_known(name)
        || __listbuiltins_is_known(name)
        || __colorbuiltins_is_known(name)
        || __metabuiltins_is_known(name)
        || __selectorbuiltins_is_known(name)
        || matches!(name, "calc" | "env" | "var")
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
    __mathbuiltins_dispatch(name, pos_args, kw_args, env)
        .or_else(|| __stringbuiltins_dispatch(name, pos_args, kw_args, env))
        .or_else(|| __mapbuiltins_dispatch(name, pos_args, kw_args, env))
        .or_else(|| __listbuiltins_dispatch(name, pos_args, kw_args, env))
        .or_else(|| __colorbuiltins_dispatch(name, pos_args, kw_args, env))
        .or_else(|| __selectorbuiltins_dispatch(name, pos_args, kw_args, env))
}
