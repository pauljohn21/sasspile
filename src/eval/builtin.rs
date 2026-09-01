//! 内建函数分派入口。
//!
//! `call_builtin` 按 match 分派到各函数组。
//! 各函数组已拆分到子模块：color/list/map/string/selector。

pub mod color;
pub mod color_adjust;
pub mod color_conv;
pub mod color_conv_ops;
pub mod color_gamut;
pub mod color_parse;
pub mod color_space;
pub mod dispatch;
pub mod list;
pub mod map;
pub mod selector;
pub mod string;
pub mod math;
pub mod math_helpers;

use super::*;
use crate::error::{Result, SassError};

impl Evaluator {
    pub(crate) fn call_builtin(
        name: &str,
        pos_args: &[Value],
        kw_args: &HashMap<String, Value>,
        env: &Env,
    ) -> Result<Value> {
        // CSS 函数名大小写不敏感（如 RGBA == rgba）
        let name = name.to_lowercase();
        let span =
            crate::__tracing::info_span!("call_builtin", name = %name, n_args = pos_args.len());
        let _enter = span.enter();
        // ── 模块分派：math/string/map/list/color/selector 六组 ──
        if let Some(result) = super::builtin::dispatch::dispatch_builtin_module(
            &name,
            pos_args,
            kw_args,
            env,
        ) {
            return result;
        }

        // ── meta 函数命名参数合并 ──
        // if/inspect/type-of 等手工分派的函数不经过 dispatch，
        // 需要在 match 之前合并 kw_args → pos_args
        let merged_meta: Option<Vec<Value>> = match name.as_str() {
            "if" => Some(merge_meta_args(pos_args, kw_args, &["condition", "if-true", "if-false"])),
            "inspect" | "type-of" => Some(merge_meta_args(pos_args, kw_args, &["value"])),
            _ => None,
        };
        let pos_args: &[Value] = if let Some(ref merged) = merged_meta {
            merged.as_slice()
        } else {
            pos_args
        };
        let kw_args = if merged_meta.is_some() { &HashMap::new() } else { kw_args };

        match name.as_str() {
            // ── sass-spec 测试辅助函数 ──
            "sass" => {
                if env.is_plain_css() {
                    return Err(SassError::Eval(
                        "sass() conditions aren't allowed in plain CSS".into(),
                    ));
                }
                if pos_args.is_empty() {
                    return Err(SassError::Eval("sass() requires at least 1 argument".into()));
                }
                Ok(pos_args[0].clone())
            }
            // ── color（手工 arm：调用 Self::builtin_* 方法）──
            "rgba" => Self::builtin_rgba("rgba", pos_args),
            "rgb" => Self::builtin_rgba("rgb", pos_args),
            "darken" => Self::builtin_darken(pos_args),
            "lighten" => Self::builtin_lighten(pos_args),
            "mix" => Self::builtin_mix(pos_args),
            // CSS Color 4 颜色函数——lab/lch/oklab/oklch/color()
            "lab" | "lch" | "oklab" | "oklch" | "color" => {
                color_parse::parse_color_fn(&name, pos_args, kw_args)
            }

            // ── meta（手工 arm，dispatch = "none"）──
            "type-of" => match pos_args {
                [Value::Number(..)] => Ok(Value::String("number".into(), false)),
                [Value::String(..)] => Ok(Value::String("string".into(), false)),
                [Value::Color(..)] => Ok(Value::String("color".into(), false)),
                [Value::Bool(..)] => Ok(Value::String("bool".into(), false)),
                [Value::List(..)] => Ok(Value::String("list".into(), false)),
                [Value::Map(..)] => Ok(Value::String("map".into(), false)),
                [Value::Null] => Ok(Value::String("null".into(), false)),
                [Value::MixinRef(..)] => Ok(Value::String("mixin".into(), false)),
                [Value::Calc(..)] => Ok(Value::String("calc".into(), false)),
                _ => Ok(Value::String("unknown".into(), false)),
            },
            "inspect" => {
                if pos_args.is_empty() {
                    return Err(SassError::Eval("Missing argument $value.".into()));
                }
                if pos_args.len() > 1 {
                    return Err(SassError::Eval(format!(
                        "Only 1 argument allowed, but {} {} passed.",
                        pos_args.len(),
                        if pos_args.len() == 1 { "was" } else { "were" }
                    )));
                }
                Ok(Value::String(crate::eval::value::inspect_value(&pos_args[0]), false))
            }
            "if" => match pos_args {
                [cond, t, f] => Ok(if Self::is_truthy(cond) {
                    t.clone()
                } else {
                    f.clone()
                }),
                _ => Err(SassError::Eval("if requires 3 arguments".into())),
            },
            "content-exists" => {
                // 检查当前环境是否有 @content 内容块
                Ok(Value::Bool(env.get_content().is_some()))
            },
            "feature-exists" => match pos_args {
                [Value::String(name, _)] => {
                    // 支持的特性列表
                    let supported = matches!(
                        name.as_str(),
                        "global-variable-shadowing"
                            | "extend-selector-pseudoclass"
                            | "units-level-3"
                            | "at-error"
                            | "custom-property"
                    );
                    Ok(Value::Bool(supported))
                }
                _ => Ok(Value::Bool(false)),
            },
            "mixin-exists" => match pos_args {
                [Value::String(name, _)] => {
                    let normalized = name.replace('-', "_");
                    let exists = env.get_mixin(name).is_some()
                        || env.get_mixin(&normalized).is_some()
                        || env.get_mixin(&name.replace('_', "-")).is_some();
                    Ok(Value::Bool(exists))
                }
                _ => Ok(Value::Bool(false)),
            },
            "function-exists" => match pos_args {
                [Value::String(name, _)] => Ok(Value::Bool(env.get_function(name).is_some())),
                _ => Ok(Value::Bool(false)),
            },
            "global-variable-exists" => match pos_args {
                [Value::String(name, _)] => Ok(Value::Bool(env.has_var(name))),
                _ => Ok(Value::Bool(false)),
            },
            "variable-exists" => match pos_args {
                [Value::String(name, _)] => Ok(Value::Bool(env.has_var(name))),
                _ => Ok(Value::Bool(false)),
            },
            "get-function" => match pos_args {
                [Value::String(fname, _)] => Ok(Value::String(fname.clone(), false)),
                _ => Err(SassError::Eval("get-function requires 1 argument".into())),
            },
            "get-mixin" => Self::meta_get_mixin(pos_args, kw_args, env),
            "call" => match pos_args {
                [Value::String(fname, _), rest @ ..] => {
                    let empty_kw = HashMap::new();
                    Self::call_function(fname, rest, &empty_kw, env)
                }
                _ => Err(SassError::Eval("call requires at least 1 argument".into())),
            },
            "module-functions" => Self::meta_module_functions(pos_args, kw_args, env),
            "module-mixins" => Self::meta_module_mixins(pos_args, kw_args, env),
            "module-variables" => Self::meta_module_variables(pos_args, kw_args, env),
            "accepts-content" => Self::meta_accepts_content(pos_args, kw_args, env),
            "keywords" => match pos_args {
                [_] => Ok(Value::Map(vec![])),
                _ => Err(SassError::Eval("keywords requires 1 argument".into())),
            },
            "calc-args" => {
                let calc_arg = pos_args.first().or_else(|| kw_args.get("calc")).or_else(|| kw_args.get("$calc"));
                match calc_arg {
                    Some(Value::Calc(s)) => {
                        let args = parse_calc_args(s);
                        Ok(Value::List(args, Separator::Comma, false))
                    }
                    Some(v) => Err(SassError::Eval(format!(
                        "$calc: {} is not a calculation.",
                        v
                    ))),
                    None => Err(SassError::Eval(
                        "Missing argument $calc.".into(),
                    )),
                }
            }
            "calc-name" => {
                let calc_arg = pos_args.first().or_else(|| kw_args.get("calc")).or_else(|| kw_args.get("$calc"));
                match calc_arg {
                    Some(Value::Calc(s)) => {
                        let name = parse_calc_name(s);
                        Ok(Value::String(name, true))
                    }
                    Some(v) => Err(SassError::Eval(format!(
                        "$calc: {} is not a calculation.",
                        v
                    ))),
                    None => Err(SassError::Eval(
                        "Missing argument $calc.".into(),
                    )),
                }
            }

            // ── CSS 原生函数——原样保留 ──
            "calc" | "env" | "var" => {
                let arg_str = pos_args
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                Ok(Value::Calc(format!("{name}({arg_str})")))
            }

            // ── 未匹配 → 已知 CSS 原生函数原样输出 ──
            _ if Self::is_css_function(&name) => {
                let arg_str = pos_args
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                Ok(Value::String(format!("{name}({arg_str})"), false))
            }
            _ => Err(SassError::UndefinedFunction(name.clone())),
        }
    }

