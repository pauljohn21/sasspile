use super::*;
use crate::__tracing::{debug, warn};
use crate::css::node::CssNode;
use crate::error::{Result, SassError};
use crate::parse::ast::BinOpKind;

mod calc;
mod display;
mod ops;
mod partial;

pub(crate) use display::{
    eval_interp_segments, eval_interp_str, eval_property_name, eval_simple_expr, inspect_value,
};
pub(crate) use ops::{add, compare, div, modulo, mul, sub, units_compatible, values_eq};
pub(crate) use partial::PartialCond;

impl Evaluator {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(env), fields(name = name, is_default = flags.default), level = "debug"))]
    pub(crate) fn eval_variable(
        name: &str,
        value: &Value,
        flags: &VarFlags,
        env: Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        // 命名空间变量赋值（namespace.$var）——更新模块变量
        if name.contains('.') {
            let val = Self::eval_value(value, &env)?;
            let parts: Vec<&str> = name.splitn(2, '.').collect();
            if parts.len() == 2 {
                let ns = parts[0];
                let var_name = parts[1];
                if env.get_namespace(ns).is_some() {
                    let env = env.with_namespace_var(ns, var_name, val);
                    return Ok((vec![], env));
                }
            }
            return Ok((vec![], env));
        }
        if flags.default {
            // !default 赋值：先检查 pending_config（with 配置覆盖值）
            let normalized = name.replace('-', "_");
            let config_val = env
                .get_pending_config()
                .get(&normalized)
                .or_else(|| env.get_pending_config().get(name))
                .cloned();
            if let Some(val) = config_val {
                let consumed_key = env
                    .get_pending_config()
                    .get(&normalized)
                    .map(|_| normalized.clone())
                    .or_else(|| env.get_pending_config().get(name).map(|_| name.to_string()));
                debug!(name = %name, consumed_key = ?consumed_key, "eval_variable: !default consumed from pending_config");
                let new_env = env.bind(name.to_string(), val);
                let new_env = if let Some(key) = consumed_key {
                    new_env.add_consumed_config(key)
                } else {
                    new_env
                };
                return Ok((vec![], new_env));
            }
            if env.has_var(name) {
                return Ok((vec![], env));
            }
        }
        let val = Self::eval_value(value, &env)?;
        let new_env = env.bind(name.to_string(), val.clone());
        if flags.global {
            Ok((vec![], new_env.add_global_write(name.to_string(), val)))
        } else {
            Ok((vec![], new_env))
        }
    }

    /// 求值值表达式。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(value, env), fields(depth = env.get_depth()), level = "trace"))]
    pub(crate) fn eval_value(value: &Value, env: &Env) -> Result<Value> {
        match value {
            Value::Number(..)
            | Value::Color(..)
            | Value::Bool(..)
            | Value::Null
            | Value::MixinRef(..) => Ok(value.clone()),
            Value::Calc(s) => Ok(Self::simplify_calc(s)),
            Value::Paren(inner) => Self::eval_value(inner, env),
            Value::String(s, quoted) => {
                if s.contains('#') && s.contains('{') {
                    Ok(Value::String(eval_interp_str(s, env), *quoted))
                } else if !*quoted {
                    if let Some(color) = Self::lookup_named_color(s) {
                        Ok(Value::Color(color))
                    } else {
                        Ok(value.clone())
                    }
                } else {
                    Ok(value.clone())
                }
            }
            Value::Variable(name) => {
                if let Some(dot) = name.find('.') {
                    let ns = &name[..dot];
                    let var_name = &name[dot + 1..];
                    if let Some(module) = env.get_namespace(ns) {
                        if let Some(val) = module
                            .all_vars()
                            .find(|(k, _)| *k == var_name)
                            .map(|(_, v)| v)
                        {
                            return Ok(val.clone());
                        }
                    } else {
                        return Err(SassError::Eval(format!(
                            "There is no module with the namespace \"{ns}\"."
                        )));
                    }
                }
                env.lookup(name)
                    .cloned()
                    .ok_or_else(|| SassError::UndefinedVariable(name.clone()))
            }
            Value::List(elements, sep, bracketed) => {
                let evaluated: Vec<Value> = elements
                    .iter()
                    .map(|e| Self::eval_value(e, env))
                    .collect::<Result<_>>()?;
                Ok(Value::List(evaluated, sep.clone(), *bracketed))
            }
            Value::Map(pairs) => {
                let evaluated: Vec<(Value, Value)> = pairs
                    .iter()
                    .map(|(k, v)| Ok((Self::eval_value(k, env)?, Self::eval_value(v, env)?)))
                    .collect::<Result<_>>()?;
                Ok(Value::Map(evaluated))
            }
            Value::Call(name, args) => Self::eval_call(name, args, env),
            Value::Interp(segments) => {
                Ok(Value::String(eval_interp_segments(segments, env), false))
            }
            Value::BinOp(b) => Self::eval_binop(&b.op, &b.left, &b.right, env),
            Value::UnaryOp(op, v) => {
                let val = Self::eval_value(v, env)?;
                match op {
                    UnaryOp::Neg => match val {
                        Value::Number(n, u) => Ok(Value::Number(-n, u)),
                        _ => Err(SassError::Eval(format!("Cannot negate {val}"))),
                    },
                    UnaryOp::Not => match val {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        _ => Ok(Value::Bool(false)),
                    },
                }
            }
            Value::Spread(v) => Self::eval_value(v, env),
        }
    }

    /// 求值函数调用。
    fn eval_call(name: &str, args: &[Arg], env: &Env) -> Result<Value> {
        // if() 惰性求值：只求值选中的分支
        if name == "if"
            && args.len() == 3
            && args
                .iter()
                .all(|a| a.name.is_none() && a.condition.is_none())
        {
            let cond = Self::eval_value(&args[0].value, env)?;
            if Self::is_truthy(&cond) {
                return Self::eval_value(&args[1].value, env);
            }
            return Self::eval_value(&args[2].value, env);
        }
        // if() 命名参数语法
        if name == "if" && args.iter().any(|a| a.name.is_some()) {
            let pos_args: Vec<&Arg> = args
                .iter()
                .filter(|a| a.name.is_none() && a.condition.is_none())
                .collect();
            if pos_args.len() == 1 {
                let cond = Self::eval_value(&pos_args[0].value, env)?;
                let if_true = args.iter().find(|a| {
                    a.name.as_deref() == Some("if-true") || a.name.as_deref() == Some("$if-true")
                });
                let if_false = args.iter().find(|a| {
                    a.name.as_deref() == Some("if-false") || a.name.as_deref() == Some("$if-false")
                });
                if Self::is_truthy(&cond) {
                    if let Some(t) = if_true {
                        return Self::eval_value(&t.value, env);
                    }
                } else if let Some(f) = if_false {
                    return Self::eval_value(&f.value, env);
                }
                return Ok(Value::Null);
            }
        }
        // if() 冒号语法：if(cond1: val1; cond2: val2; else: default)
        if name == "if" && args.iter().any(|a| a.condition.is_some()) {
            return Self::eval_if_colon(args, env);
        }
        // if(else: value) — else-only 语法
        if name == "if"
            && !args.is_empty()
            && args.iter().all(|a| a.name.as_deref() == Some("else"))
        {
            return Self::eval_value(&args[0].value, env);
        }
        // 分离位置参数和关键字参数，展开 spread
        let (pos_args, kw_args) = Self::collect_args(args, env)?;
        // 尝试作为 Sass 函数调用，未定义时 CSS 透传
        Self::dispatch_function(name, pos_args, kw_args, env)
    }

    /// `if()` 冒号语法求值。
    fn eval_if_colon(args: &[Arg], env: &Env) -> Result<Value> {
        let else_arg = args.iter().find(|a| a.name.as_deref() == Some("else"));
        for (i, cond_arg) in args
            .iter()
            .enumerate()
            .filter(|(_, a)| a.condition.is_some())
        {
            let condition = cond_arg.condition.as_ref().expect("已检查");
            // 检查条件中是否有 sass()+CSS 混用
            Self::check_sass_css_mix(condition)?;
            match Self::partial_eval_condition(condition, env)? {
                PartialCond::True => return Self::eval_value(&cond_arg.value, env),
                PartialCond::False => continue,
                PartialCond::Css(css_str) => {
                    let mut parts: Vec<String> = Vec::new();
                    parts.push(format!("{css_str}: {}", cond_arg.value));
                    for (_, a) in args
                        .iter()
                        .enumerate()
                        .filter(|(j, a)| *j > i && a.condition.is_some())
                    {
                        let cond = a.condition.as_ref().expect("已检查");
                        parts.push(format!("{cond}: {}", a.value));
                    }
                    if let Some(else_a) = else_arg {
                        parts.push(format!("else: {}", else_a.value));
                    }
                    return Ok(Value::String(format!("if({})", parts.join("; ")), false));
                }
            }
        }
        // 所有条件都为 false，返回 else 或 Null
        if let Some(else_a) = else_arg {
            Self::eval_value(&else_a.value, env)
        } else {
            Ok(Value::Null)
        }
    }

    /// 分离位置参数和关键字参数，展开 spread。
    fn collect_args(args: &[Arg], env: &Env) -> Result<(Vec<Value>, HashMap<String, Value>)> {
        let mut pos_args: Vec<Value> = Vec::new();
        let mut kw_args: HashMap<String, Value> = HashMap::new();
        for arg in args {
            let val = Self::eval_value(&arg.value, env)?;
            if arg.spread {
                match &val {
                    Value::List(items, _, _) => pos_args.extend(items.iter().cloned()),
                    Value::Map(pairs) => {
                        for (k, v) in pairs {
                            if let Value::String(key, _) = k {
                                kw_args.insert(key.clone(), v.clone());
                            }
                        }
                    }
                    other => pos_args.push(other.clone()),
                }
            } else if let Some(n) = &arg.name {
                kw_args.insert(n.clone(), val);
            } else {
                pos_args.push(val);
            }
        }
        Ok((pos_args, kw_args))
    }

    /// 分派函数调用——Sass 函数 → CSS 透传。
    fn dispatch_function(
        name: &str,
        pos_args: Vec<Value>,
        kw_args: HashMap<String, Value>,
        env: &Env,
    ) -> Result<Value> {
        match Self::call_function(name, &pos_args, &kw_args, env) {
            Ok(result) => Ok(result),
            Err(SassError::UndefinedFunction(_))
                if !name.contains('.') && !Self::is_known_builtin(name) =>
            {
                let mut parts: Vec<String> = pos_args
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect();
                for (k, v) in &kw_args {
                    parts.push(format!("{k}={v}"));
                }
                Ok(Value::String(
                    format!("{name}({})", parts.join(", ")),
                    false,
                ))
            }
            Err(SassError::Eval(_))
                if !name.contains('.') && matches!(name, "min" | "max" | "clamp") =>
            {
                let arg_str = pos_args
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                Ok(Value::Calc(format!("{name}({arg_str})")))
            }
            Err(e) => Err(e),
        }
    }

    /// 求值二元运算。
    pub(crate) fn eval_binop(
        op: &BinOpKind,
        left: &Value,
        right: &Value,
        env: &Env,
    ) -> Result<Value> {
        // 短路求值：and / or
        match op {
            BinOpKind::And => {
                let l = Self::eval_value(left, env)?;
                if !Self::is_truthy(&l) {
                    return Ok(l);
                }
                return Self::eval_value(right, env);
            }
            BinOpKind::Or => {
                let l = Self::eval_value(left, env)?;
                if Self::is_truthy(&l) {
                    return Ok(l);
                }
                return Self::eval_value(right, env);
            }
            _ => {}
        }
        let l = Self::eval_value(left, env)?;
        let r = Self::eval_value(right, env)?;
        crate::__tracing::trace!(
            target: "sasspile::binop",
            op = ?op,
            left = %l, right = %r,
            "binop operands evaluated"
        );
        let result = match op {
            BinOpKind::Add => add(&l, &r),
            BinOpKind::Sub => sub(&l, &r),
            BinOpKind::Mul => mul(&l, &r),
            BinOpKind::Div => div(&l, &r),
            BinOpKind::Mod => modulo(&l, &r),
            BinOpKind::Eq => Ok(Value::Bool(values_eq(&l, &r))),
            BinOpKind::NotEq => Ok(Value::Bool(!values_eq(&l, &r))),
            BinOpKind::And => Ok(r),
            BinOpKind::Or => Ok(r),
            BinOpKind::Lt | BinOpKind::Gt | BinOpKind::LtEq | BinOpKind::GtEq => {
                compare(op, &l, &r)
            }
        };
        if let Ok(v) = &result {
            crate::__tracing::trace!(
                target: "sasspile::binop",
                op = ?op,
                result = %v,
                "binop result"
            );
        }
        result
    }
}
