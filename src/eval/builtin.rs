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
            // sass(expr) 将参数作为 Sass 表达式求值（测试专用，实际就是 identity 函数）
            // 在 plain CSS 模式下不允许使用
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
            // ── math ──
            "abs" => match pos_args {
                [Value::Number(n, u)] => Ok(Value::Number(n.abs(), u.clone())),
                _ => Err(SassError::Eval("abs 需要 1 个数字参数".into())),
            },
            "ceil" => match pos_args {
                [Value::Number(n, u)] => Ok(Value::Number(n.ceil(), u.clone())),
                _ => Err(SassError::Eval("ceil 需要 1 个数字参数".into())),
            },
            "floor" => match pos_args {
                [Value::Number(n, u)] => Ok(Value::Number(n.floor(), u.clone())),
                _ => Err(SassError::Eval("floor 需要 1 个数字参数".into())),
            },
            "round" => match pos_args {
                [Value::Number(n, u)] => Ok(Value::Number(n.round(), u.clone())),
                _ => Err(SassError::Eval("round 需要 1 个数字参数".into())),
            },
            "min" => pos_args
                .iter()
                .try_fold(Value::Number(f64::INFINITY, None), |acc, v| {
                    match (acc, v) {
                        (Value::Number(a, _), Value::Number(b, u)) => {
                            Ok(Value::Number(a.min(*b), u.clone()))
                        }
                        _ => Err(SassError::Eval("min 需要数字参数".into())),
                    }
                }),
            "max" => pos_args
                .iter()
                .try_fold(Value::Number(f64::NEG_INFINITY, None), |acc, v| {
                    match (acc, v) {
                        (Value::Number(a, _), Value::Number(b, u)) => {
                            Ok(Value::Number(a.max(*b), u.clone()))
                        }
                        _ => Err(SassError::Eval("max 需要数字参数".into())),
                    }
                }),
            "percentage" => match pos_args {
                [Value::Number(n, _)] => Ok(Value::Number(n * 100.0, Some("%".into()))),
                _ => Err(SassError::Eval("percentage 需要 1 个数字参数".into())),
            },
            "math.div" | "div" => match pos_args {
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
            "pow" => {
                let (a, b) = if pos_args.len() == 2 {
                    match (&pos_args[0], &pos_args[1]) {
                        (Value::Number(a, _), Value::Number(b, _)) => (*a, *b),
                        _ => return Err(SassError::Eval("pow 需要 2 个数字参数".into())),
                    }
                } else if pos_args.len() == 1 {
                    let a = match &pos_args[0] {
                        Value::Number(a, _) => *a,
                        _ => return Err(SassError::Eval("pow 需要 2 个数字参数".into())),
                    };
                    let b = match kw_args.get("exponent") {
                        Some(Value::Number(b, _)) => *b,
                        _ => return Err(SassError::Eval("pow 需要 exponent 参数".into())),
                    };
                    (a, b)
                } else {
                    let a = match kw_args.get("base") {
                        Some(Value::Number(a, _)) => *a,
                        _ => return Err(SassError::Eval("pow 需要 base 参数".into())),
                    };
                    let b = match kw_args.get("exponent") {
                        Some(Value::Number(b, _)) => *b,
                        _ => return Err(SassError::Eval("pow 需要 exponent 参数".into())),
                    };
                    (a, b)
                };
                Ok(Value::Number(a.powf(b), None))
            }
            "sqrt" => match pos_args {
                [Value::Number(n, _)] => Ok(Value::Number(n.sqrt(), None)),
                _ => Err(SassError::Eval("sqrt 需要 1 个数字参数".into())),
            },
            "sin" => match pos_args {
                [Value::Number(n, _)] => Ok(Value::Number(n.sin(), None)),
                _ => Err(SassError::Eval("sin 需要 1 个参数".into())),
            },
            "cos" => match pos_args {
                [Value::Number(n, _)] => Ok(Value::Number(n.cos(), None)),
                _ => Err(SassError::Eval("cos 需要 1 个参数".into())),
            },
            "tan" => match pos_args {
                [Value::Number(n, _)] => Ok(Value::Number(n.tan(), None)),
                _ => Err(SassError::Eval("tan 需要 1 个参数".into())),
            },
            "atan2" => match pos_args {
                [Value::Number(y, yu), Value::Number(x, xu)] => {
                    // 检查单位兼容性
                    if let (Some(yu), Some(xu)) = (yu, xu) {
                        if yu != xu {
                            // 尝试兼容转换（如 cm 和 mm）
                            // 简化处理：如果单位不同但都是长度单位，比值消去单位
                        }
                    }
                    let result = y.atan2(*x).to_degrees();
                    Ok(Value::Number(result, Some("deg".to_string())))
                }
                _ => Err(SassError::Eval("atan2 需要 2 个数字参数".into())),
            },
            "asin" => match pos_args {
                [Value::Number(n, _)] => {
                    let result = n.asin().to_degrees();
                    Ok(Value::Number(result, Some("deg".to_string())))
                }
                _ => Err(SassError::Eval("asin 需要 1 个参数".into())),
            },
            "acos" => match pos_args {
                [Value::Number(n, _)] => {
                    let result = n.acos().to_degrees();
                    Ok(Value::Number(result, Some("deg".to_string())))
                }
                _ => Err(SassError::Eval("acos 需要 1 个参数".into())),
            },
            "atan" => match pos_args {
                [Value::Number(n, _)] => {
                    let result = n.atan().to_degrees();
                    Ok(Value::Number(result, Some("deg".to_string())))
                }
                _ => Err(SassError::Eval("atan 需要 1 个参数".into())),
            },
            "hypot" => {
                if pos_args.is_empty() {
                    return Err(SassError::Eval("hypot 需要 1+ 个参数".into()));
                }
                let sum: f64 = pos_args.iter()
                    .map(|a| match a {
                        Value::Number(n, _) => n * n,
                        _ => 0.0,
                    })
                    .sum();
                Ok(Value::Number(sum.sqrt(), None))
            },
            "log" => match pos_args {
                [Value::Number(n, _)] => {
                    if *n < 0.0 {
                        return Ok(Value::String("calc(NaN)".to_string(), false));
                    }
                    if *n == 0.0 {
                        return Ok(Value::String("calc(-infinity)".to_string(), false));
                    }
                    Ok(Value::Number(n.ln(), None))
                }
                [Value::Number(n, _), Value::Number(base, _)] => {
                    if *n < 0.0 {
                        return Ok(Value::String("calc(NaN)".to_string(), false));
                    }
                    if *n == 0.0 {
                        return Ok(Value::String("calc(-infinity)".to_string(), false));
                    }
                    Ok(Value::Number(n.log(*base), None))
                }
                _ => Err(SassError::Eval("log 需要 1-2 个数字参数".into())),
            },
            "random" => match pos_args {
                [] => Ok(Value::Number(Self::simple_random(), None)),
                [Value::Number(n, _)] => Ok(Value::Number(
                    (Self::simple_random() * n).floor() + 1.0,
                    None,
                )),
                _ => Err(SassError::Eval("random 需要 0-1 个参数".into())),
            },
            "clamp" => match pos_args {
                [
                    Value::Number(min, _),
                    Value::Number(val, _),
                    Value::Number(max, _),
                ] => Ok(Value::Number(val.max(*min).min(*max), None)),
                _ => Err(SassError::Eval("clamp 需要 3 个数字参数".into())),
            },
            "unit" => match pos_args {
                [Value::Number(_, Some(u))] => Ok(Value::String(u.clone(), false)),
                [Value::Number(_, None)] => Ok(Value::String("".into(), false)),
                _ => Err(SassError::Eval("unit 需要 1 个数字参数".into())),
            },
            "is-unitless" => match pos_args {
                [Value::Number(_, None)] => Ok(Value::Bool(true)),
                [Value::Number(_, Some(_))] => Ok(Value::Bool(false)),
                _ => Err(SassError::Eval("is-unitless 需要 1 个数字参数".into())),
            },
            "unitless" => match pos_args {
                [Value::Number(_, None)] => Ok(Value::Bool(true)),
                [Value::Number(_, Some(_))] => Ok(Value::Bool(false)),
                _ => Err(SassError::Eval("unitless 需要 1 个数字参数".into())),
            },
            "compatible" => match pos_args {
                [Value::Number(_, u1), Value::Number(_, u2)] => Ok(Value::Bool(
                    Self::units_compatible(u1.as_deref(), u2.as_deref()),
                )),
                _ => Err(SassError::Eval("compatible 需要 2 个数字参数".into())),
            },
            "comparable" => match pos_args {
                [Value::Number(_, u1), Value::Number(_, u2)] => Ok(Value::Bool(
                    Self::units_compatible(u1.as_deref(), u2.as_deref()),
                )),
                _ => Err(SassError::Eval("comparable 需要 2 个数字参数".into())),
            },

            // ── color ──
            "rgba" => Self::builtin_rgba(pos_args),
            "rgb" => Self::builtin_rgba(pos_args),
            "darken" => Self::builtin_darken(pos_args),
            "lighten" => Self::builtin_lighten(pos_args),
            "mix" => Self::builtin_mix(pos_args),
            "invert" | "grayscale" | "color-channel" | "adjust-color" | "change-color"
| "scale-color" | "hwb" | "complement" | "hsl" | "hsla" | "adjust-hue" | "saturate"
| "desaturate" | "transparentize" | "fade-out" | "opacify" | "fade-in" | "alpha"
| "opacity" | "red" | "green" | "blue" | "hue" | "saturation" | "lightness"
| "whiteness" | "blackness" => {
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
                Self::call_string_builtin(&name, pos_args)?
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
                [v] => Ok(Value::String(Self::inspect_value(v), false)),
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
                list::call(&name, pos_args)?
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