    /// 检查函数名是否为已知的 Sass 内置函数。
    /// 用于区分"真正未定义的函数"（应 CSS 透传）和"已知但参数错误的函数"（应报错）。
    pub(crate) fn is_known_builtin(name: &str) -> bool {
        super::builtin::dispatch::is_known_builtin(name)
            || matches!(name, "rgba" | "rgb" | "darken" | "lighten" | "mix")
    }

    /// 检查函数名是否为已知 CSS 原生函数（应原样输出，不求值）。
    pub(crate) fn is_css_function(name: &str) -> bool {
        matches!(
            name,
            // CSS 变换函数
            "rotate" | "rotateX" | "rotateY" | "rotateZ" | "rotate3d"
            | "translate" | "translateX" | "translateY" | "translateZ" | "translate3d"
            | "scale" | "scaleX" | "scaleY" | "scaleZ" | "scale3d"
            | "skew" | "skewX" | "skewY" | "matrix" | "matrix3d" | "perspective"
            // CSS 滤镜函数
            | "blur" | "brightness" | "contrast" | "drop-shadow"
            | "grayscale" | "hue-rotate" | "invert" | "opacity" | "saturate" | "sepia"
            // CSS 渐变函数
            | "linear-gradient" | "radial-gradient" | "conic-gradient"
            | "repeating-linear-gradient" | "repeating-radial-gradient"
            | "repeating-conic-gradient"
            // CSS 其他函数
            | "cubic-bezier" | "steps" | "frames" | "path" | "paint" | "image"
            | "cross-fade" | "element" | "counter" | "counters" | "symbols"
            | "attr" | "fit-content" | "min-content" | "max-content"
            | "repeat" | "minmax" | "clamp" | "calc" | "env" | "var" | "url"
            | "hsl" | "hsla" | "lab" | "lch" | "oklab" | "oklch"
            | "color" | "color-mix" | "color-contrast"
            | "gradient" | "icrgb" | "device-cmyk"
            // CSS transform 函数（小写变体）
            | "translatex" | "translatey" | "translatez"
            | "scalex" | "scaley" | "scalez"
            | "rotatex" | "rotatey" | "rotatez"
            | "skewx" | "skewy"
            // CSS shape 函数
            | "circle" | "ellipse" | "inset" | "polygon" | "rect" | "xywh" | "ray"
            // CSS 网格函数
            | "grid" | "subgrid"
            // CSS 动画函数
            | "spring" | "scroll" | "view"
        )
    }
}

