//! 手工分派函数——rgba/rgb/darken/lighten/mix/if/inspect/type-of 等特殊函数。
//!
//! 这些函数不经过 `dispatch_builtin_module`（派生宏分派），
//! 而是需要特殊参数合并或直接 env 访问的函数。

use super::super::Evaluator;
use crate::error::{Result, SassError};
use crate::eval::Env;
use crate::parse::ast::Value;
use imbl::HashMap;

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
                [] => Err(SassError::Eval("Missing argument $value.".into())),
                [Value::Number(..)] => Ok(Value::String("number".into(), false)),
                [Value::String(..)] => Ok(Value::String("string".into(), false)),
                [Value::Color(..)] => Ok(Value::String("color".into(), false)),
                [Value::Bool(..)] => Ok(Value::String("bool".into(), false)),
                [Value::List(..)] => Ok(Value::String("list".into(), false)),
                [Value::ArgList(..)] => Ok(Value::String("arglist".into(), false)),
                [Value::Map(..)] => Ok(Value::String("map".into(), false)),
                [Value::Null] => Ok(Value::String("null".into(), false)),
                [Value::MixinRef(..)] => Ok(Value::String("mixin".into(), false)),
                [Value::FunctionRef(..)] => Ok(Value::String("function".into(), false)),
                [Value::Calc(c)] => {
                    // 仅含特殊常量（infinity/NaN）的 calc → number
                    let inner = c
                        .strip_prefix("calc(")
                        .and_then(|s| s.strip_suffix(")"))
                        .unwrap_or(c.as_str())
                        .trim();
                    match inner {
                        "infinity" | "-infinity" | "NaN" => {
                            Ok(Value::String("number".into(), false))
                        }
                        _ => Ok(Value::String("calculation".into(), false)),
                    }
                }
                [_, _, ..] => Err(SassError::Eval(format!(
                    "Only 1 argument allowed, but {} were passed.",
                    pos_args.len()
                ))),
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
                [Value::String(name, _), Value::String(module, _)] => {
                    // 2-arg form: check specific module's exports.
                    let dash = name.replace('-', "_");
                    let underscore = name.replace('_', "-");
                    match env.get_namespace(module) {
                        Some(m) => {
                            let found = m.all_functions().any(|(k, _)| {
                                k == name || k == &dash || k == &underscore
                            });
                            Ok(Value::Bool(found))
                        }
                        None => Err(SassError::Eval(format!(
                            "There is no module with namespace \"{module}\"."
                        ))),
                    }
                }
                [Value::String(name, _)] => {
                    // 1-arg form: check local scope + all namespaces + builtins.
                    let dash = name.replace('-', "_");
                    let underscore = name.replace('_', "-");
                    let in_local = env.get_function(name).is_some()
                        || env.get_function(&dash).is_some()
                        || env.get_function(&underscore).is_some();
                    let in_namespace = env.get_namespaces().values().any(|ns| {
                        ns.all_functions().any(|(k, _)| {
                            k == name || k == &dash || k == &underscore
                        })
                    });
                    let is_builtin = super::dispatch::is_known_builtin(name);
                    Ok(Value::Bool(in_local || in_namespace || is_builtin))
                }
                _ => Ok(Value::Bool(false)),
            },
            "global-variable-exists" => match pos_args {
                [Value::String(name, _)] => {
                    // Check local scope + all namespaces.
                    let exists = env.has_var(name)
                        || env.get_namespaces().values().any(|ns| {
                            ns.all_vars().any(|(k, _)| k == name)
                        });
                    Ok(Value::Bool(exists))
                }
                _ => Ok(Value::Bool(false)),
            },
            "variable-exists" => match pos_args {
                [Value::String(name, _)] => Ok(Value::Bool(env.has_var(name))),
                _ => Ok(Value::Bool(false)),
            },
            "get-function" => {
                // 提取 $name 和 $module（支持命名参数形式）
                let name_val = pos_args
                    .first()
                    .or_else(|| kw_args.get("name"))
                    .or_else(|| kw_args.get("$name"));
                let module_val = pos_args
                    .get(1)
                    .or_else(|| kw_args.get("module"))
                    .or_else(|| kw_args.get("$module"));

                match (name_val, module_val) {
                    (Some(Value::String(fname, _)), module_opt) => {
                        // 检查位置参数个数
                        if pos_args.len() > 2 {
                            return Err(SassError::Eval(format!(
                                "Only 2 arguments allowed, but {} were passed.",
                                pos_args.len()
                            )));
                        }
                        let module_str: Option<String> = match module_opt {
                            Some(Value::String(s, _)) => Some(s.clone()),
                            Some(Value::Null) | None => None,
                            Some(v) => {
                                return Err(SassError::Eval(format!(
                                    "$module: {v} is not a string."
                                )))
                            }
                        };
                        // dash-insensitive 查找
                        let lookup_name = fname.replace('-', "_");
                        let lookup_variants = [
                            fname.as_str(),
                            &lookup_name,
                            &fname.replace('_', "-"),
                        ];
                        // 先在指定命名空间查找
                        if let Some(ns_name) = &module_str {
                            if let Some(module) = env.get_namespace(ns_name) {
                                for variant in &lookup_variants {
                                    if let Some(func) = module
                                        .all_functions()
                                        .find(|(k, _)| *k == variant)
                                        .map(|(_, f)| f)
                                    {
                                        return Ok(Value::FunctionRef(std::rc::Rc::new(
                                            crate::parse::ast::FunctionRefData {
                                                name: fname.clone(),
                                                module: module_str.clone(),
                                                params: func.params.clone(),
                                                body: func.body.clone(),
                                                captured_ns_keys: func
                                                    .captured_namespaces
                                                    .keys()
                                                    .cloned()
                                                    .collect(),
                                            },
                                        )));
                                    }
                                }
                            }
                            // 未知命名空间 → 错误
                            return Err(SassError::Eval(format!(
                                "There is no module with namespace \"{ns_name}\"."
                            )));
                        }
                        // 全局查找（local_functions / namespaces）
                        for variant in &lookup_variants {
                            if let Some(func) = env.get_function(variant) {
                                return Ok(Value::FunctionRef(std::rc::Rc::new(
                                    crate::parse::ast::FunctionRefData {
                                        name: fname.clone(),
                                        module: None,
                                        params: func.params.clone(),
                                        body: func.body.clone(),
                                        captured_ns_keys: func
                                            .captured_namespaces
                                            .keys()
                                            .cloned()
                                            .collect(),
                                    },
                                )));
                            }
                        }
                        // 内建函数检查（module 未指定时方可全局查找）
                        if module_str.is_none() {
                            let lookup_norm = fname.replace('-', "_");
                            if super::dispatch::is_known_builtin(&lookup_norm) {
                                return Ok(Value::FunctionRef(std::rc::Rc::new(
                                    crate::parse::ast::FunctionRefData {
                                        name: fname.clone(),
                                        module: None,
                                        params: vec![],
                                        body: vec![], // 空 body 标记内建函数
                                        captured_ns_keys: vec![],
                                    },
                                )));
                            }
                        }
                        // 未知函数
                        Err(SassError::Eval(format!(
                            "Undefined function: {fname}."
                        )))
                    }
                    (Some(v), _) => Err(SassError::Eval(format!(
                        "$name: {v} is not a string."
                    ))),
                    (None, _) => Err(SassError::Eval(
                        "Missing argument $name.".into(),
                    )),
                }
            }
            "get-mixin" => Self::meta_get_mixin(pos_args, kw_args, env),
            "call" => match pos_args {
                [Value::String(fname, _), rest @ ..] => {
                    Self::call_function(fname, rest, kw_args, env)
                }
                [Value::FunctionRef(fn_data), rest @ ..] => {
                    // 空 body 标记内建函数，转分派到 call_builtin
                    match fn_data.body.is_empty() {
                        true => Self::call_builtin(&fn_data.name, rest, kw_args, env),
                        false => Self::call_user_function_ref(fn_data, rest, kw_args, env),
                    }
                }
                _ => Err(SassError::Eval("call requires at least 1 argument".into())),
            },
            "module-functions" => Self::meta_module_functions(pos_args, kw_args, env),
            "module-mixins" => Self::meta_module_mixins(pos_args, kw_args, env),
            "module-variables" => Self::meta_module_variables(pos_args, kw_args, env),
            "accepts-content" => Self::meta_accepts_content(pos_args, kw_args, env),
            "keywords" => match pos_args {
                [Value::ArgList(elements, _, _)] => {
                    // 从 ArgList 末尾提取关键字参数 map
                    match elements.last() {
                        Some(Value::Map(pairs)) => Ok(Value::Map(pairs.clone())),
                        _ => Ok(Value::Map(vec![])),
                    }
                }
                [_] => Ok(Value::Map(vec![])),
                _ => Err(SassError::Eval("keywords requires 1 argument".into())),
            },
            "calc-args" => {
                // 命名参数已在 call_builtin 合并到 pos_args，直接检查个数
                match pos_args.len() {
                    0 => return Err(SassError::Eval("Missing argument $calc.".into())),
                    1 => {}
                    n => {
                        return Err(SassError::Eval(format!(
                            "Only 1 argument allowed, but {n} were passed."
                        )))
                    }
                }
                match &pos_args[0] {
                    Value::Calc(s) => {
                        let args = super::parse_calc_args(s);
                        Ok(Value::List(
                            args,
                            crate::parse::ast::Separator::Comma,
                            false,
                        ))
                    }
                    // calc(1px) 经 eval 简化为 Number，包装回单元素列表
                    Value::Number(..) => Ok(Value::List(
                        vec![pos_args[0].clone()],
                        crate::parse::ast::Separator::Comma,
                        false,
                    )),
                    v => Err(SassError::Eval(format!("$calc: {v} is not a calculation."))),
                }
            }
            "calc-name" => {
                match pos_args.len() {
                    0 => return Err(SassError::Eval("Missing argument $calc.".into())),
                    1 => {}
                    n => {
                        return Err(SassError::Eval(format!(
                            "Only 1 argument allowed, but {n} were passed."
                        )))
                    }
                }
                match &pos_args[0] {
                    Value::Calc(s) => {
                        let name = super::parse_calc_name(s);
                        Ok(Value::String(name, true))
                    }
                    // calc(N) 简化为 Number 后也要识别为 calc 函数名
                    Value::Number(..) => Ok(Value::String("calc".into(), true)),
                    v => Err(SassError::Eval(format!("$calc: {v} is not a calculation."))),
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
