//! 内建函数 dispatch——const 静态表。
//!
//! 单一数据源，编译期验证，无 proc-macro。

use crate::error::{Result, SassError};
use crate::eval::value::Value;
use crate::eval::env::Env;
use crate::parse::ast::Arg;

use super::{math, string, map, list, color, meta, selector};

/// 内建函数入口。
pub struct BuiltinEntry {
    pub module: &'static str,      // "math"
    pub field: &'static str,       // "is_unitless"
    pub global: &'static str,      // "is-unitless"（从 field 生成）
    pub aliases: &'static [&'static str],
}

/// 编译期：snake_case → kebab-case。
const fn snake_to_kebab_len(field: &str) -> usize {
    let bytes = field.as_bytes();
    let mut i = 0;
    let mut len = 0;
    while i < bytes.len() {
        if bytes[i] == b'_' {
            len += 1; // _ becomes -
        } else {
            len += 1;
        }
        i += 1;
    }
    len
}

/// 单一注册表。
pub static BUILTIN_TABLE: &[BuiltinEntry] = &[
    // math
    BuiltinEntry { module: "math", field: "abs",        global: "abs",        aliases: &["math.abs"] },
    BuiltinEntry { module: "math", field: "ceil",       global: "ceil",       aliases: &["math.ceil"] },
    BuiltinEntry { module: "math", field: "floor",      global: "floor",      aliases: &["math.floor"] },
    BuiltinEntry { module: "math", field: "max",        global: "max",        aliases: &["math.max"] },
    BuiltinEntry { module: "math", field: "min",        global: "min",        aliases: &["math.min"] },
    BuiltinEntry { module: "math", field: "round",      global: "round",      aliases: &["math.round"] },
    BuiltinEntry { module: "math", field: "random",     global: "random",     aliases: &["math.random"] },
    BuiltinEntry { module: "math", field: "unit",        global: "unit",        aliases: &["math.unit"] },
    BuiltinEntry { module: "math", field: "is_unitless", global: "is-unitless", aliases: &["math.is-unitless"] },
    BuiltinEntry { module: "math", field: "percentage", global: "percentage", aliases: &["math.percentage"] },
    BuiltinEntry { module: "math", field: "div",         global: "",            aliases: &["math.div"] },
    BuiltinEntry { module: "math", field: "clamp",      global: "clamp",      aliases: &["math.clamp"] },
    BuiltinEntry { module: "math", field: "hypot",       global: "",            aliases: &["math.hypot"] },
    BuiltinEntry { module: "math", field: "log",         global: "",            aliases: &["math.log"] },
    BuiltinEntry { module: "math", field: "pow",         global: "",            aliases: &["math.pow"] },
    BuiltinEntry { module: "math", field: "sin",         global: "",            aliases: &["math.sin"] },
    BuiltinEntry { module: "math", field: "cos",         global: "",            aliases: &["math.cos"] },
    BuiltinEntry { module: "math", field: "tan",         global: "",            aliases: &["math.tan"] },
    BuiltinEntry { module: "math", field: "asin",        global: "",            aliases: &["math.asin"] },
    BuiltinEntry { module: "math", field: "acos",        global: "",            aliases: &["math.acos"] },
    BuiltinEntry { module: "math", field: "atan",        global: "",            aliases: &["math.atan"] },
    BuiltinEntry { module: "math", field: "atan2",       global: "",            aliases: &["math.atan2"] },
    BuiltinEntry { module: "math", field: "sqrt",        global: "",            aliases: &["math.sqrt"] },

    // string
    BuiltinEntry { module: "string", field: "length",     global: "str-length",     aliases: &["string.length"] },
    BuiltinEntry { module: "string", field: "quote",      global: "quote",          aliases: &["string.quote"] },
    BuiltinEntry { module: "string", field: "unquote",    global: "unquote",        aliases: &["string.unquote"] },
    BuiltinEntry { module: "string", field: "to_upper_case", global: "to-upper-case", aliases: &["string.to-upper-case"] },
    BuiltinEntry { module: "string", field: "to_lower_case", global: "to-lower-case", aliases: &["string.to-lower-case"] },
    BuiltinEntry { module: "string", field: "index",      global: "str-index",      aliases: &["string.index"] },
    BuiltinEntry { module: "string", field: "insert",     global: "str-insert",     aliases: &["string.insert"] },
    BuiltinEntry { module: "string", field: "slice",      global: "str-slice",      aliases: &["string.slice"] },
    BuiltinEntry { module: "string", field: "split",      global: "",                aliases: &["string.split"] },

    // map
    BuiltinEntry { module: "map", field: "get",         global: "map-get",        aliases: &["map.get"] },
    BuiltinEntry { module: "map", field: "merge",       global: "map-merge",      aliases: &["map.merge"] },
    BuiltinEntry { module: "map", field: "remove",      global: "map-remove",     aliases: &["map.remove"] },
    BuiltinEntry { module: "map", field: "keys",        global: "map-keys",       aliases: &["map.keys"] },
    BuiltinEntry { module: "map", field: "values",      global: "map-values",     aliases: &["map.values"] },
    BuiltinEntry { module: "map", field: "has_key",     global: "map-has-key",    aliases: &["map.has-key"] },
    BuiltinEntry { module: "map", field: "deep_merge",  global: "",               aliases: &["map.deep-merge"] },
    BuiltinEntry { module: "map", field: "deep_remove", global: "",              aliases: &["map.deep-remove"] },

    // list
    BuiltinEntry { module: "list", field: "length",     global: "length",         aliases: &["list.length"] },
    BuiltinEntry { module: "list", field: "nth",        global: "nth",            aliases: &["list.nth"] },
    BuiltinEntry { module: "list", field: "set_nth",    global: "set-nth",        aliases: &["list.set-nth"] },
    BuiltinEntry { module: "list", field: "join",       global: "join",           aliases: &["list.join"] },
    BuiltinEntry { module: "list", field: "append",     global: "append",         aliases: &["list.append"] },
    BuiltinEntry { module: "list", field: "zip",        global: "zip",            aliases: &["list.zip"] },
    BuiltinEntry { module: "list", field: "index",      global: "index",          aliases: &["list.index"] },
    BuiltinEntry { module: "list", field: "is_bracketed", global: "is-bracketed", aliases: &["list.is-bracketed"] },
    BuiltinEntry { module: "list", field: "separator",  global: "list-separator", aliases: &["list.separator"] },
    BuiltinEntry { module: "list", field: "slash",       global: "",               aliases: &["list.slash"] },

    // color
    BuiltinEntry { module: "color", field: "adjust",     global: "adjust-color",    aliases: &["color.adjust"] },
    BuiltinEntry { module: "color", field: "change",     global: "change-color",    aliases: &["color.change"] },
    BuiltinEntry { module: "color", field: "scale",      global: "scale-color",     aliases: &["color.scale"] },
    BuiltinEntry { module: "color", field: "ie_hex_str", global: "ie-hex-str",     aliases: &["color.ie-hex-str"] },
    BuiltinEntry { module: "color", field: "channel",    global: "",                aliases: &["color.channel"] },
    BuiltinEntry { module: "color", field: "mix",        global: "mix",            aliases: &["color.mix"] },

    // meta
    BuiltinEntry { module: "meta", field: "call",            global: "call",             aliases: &["meta.call"] },
    BuiltinEntry { module: "meta", field: "content_exists",  global: "content-exists",   aliases: &["meta.content-exists"] },
    BuiltinEntry { module: "meta", field: "feature_exists",  global: "feature-exists",   aliases: &["meta.feature-exists"] },
    BuiltinEntry { module: "meta", field: "function_exists", global: "function-exists",   aliases: &["meta.function-exists"] },
    BuiltinEntry { module: "meta", field: "get_function",    global: "get-function",      aliases: &["meta.get-function"] },
    BuiltinEntry { module: "meta", field: "get_mixin",       global: "get-mixin",        aliases: &["meta.get-mixin"] },
    BuiltinEntry { module: "meta", field: "global_variable_exists", global: "global-variable-exists", aliases: &["meta.global-variable-exists"] },
    BuiltinEntry { module: "meta", field: "inspect",         global: "inspect",          aliases: &["meta.inspect"] },
    BuiltinEntry { module: "meta", field: "keywords",        global: "keywords",         aliases: &["meta.keywords"] },
    BuiltinEntry { module: "meta", field: "mixin_exists",    global: "mixin-exists",     aliases: &["meta.mixin-exists"] },
    BuiltinEntry { module: "meta", field: "module_functions", global: "module-functions", aliases: &["meta.module-functions"] },
    BuiltinEntry { module: "meta", field: "module_variables", global: "module-variables", aliases: &["meta.module-variables"] },
    BuiltinEntry { module: "meta", field: "type_of",         global: "type-of",          aliases: &["meta.type-of"] },
    BuiltinEntry { module: "meta", field: "variable_exists",  global: "variable-exists", aliases: &["meta.variable-exists"] },
    BuiltinEntry { module: "meta", field: "load_css",         global: "",                 aliases: &["meta.load-css"] },

    // selector
    BuiltinEntry { module: "selector", field: "is_super_selector", global: "is-superselector", aliases: &["selector.is-superselector"] },
    BuiltinEntry { module: "selector", field: "append",       global: "selector-append",     aliases: &["selector.append"] },
    BuiltinEntry { module: "selector", field: "extend",      global: "selector-extend",     aliases: &["selector.extend"] },
    BuiltinEntry { module: "selector", field: "nest",        global: "selector-nest",        aliases: &["selector.nest"] },
    BuiltinEntry { module: "selector", field: "parse",       global: "selector-parse",      aliases: &["selector.parse"] },
    BuiltinEntry { module: "selector", field: "replace",     global: "selector-replace",    aliases: &["selector.replace"] },
    BuiltinEntry { module: "selector", field: "unify",      global: "selector-unify",      aliases: &["selector.unify"] },
    BuiltinEntry { module: "selector", field: "simple",      global: "simple-selectors",    aliases: &["selector.simple"] },
];