/// 从 `Value::Calc` 字符串中提取函数名。
///
/// `calc(var(--c))` → `"calc"`
/// `clamp(1%, 2px, 3px)` → `"clamp"`
/// `min(var(--c))` → `"min"`
fn parse_calc_name(s: &str) -> String {
    let s = s.trim();
    if let Some(end) = s.find('(') {
        s[..end].trim().to_string()
    } else {
        s.to_string()
    }
}

/// 从 `Value::Calc` 字符串中提取参数列表。
///
/// `calc(var(--c))` → `[var(--c)]`
/// `clamp(1%, 2px, 3px)` → `[1%, 2px, 3px]`
///
/// 顶层逗号分隔参数，括号内的逗号不计入。
fn parse_calc_args(s: &str) -> Vec<Value> {
    let s = s.trim();
    let inner = if let Some(start) = s.find('(') {
        let end = s.rfind(')').unwrap_or(s.len());
        &s[start + 1..end]
    } else {
        s
    };

    let mut args = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for ch in inner.chars() {
        match ch {
            '(' | '[' => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    args.push(parse_calc_arg_value(trimmed));
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        args.push(parse_calc_arg_value(trimmed));
    }
    args
}

/// 将单个 calc 参数字符串解析为 `Value`。
///
/// `var(--c)` → `Value::String("var(--c)", false)`（未加引号字符串）
/// `1%` → `Value::Number(1.0, Some("%"))`
/// `2px` → `Value::Number(2.0, Some("px"))`
/// `calc(...)` → `Value::Calc("calc(...)")`
fn parse_calc_arg_value(s: &str) -> Value {
    let s = s.trim();
    // 嵌套 calc/min/max/clamp → Value::Calc
    if s.starts_with("calc(")
        || s.starts_with("min(")
        || s.starts_with("max(")
        || s.starts_with("clamp(")
        || s.starts_with("var(")
        || s.starts_with("env(")
    {
        return Value::Calc(s.to_string());
    }
    // 尝试解析为数字+单位
    if let Some(val) = parse_number_with_unit(s) {
        return val;
    }
    // 默认为未加引号字符串
    Value::String(s.to_string(), false)
}

/// 解析 `1%`、`2px`、`3` 等数字字符串为 `Value::Number`。
fn parse_number_with_unit(s: &str) -> Option<Value> {
    let s = s.trim();
    let mut split = s.len();
    for (i, ch) in s.char_indices() {
        if !ch.is_ascii_digit() && ch != '.' && ch != '-' && ch != '+' && ch != 'e' && ch != 'E' {
            split = i;
            break;
        }
    }
    let num_str = &s[..split];
    let unit = s[split..].trim();
    num_str.parse::<f64>().ok().map(|n| {
        Value::Number(n, if unit.is_empty() { None } else { Some(unit.to_string()) })
    })
}

/// 返回每个 map 函数的固定参数名列表（按位置顺序）。
/// 可变参数（多 key）返回前缀部分，超出部分从 pos_args 追加。
fn map_param_names(name: &str) -> &'static [&'static str] {
    match name {
        "map-get" => &["map", "key"],
        "map-keys" => &["map"],
        "map-values" => &["map"],
        "map-has-key" => &["map", "key"],
        "map-merge" => &["map1", "map2"],
        "map-remove" => &["map", "key"],
        "map-set" => &["map", "key", "value"],
        "map-deep-merge" => &["map1", "map2"],
        "map-deep-remove" => &["map", "key"],
        _ => &[],
    }
}

