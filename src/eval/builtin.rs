//! 内建函数分派入口。
//!
//! `call_builtin` 按 match 分派到各函数组。
//! 各函数组已拆分到子模块：color/list/map/string/selector。

pub mod color;
pub mod list;
pub mod map;
pub mod selector;
pub mod string;

use super::*;
use crate::error::{Result, SassError};

impl Evaluator {
    pub(crate) fn call_builtin(name: &str, args: &[Value], env: &Env) -> Result<Value> {
        // CSS 函数名大小写不敏感（如 RGBA == rgba）
        let name = name.to_lowercase();
        let span = tracing::info_span!("call_builtin", name = %name, n_args = args.len());
        let _enter = span.enter();
        match name.as_str() {
            // ── math ──
            "abs" => match args {
                [Value::Number(n, u)] => Ok(Value::Number(n.abs(), u.clone())),
                _ => Err(SassError::Eval("abs 需要 1 个数字参数".into())),
            },
            "ceil" => match args {
                [Value::Number(n, u)] => Ok(Value::Number(n.ceil(), u.clone())),
                _ => Err(SassError::Eval("ceil 需要 1 个数字参数".into())),
            },
            "floor" => match args {
                [Value::Number(n, u)] => Ok(Value::Number(n.floor(), u.clone())),
                _ => Err(SassError::Eval("floor 需要 1 个数字参数".into())),
            },
            "round" => match args {
                [Value::Number(n, u)] => Ok(Value::Number(n.round(), u.clone())),
                _ => Err(SassError::Eval("round 需要 1 个数字参数".into())),
            },
            "min" => args
                .iter()
                .try_fold(Value::Number(f64::INFINITY, None), |acc, v| {
                    match (acc, v) {
                        (Value::Number(a, _), Value::Number(b, u)) => {
                            Ok(Value::Number(a.min(*b), u.clone()))
                        }
                        _ => Err(SassError::Eval("min 需要数字参数".into())),
                    }
                }),
            "max" => args
                .iter()
                .try_fold(Value::Number(f64::NEG_INFINITY, None), |acc, v| {
                    match (acc, v) {
                        (Value::Number(a, _), Value::Number(b, u)) => {
                            Ok(Value::Number(a.max(*b), u.clone()))
                        }
                        _ => Err(SassError::Eval("max 需要数字参数".into())),
                    }
                }),
            "percentage" => match args {
                [Value::Number(n, _)] => Ok(Value::Number(n * 100.0, Some("%".into()))),
                _ => Err(SassError::Eval("percentage 需要 1 个数字参数".into())),
            },
            "math.div" | "div" => match args {
                [Value::Number(a, u1), Value::Number(b, _)] => {
                    if *b == 0.0 {
                        if *a == 0.0 {
                            return Ok(Value::Number(f64::NAN, u1.clone()));
                        }
                        return Ok(Value::Number(a / b, u1.clone()));
                    }
                    Ok(Value::Number(a / b, u1.clone()))
                }
                _ => Err(SassError::Eval("div 需要 2 个数字参数".into())),
            },
            "pow" => match args {
                [Value::Number(a, _), Value::Number(b, _)] => Ok(Value::Number(a.powf(*b), None)),
                _ => Err(SassError::Eval("pow 需要 2 个数字参数".into())),
            },
            "sqrt" => match args {
                [Value::Number(n, _)] => Ok(Value::Number(n.sqrt(), None)),
                _ => Err(SassError::Eval("sqrt 需要 1 个数字参数".into())),
            },
            "sin" => match args {
                [Value::Number(n, _)] => Ok(Value::Number(n.sin(), None)),
                _ => Err(SassError::Eval("sin 需要 1 个参数".into())),
            },
            "cos" => match args {
                [Value::Number(n, _)] => Ok(Value::Number(n.cos(), None)),
                _ => Err(SassError::Eval("cos 需要 1 个参数".into())),
            },
            "tan" => match args {
                [Value::Number(n, _)] => Ok(Value::Number(n.tan(), None)),
                _ => Err(SassError::Eval("tan 需要 1 个参数".into())),
            },
            "log" => match args {
                [Value::Number(n, _)] => Ok(Value::Number(n.ln(), None)),
                _ => Err(SassError::Eval("log 需要 1 个数字参数".into())),
            },
            "random" => match args {
                [] => Ok(Value::Number(Self::simple_random(), None)),
                [Value::Number(n, _)] => Ok(Value::Number(
                    (Self::simple_random() * n).floor() + 1.0,
                    None,
                )),
                _ => Err(SassError::Eval("random 需要 0-1 个参数".into())),
            },
            "clamp" => match args {
                [
                    Value::Number(min, _),
                    Value::Number(val, _),
                    Value::Number(max, _),
                ] => Ok(Value::Number(val.max(*min).min(*max), None)),
                _ => Err(SassError::Eval("clamp 需要 3 个数字参数".into())),
            },
            "unit" => match args {
                [Value::Number(_, Some(u))] => Ok(Value::String(u.clone(), false)),
                [Value::Number(_, None)] => Ok(Value::String("".into(), false)),
                _ => Err(SassError::Eval("unit 需要 1 个数字参数".into())),
            },
            "is-unitless" => match args {
                [Value::Number(_, None)] => Ok(Value::Bool(true)),
                [Value::Number(_, Some(_))] => Ok(Value::Bool(false)),
                _ => Err(SassError::Eval("is-unitless 需要 1 个数字参数".into())),
            },
            "unitless" => match args {
                [Value::Number(_, None)] => Ok(Value::Bool(true)),
                [Value::Number(_, Some(_))] => Ok(Value::Bool(false)),
                _ => Err(SassError::Eval("unitless 需要 1 个数字参数".into())),
            },
            "compatible" => match args {
                [Value::Number(_, u1), Value::Number(_, u2)] => Ok(Value::Bool(
                    Self::units_compatible(u1.as_deref(), u2.as_deref()),
                )),
                _ => Err(SassError::Eval("compatible 需要 2 个数字参数".into())),
            },
            "comparable" => match args {
                [Value::Number(_, u1), Value::Number(_, u2)] => Ok(Value::Bool(
                    Self::units_compatible(u1.as_deref(), u2.as_deref()),
                )),
                _ => Err(SassError::Eval("comparable 需要 2 个数字参数".into())),
            },

            // ── color ──
            "rgba" => Self::builtin_rgba(args),
            "rgb" => Self::builtin_rgba(args),
            "darken" => Self::builtin_darken(args),
            "lighten" => Self::builtin_lighten(args),
            "mix" => Self::builtin_mix(args),
            "invert" | "grayscale" | "color-channel" | "adjust-color" | "change-color"
            | "scale-color" | "hwb" | "complement" | "hsl" | "hsla" | "adjust-hue" | "saturate"
            | "desaturate" | "transparentize" | "fade-out" | "opacify" | "fade-in" | "alpha"
            | "opacity" | "red" | "green" | "blue" | "hue" | "saturation" | "lightness" => {
                color::call(&name, args)?.ok_or_else(|| SassError::UndefinedFunction(name.clone()))
            }

            // ── map ──
            "map-get" | "map-keys" | "map-values" | "map-has-key" | "map-merge" | "map-remove"
            | "map-set" | "map-deep-merge" | "map-deep-remove" => {
                Self::call_map_builtin(&name, args, env)?
                    .ok_or_else(|| SassError::UndefinedFunction(name.clone()))
            }

            // ── string ──
            "str-length" | "to-upper-case" | "to-lower-case" | "unquote" | "quote"
            | "str-slice" | "str-index" | "str-insert" | "str-split" | "unique-id" => {
                Self::call_string_builtin(&name, args)?
                    .ok_or_else(|| SassError::UndefinedFunction(name.clone()))
            }

            // ── meta ──
            "type-of" => match args {
                [Value::Number(..)] => Ok(Value::String("number".into(), false)),
                [Value::String(..)] => Ok(Value::String("string".into(), false)),
                [Value::Color(..)] => Ok(Value::String("color".into(), false)),
                [Value::Bool(..)] => Ok(Value::String("bool".into(), false)),
                [Value::List(..)] => Ok(Value::String("list".into(), false)),
                [Value::Map(..)] => Ok(Value::String("map".into(), false)),
                [Value::Null] => Ok(Value::String("null".into(), false)),
                _ => Ok(Value::String("unknown".into(), false)),
            },
            "inspect" => match args {
                [v] => Ok(Value::String(Self::inspect_value(v), false)),
                _ => Err(SassError::Eval("inspect 需要 1 个参数".into())),
            },
            "if" => match args {
                [cond, t, f] => Ok(if Self::is_truthy(cond) {
                    t.clone()
                } else {
                    f.clone()
                }),
                _ => Err(SassError::Eval("if 需要 3 个参数".into())),
            },
            "mixin-exists" => Ok(Value::Bool(false)),
            "function-exists" => match args {
                [Value::String(name, _)] => Ok(Value::Bool(env.get_function(name).is_some())),
                _ => Ok(Value::Bool(false)),
            },
            "global-variable-exists" => match args {
                [Value::String(name, _)] => Ok(Value::Bool(env.has_var(name))),
                _ => Ok(Value::Bool(false)),
            },
            "variable-exists" => match args {
                [Value::String(name, _)] => Ok(Value::Bool(env.has_var(name))),
                _ => Ok(Value::Bool(false)),
            },
            "get-function" => match args {
                [Value::String(fname, _)] => Ok(Value::String(fname.clone(), false)),
                _ => Err(SassError::Eval("get-function 需要 1 个参数".into())),
            },
            "call" => match args {
                [Value::String(fname, _), rest @ ..] => Self::call_function(fname, rest, env),
                _ => Err(SassError::Eval("call 需要至少 1 个参数".into())),
            },
            "keywords" => match args {
                [_] => Ok(Value::Map(vec![])),
                _ => Err(SassError::Eval("keywords 需要 1 个参数".into())),
            },

            // ── CSS 原生函数——原样保留 ──
            "calc" | "env" | "var" => {
                let arg_str = args
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                Ok(Value::Calc(format!("{name}({arg_str})")))
            }

            // ── list 子模块分派 ──
            "length" | "list-length" | "nth" | "append" | "join" | "index" | "list-separator"
            | "separator" | "set-nth" | "is-bracketed" | "list-slash" | "zip" => {
                list::call(&name, args)?.ok_or_else(|| SassError::UndefinedFunction(name.clone()))
            }

            // ── selector 子模块分派 ──
            "selector-append"
            | "selector-nest"
            | "selector-is-super"
            | "selector-parse"
            | "selector-simple-selectors"
            | "selector-unify"
            | "selector-extend" => selector::call(&name, args)?
                .ok_or_else(|| SassError::UndefinedFunction(name.clone())),

            // ── 未匹配 → 已知 CSS 原生函数原样输出 ──
            _ if Self::is_css_function(&name) => {
                let arg_str = args
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                Ok(Value::String(format!("{name}({arg_str})"), false))
            }
            _ => Err(SassError::UndefinedFunction(name.clone())),
        }
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
            // CSS transform 函数
            | "matrix" | "matrix3d" | "translate" | "translatex" | "translatey" | "translatez"
            | "translate3d" | "scale" | "scalex" | "scaley" | "scalez" | "scale3d"
            | "rotate" | "rotatex" | "rotatey" | "rotatez" | "rotate3d"
            | "skew" | "skewx" | "skewy" | "perspective"
            // CSS filter 函数
            | "blur" | "brightness" | "contrast" | "drop-shadow" | "grayscale"
            | "hue-rotate" | "invert" | "opacity" | "saturate" | "sepia"
            // CSS shape 函数
            | "circle" | "ellipse" | "inset" | "polygon" | "rect" | "xywh" | "ray"
            // CSS 网格函数
            | "grid" | "subgrid"
            // CSS 动画函数
            | "spring" | "scroll" | "view"
        )
    }
}
