//! 内建函数分派入口。
//!
//! `call_builtin` 按 match 分派到各函数组。
//! 各函数组已拆分到子模块：color/list/map/string/selector。

pub mod color;
pub mod list;
pub mod map;
pub mod selector;
pub mod string;
pub mod math;

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
        match name.as_str() {
            // ── sass-spec 测试辅助函数 ──
            "sass" => {
                if env.plain_css {
                    return Err(SassError::Eval(
                        "sass() conditions aren't allowed in plain CSS".into(),
                    ));
                }
                if pos_args.is_empty() {
                    return Err(SassError::Eval("sass() 需要至少 1 个参数".into()));
                }
                Ok(pos_args[0].clone())
            }
            // ── math ──（分派到 builtin::math 模块，支持命名参数）
            "abs" | "ceil" | "floor" | "round" | "min" | "max" | "percentage"
            | "math.div" | "div" | "pow" | "sqrt" | "sin" | "cos" | "tan" | "log"
            | "atan2" | "asin" | "acos" | "atan" | "hypot" | "random" | "clamp"
            | "unit" | "is-unitless" | "unitless" | "compatible" | "comparable" => {
                math::call(&name, pos_args, kw_args)?
                    .ok_or_else(|| SassError::UndefinedFunction(name.clone()))
            }

            // ── color ──
            "rgba" => Self::builtin_rgba(pos_args),
            "rgb" => Self::builtin_rgba(pos_args),
            "darken" => Self::builtin_darken(pos_args),
            "lighten" => Self::builtin_lighten(pos_args),
            "mix" => Self::builtin_mix(pos_args),
            "invert" | "grayscale" | "color-channel" | "adjust-color" | "change-color"
| "scale-color" | "hwb" | "complement" | "adjust-hue" | "saturate"
| "desaturate" | "transparentize" | "fade-out" | "opacify" | "fade-in" | "alpha"
| "opacity" | "red" | "green" | "blue" | "hue" | "saturation" | "lightness"
| "whiteness" | "blackness" | "is-powerless" | "is-in-gamut" | "is-legacy"
| "channel" | "to-space" | "to-gamut" => {
color::call(&name, pos_args, kw_args)?
                    .ok_or_else(|| SassError::UndefinedFunction(name.clone()))
            }
            // hsl/hsla 颜色构造函数——分派到 color::call 处理
            "hsl" | "hsla" => {
                color::call(&name, pos_args, kw_args)?
                    .ok_or_else(|| SassError::UndefinedFunction(name.clone()))
            }

            // ── map ──
            "map-get" | "map-keys" | "map-values" | "map-has-key" | "map-merge" | "map-remove"
            | "map-set" | "map-deep-merge" | "map-deep-remove" => {
                // 支持 $map 关键字参数（如 map.get($map: $m, $key: k)）
                let mut combined_args = Vec::new();
                if pos_args.is_empty() {
                    if let Some(m) = kw_args.get("$map") {
                        combined_args.push(m.clone());
                    }
                } else {
                    combined_args.extend_from_slice(pos_args);
                }
                Self::call_map_builtin(&name, &combined_args, env)?
                    .ok_or_else(|| SassError::UndefinedFunction(name.clone()))
            }

            // ── string ──
            "str-length" | "to-upper-case" | "to-lower-case" | "unquote" | "quote"
            | "str-slice" | "str-index" | "str-insert" | "str-split" | "unique-id" => {
                Self::call_string_builtin(&name, pos_args, kw_args)?
                    .ok_or_else(|| SassError::UndefinedFunction(name.clone()))
            }

            // ── meta ──
            "type-of" => match pos_args {
                [Value::Number(..)] => Ok(Value::String("number".into(), false)),
                [Value::String(..)] => Ok(Value::String("string".into(), false)),
                [Value::Color(..)] => Ok(Value::String("color".into(), false)),
                [Value::Bool(..)] => Ok(Value::String("bool".into(), false)),
                [Value::List(..)] => Ok(Value::String("list".into(), false)),
                [Value::Map(..)] => Ok(Value::String("map".into(), false)),
                [Value::Null] => Ok(Value::String("null".into(), false)),
                _ => Ok(Value::String("unknown".into(), false)),
            },
            "inspect" => match pos_args {
                [v] => Ok(Value::String(crate::eval::value::inspect_value(v), false)),
                _ => Err(SassError::Eval("inspect 需要 1 个参数".into())),
            },
            "if" => match pos_args {
                [cond, t, f] => Ok(if Self::is_truthy(cond) {
                    t.clone()
                } else {
                    f.clone()
                }),
                _ => Err(SassError::Eval("if 需要 3 个参数".into())),
            },
            "content-exists" => {
                // 检查当前环境是否有 @content 内容块
                Ok(Value::Bool(env.content.is_some()))
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
            "mixin-exists" => Ok(Value::Bool(false)),
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
                _ => Err(SassError::Eval("get-function 需要 1 个参数".into())),
            },
            "call" => match pos_args {
                [Value::String(fname, _), rest @ ..] => {
                    let empty_kw = HashMap::new();
                    Self::call_function(fname, rest, &empty_kw, env)
                }
                _ => Err(SassError::Eval("call 需要至少 1 个参数".into())),
            },
            "keywords" => match pos_args {
                [_] => Ok(Value::Map(vec![])),
                _ => Err(SassError::Eval("keywords 需要 1 个参数".into())),
            },

            // ── CSS 原生函数——原样保留 ──
            "calc" | "env" | "var" => {
                let arg_str = pos_args
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                Ok(Value::Calc(format!("{name}({arg_str})")))
            }

            // ── list 子模块分派 ──
            "length" | "list-length" | "nth" | "append" | "join" | "index" | "list-separator"
            | "separator" | "set-nth" | "is-bracketed" | "list-slash" | "zip" => {
                list::call(&name, pos_args, kw_args)?
                    .ok_or_else(|| SassError::UndefinedFunction(name.clone()))
            }

            // ── selector 子模块分派 ──
            "selector-append"
            | "selector-nest"
            | "selector-is-super"
            | "selector-parse"
            | "selector-simple-selectors"
            | "selector-unify"
            | "selector-extend"
            | "selector-replace" => selector::call(&name, pos_args)?
                .ok_or_else(|| SassError::UndefinedFunction(name.clone())),

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
        matches!(
            name,
            // ── math ──
            "abs" | "ceil" | "floor" | "round" | "min" | "max" | "percentage"
| "math.div" | "div" | "pow" | "sqrt" | "sin" | "cos" | "tan" | "log"
| "atan2" | "asin" | "acos" | "atan" | "hypot" | "random" | "clamp" | "unit" | "is-unitless" | "unitless"
            | "compatible" | "comparable"
            // ── color ──
            | "rgba" | "rgb" | "darken" | "lighten" | "mix"
            | "invert" | "grayscale" | "color-channel" | "adjust-color" | "change-color"
            | "scale-color" | "hwb" | "complement" | "hsl" | "hsla" | "adjust-hue"
| "saturate" | "desaturate" | "transparentize" | "fade-out" | "opacify"
| "fade-in" | "alpha" | "opacity" | "red" | "green" | "blue"
| "hue" | "saturation" | "lightness" | "whiteness" | "blackness"
| "is-powerless" | "is-in-gamut" | "is-legacy" | "channel" | "to-space" | "to-gamut"
            // ── map ──
            | "map-get" | "map-keys" | "map-values" | "map-has-key" | "map-merge"
            | "map-remove" | "map-set" | "map-deep-merge" | "map-deep-remove"
            // ── string ──
            | "str-length" | "to-upper-case" | "to-lower-case" | "unquote" | "quote"
            | "str-slice" | "str-index" | "str-insert" | "str-split" | "unique-id"
            // ── meta ──
            | "type-of" | "inspect" | "if" | "feature-exists" | "content-exists" | "mixin-exists" | "function-exists"
            | "global-variable-exists" | "variable-exists" | "get-function" | "call"
            | "keywords"
            // ── list ──
            | "length" | "list-length" | "nth" | "append" | "join" | "index"
            | "list-separator" | "separator" | "set-nth" | "is-bracketed"
            | "list-slash" | "zip"
            // ── selector ──
            | "selector-append" | "selector-nest" | "selector-is-super"
            | "selector-parse" | "selector-simple-selectors" | "selector-unify"
            | "selector-extend" | "selector-replace"
            // ── CSS 原生（在 call_builtin 中有专门分支）──
            | "calc" | "env" | "var"
        )
    }

    /// 检查函数名是否为已知 CSS 原生函数（应原样输出，不求值）。
    fn is_css_function(name: &str) -> bool {
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