/// 将位置参数和命名参数合并为统一的位置参数列表。
/// 按 `param_names` 顺序填充：先取 pos_args 对应位置，不足的从 kw_args 按参数名查找。
/// 可变参数函数（如 map-get 的多 key）支持从 pos_args 追加。
pub(crate) fn merge_map_args(pos_args: &[Value], kw_args: &HashMap<String, Value>, name: &str) -> Vec<Value> {
    let param_names = map_param_names(name);
    if param_names.is_empty() {
        return pos_args.to_vec();
    }
    let mut result = Vec::with_capacity(param_names.len().max(pos_args.len()));
    for (i, pname) in param_names.iter().enumerate() {
        if i < pos_args.len() {
            result.push(pos_args[i].clone());
        } else if let Some(v) = kw_args.get(*pname) {
            result.push(v.clone());
        } else if let Some(v) = kw_args.get(&format!("${pname}")) {
            result.push(v.clone());
        }
    }
    // 追加多余的 pos_args（如 map-get(map, k1, k2, k3) 的多 key）
    if pos_args.len() > param_names.len() {
        result.extend_from_slice(&pos_args[param_names.len()..]);
    }
    result
}

/// 合并 meta 函数（if/inspect/type-of）的位置参数和命名参数。
///
/// 与 `merge_args` 逻辑相同：按 `param_names` 顺序填充，
/// 先取位置参数，不足的从命名参数查找。
pub(crate) fn merge_meta_args(
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    param_names: &[&str],
) -> Vec<Value> {
    let mut result = Vec::with_capacity(param_names.len().max(pos_args.len()));
    for (i, pname) in param_names.iter().enumerate() {
        if i < pos_args.len() {
            result.push(pos_args[i].clone());
        } else if let Some(v) = kw_args.get(*pname) {
            result.push(v.clone());
        } else if let Some(v) = kw_args.get(&format!("${pname}")) {
            result.push(v.clone());
        }
    }
    if pos_args.len() > param_names.len() {
        result.extend_from_slice(&pos_args[param_names.len()..]);
    }
    result
}