/// 查找全局名对应的 module.field 限定名。
pub fn module_builtin_name(name: &str) -> Option<&'static str> {
    for entry in BUILTIN_TABLE {
        for alias in entry.aliases {
            if *alias == name {
                return Some(alias);
            }
        }
    }
    None
}

/// 是否为已知内建函数名。
pub fn is_known_builtin(name: &str) -> bool {
    for entry in BUILTIN_TABLE {
        if entry.global == name {
            return true;
        }
        for alias in entry.aliases {
            if *alias == name {
                return true;
            }
        }
    }
    false
}

/// dispatch 内建函数。
pub fn dispatch_builtin(name: &str, _args: &[Arg], _env: &Env) -> Option<Result<Value>> {
    // 查找匹配的入口
    let entry = BUILTIN_TABLE.iter().find(|e| {
        e.global == name || e.aliases.iter().any(|a| *a == name)
    })?;

    // 按模块 dispatch
    let module = entry.module;
    let field = entry.field;

    match module {
        "math" => Some(math::dispatch(field, _args, _env)),
        "string" => Some(string::dispatch(field, _args, _env)),
        "map" => Some(map::dispatch(field, _args, _env)),
        "list" => Some(list::dispatch(field, _args, _env)),
        "color" => Some(color::dispatch(field, _args, _env)),
        "meta" => Some(meta::dispatch(field, _args, _env)),
        "selector" => Some(selector::dispatch(field, _args, _env)),
        _ => None,
    }
}
