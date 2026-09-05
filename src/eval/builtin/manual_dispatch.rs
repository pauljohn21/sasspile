//! 手工分派函数——rgba/rgb/darken/lighten/mix/if/inspect/type-of 等特殊函数。
//!
//! 这些函数不经过 `dispatch_builtin_module`（派生宏分派），
//! 而是需要特殊参数合并或直接 env 访问的函数。

use super::super::Evaluator;
use crate::error::{Result, SassError};
use crate::eval::Env;
use crate::parse::ast::Value;
use std::collections::HashMap;

impl Evaluator {
    /// 手工分派——处理 rgba/rgb/darken/lighten/mix/if/inspect/type-of 等特殊函数。
    ///
    /// 调用时 `pos_args` 和 `kw_args` 已经过 meta 命名参数合并。
    pub(crate) fn manual_dispatch(
        name: &str,
        pos_args: &[Value],
        kw_args: &HashMap<String, Value>,
        env: &Env,
    ) -> Result<Value> {
        match name {
            // ── sass-spec 测试辅助函数 ──
            "sass" => {
                match env.is_plain_css() {
                    true => return Err(SassError::Eval(
                        "sass() conditions aren't allowed in plain CSS".into(),
                    )),
                    false => {}
                }
                match pos_args.is_empty() {
                    true => return Err(SassError::Eval(
                        "sass() requires at least 1 argument".into(),
                    )),
                    false => {}
                }
                Ok(pos_args[0].clone())
            }
            // ── color（手工 arm：调用 Self::builtin_* 方法）──
            // 合并命名参数 $red/$green/$blue/$alpha → 位置参数
            "rgba" | "rgb" => {
                let merged = super::merge_color_args(pos_args, kw_args);
                Self::builtin_rgba(name, &merged)
            }
            // darken/lighten/mix 合并 $color/$amount 命名参数
            "darken" | "lighten" => {
                let merged = super::merge_two_args(pos_args, kw_args, "color", "amount");
                match name {
                    "darken" => Self::builtin_darken(&merged),
                    _ => Self::builtin_lighten(&merged),
                }
            }
            "mix" => {
                // 提取 $method 参数（第 4 个位置参数或命名参数 $method）
                let method = kw_args.get("method")
                    .or_else(|| kw_args.get("method"))
                    .or_else(|| pos_args.get(3));
                Self::builtin_mix_modern(pos_args, method)
            }
            // CSS Color 4 颜色函数——lab/lch/oklab/oklch/color()
            "lab" | "lch" | "oklab" | "oklch" | "color" => {
                super::color_parse::parse_color_fn(name, pos_args, kw_args)
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
                match pos_args.len() {
                    0 => return Err(SassError::Eval("Missing argument $value.".into())),
                    1 => {}
                    n => return Err(SassError::Eval(format!(
                        "Only 1 argument allowed, but {n} {} passed.",
                        match n == 1 { true => "was", false => "were" }
                    ))),
                }
                Ok(Value::String(
                    crate::eval::value::inspect_value(&pos_args[0]),
                    false,
                ))
            }
            "if" => match pos_args {
                [cond, t, f] => Ok(match Self::is_truthy(cond) {
                    true => t.clone(),
                    false => f.clone(),
                }),
                _ => Err(SassError::Eval("if requires 3 arguments".into())),
            },
            "content-exists" => {
                // 检查当前环境是否有 @content 内容块
                Ok(Value::Bool(env.get_content().is_some()))
            }
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
                let calc_arg = pos_args
                    .first()
                    .or_else(|| kw_args.get("calc"))
                    .or_else(|| kw_args.get("calc"));
                match calc_arg {
                    Some(Value::Calc(s)) => {
                        let args = super::parse_calc_args(s);
                        Ok(Value::List(
                            args,
                            crate::parse::ast::Separator::Comma,
                            false,
                        ))
                    }
                    Some(v) => Err(SassError::Eval(format!("$calc: {v} is not a calculation."))),
                    None => Err(SassError::Eval("Missing argument $calc.".into())),
                }
            }
            "calc-name" => {
                let calc_arg = pos_args
                    .first()
                    .or_else(|| kw_args.get("calc"))
                    .or_else(|| kw_args.get("calc"));
                match calc_arg {
                    Some(Value::Calc(s)) => {
                        let name = super::parse_calc_name(s);
                        Ok(Value::String(name, true))
                    }
                    Some(v) => Err(SassError::Eval(format!("$calc: {v} is not a calculation."))),
                    None => Err(SassError::Eval("Missing argument $calc.".into())),
                }
            }

            // ── CSS 原生函数——原样保留 ──
            "calc" | "env" | "var" => {
                let arg_str = pos_args
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                Ok(Value::Calc(format!("{name}({arg_str})")))
            }

            // ── 未匹配 → 已知 CSS 原生函数原样输出 ──
            _ if Self::is_css_function(name) => {
                let arg_str = pos_args
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                Ok(Value::String(format!("{name}({arg_str})"), false))
            }
            _ => Err(SassError::UndefinedFunction(name.to_string())),
        }
    }
}
