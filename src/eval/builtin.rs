//! 内建函数分派入口。
//!
//! `call_builtin` 按 match 分派到各函数组。
//! 各函数组已拆分到子模块：color/list/map/string/selector。
//! 手工分派函数（rgba/rgb/darken/lighten/mix/if/inspect/type-of 等）在 `manual_dispatch` 中。

pub mod color;
pub mod color_adjust;
pub mod color_conv;
pub mod color_conv_ops;
pub mod color_gamut;
pub mod color_inspect;
pub(crate) mod color_parse;
pub mod color_space;
pub mod dispatch;
pub mod list;
pub(crate) mod manual_dispatch;
pub mod map;
pub mod math;
pub mod math_css;
pub mod math_helpers;
pub mod math_trig;
pub mod selector;
pub mod string;

use super::*;
use crate::error::Result;

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
        if let Some(result) =
            super::builtin::dispatch::dispatch_builtin_module(&name, pos_args, kw_args, env)
        {
            return result;
        }

        // ── meta 函数命名参数合并 ──
        // if/inspect/type-of 等手工分派的函数不经过 dispatch，
        // 需要在 match 之前合并 kw_args → pos_args
        let merged_meta: Option<Vec<Value>> = match name.as_str() {
            "if" => Some(merge_meta_args(
                pos_args,
                kw_args,
                &["condition", "if-true", "if-false"],
            )),
            "inspect" | "type-of" => Some(merge_meta_args(pos_args, kw_args, &["value"])),
            _ => None,
        };
        let pos_args: &[Value] = if let Some(ref merged) = merged_meta {
            merged.as_slice()
        } else {
            pos_args
        };
        let kw_args = if merged_meta.is_some() {
            &HashMap::new()
        } else {
            kw_args
        };

        // ── 手工分派：rgba/rgb/darken/lighten/mix/if/inspect/type-of 等 ──
        Self::manual_dispatch(&name, pos_args, kw_args, env)
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
pub(crate) fn parse_calc_name(s: &str) -> String {
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
pub(crate) fn parse_calc_args(s: &str) -> Vec<Value> {
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
                match !trimmed.is_empty() {
                    true => args.push(parse_calc_arg_value(trimmed)),
                    false => {}
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let trimmed = current.trim();
    match !trimmed.is_empty() {
        true => args.push(parse_calc_arg_value(trimmed)),
        false => {}
    }
    args
}

/// 将单个 calc 参数字符串解析为 `Value`.
///
/// `var(--c)` → `Value::String("var(--c)", false)`（未加引号字符串）
/// `1%` → `Value::Number(1.0, Some("%"))`
/// `2px` → `Value::Number(2.0, Some("px"))`
/// `calc(...)` → `Value::Calc("calc(...)")`
pub(crate) fn parse_calc_arg_value(s: &str) -> Value {
    let s = s.trim();
    // 嵌套 calc/min/max/clamp → Value::Calc
    match s.starts_with("calc(")
        || s.starts_with("min(")
        || s.starts_with("max(")
        || s.starts_with("clamp(")
        || s.starts_with("var(")
        || s.starts_with("env(")
    {
        true => return Value::Calc(s.to_string()),
        false => {}
    }
    // 尝试解析为数字+单位
    match parse_number_with_unit(s) {
        Some(val) => return val,
        None => {}
    }
    // 默认为未加引号字符串
    Value::String(s.to_string(), false)
}

/// 解析 `1%`、`2px`、`3` 等数字字符串为 `Value::Number`。
fn parse_number_with_unit(s: &str) -> Option<Value> {
    let s = s.trim();
    let mut split = s.len();
    for (i, ch) in s.char_indices() {
        match !ch.is_ascii_digit() && ch != '.' && ch != '-' && ch != '+' && ch != 'e' && ch != 'E' {
            true => {
                split = i;
                break;
            }
            false => {}
        }
    }
    let num_str = &s[..split];
    let unit = s[split..].trim();
    num_str.parse::<f64>().ok().map(|n| {
        Value::Number(
            n,
            match unit.is_empty() {
                true => None,
                false => Some(unit.to_string()),
            },
        )
    })
}

/// 返回每个 map 函数的固定参数名列表（按位置顺序）。
/// 可变参数（多 key）返回前缀部分，超出部分从 `pos_args` 追加。
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

/// 通用参数合并——按 `param_names` 顺序填充：先取 `pos_args` 对应位置，
/// 不足的从 `kw_args` 按参数名查找（含 `$` 前缀回退）。
/// 超出 `param_names` 的 `pos_args` 追加到末尾。
fn merge_params_impl(
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    param_names: &[&str],
) -> Vec<Value> {
    let mut result: Vec<Value> = param_names
        .iter()
        .enumerate()
        .filter_map(|(i, pname)| {
            pos_args
                .get(i)
                .cloned()
                .or_else(|| kw_args.get(*pname).cloned())
                .or_else(|| kw_args.get(&format!("${pname}")).cloned())
        })
        .collect();
    match pos_args.len() > param_names.len() {
        true => {
            result.extend_from_slice(&pos_args[param_names.len()..]);
        }
        false => {}
    }
    result
}

/// 将位置参数和命名参数合并为统一的位置参数列表。
/// 按 `param_names` 顺序填充：先取 `pos_args` 对应位置，不足的从 `kw_args` 按参数名查找。
/// 可变参数函数（如 map-get 的多 key）支持从 `pos_args` 追加。
pub(crate) fn merge_map_args(
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    name: &str,
) -> Vec<Value> {
    let param_names = map_param_names(name);
    match param_names.is_empty() {
        true => {
            return pos_args.to_vec();
        }
        false => {}
    }
    merge_params_impl(pos_args, kw_args, param_names)
}

/// 合并 meta 函数（if/inspect/type-of）的位置参数和命名参数。
pub(crate) fn merge_meta_args(
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    param_names: &[&str],
) -> Vec<Value> {
    merge_params_impl(pos_args, kw_args, param_names)
}

/// 合并 rgba/rgb 的位置参数和命名参数（$red/$green/$blue/$alpha）。
pub(crate) fn merge_color_args(pos_args: &[Value], kw_args: &HashMap<String, Value>) -> Vec<Value> {
    const PARAMS: &[&str] = &["red", "green", "blue", "alpha"];
    merge_params_impl(pos_args, kw_args, PARAMS)
}

/// 合并 darken/lighten 的位置参数和命名参数（$color/$amount）。
pub(crate) fn merge_two_args(
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    p1: &str,
    p2: &str,
) -> Vec<Value> {
    merge_params_impl(pos_args, kw_args, &[p1, p2])
}


