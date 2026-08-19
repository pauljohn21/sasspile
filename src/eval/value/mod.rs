use super::*;
use crate::css::node::CssNode;
use crate::error::{Result, SassError};
use crate::parse::ast::BinOpKind;
use crate::__tracing::warn;

mod display;
mod ops;

pub(crate) use display::{eval_interp_str, eval_property_name, eval_simple_expr, inspect_value};
pub(crate) use ops::{add, compare, div, modulo, mul, sub, units_compatible, values_eq};

/// 部分条件求值结果。
enum PartialCond {
    True,
    False,
    Css(String),
}

impl Evaluator {
pub(crate) fn eval_variable(
    name: &str,
    value: &Value,
    flags: &VarFlags,
    env: &Env,
) -> Result<(Vec<CssNode>, Env)> {
    // 命名空间变量赋值（namespace.$var）——更新模块变量
    if name.contains('.') {
        let val = Self::eval_value(value, env)?;
        // 分割 namespace.var_name
        let parts: Vec<&str> = name.splitn(2, '.').collect();
        if parts.len() == 2 {
            let ns = parts[0];
            let var_name = parts[1];
            // 更新命名空间模块中的变量
            if let Some(exports) = env.namespaces.get(&ns.to_string()) {
                let mut new_exports = (**exports).clone();
                new_exports.vars.insert(var_name.to_string(), val.clone());
                let mut new_env = env.clone();
                new_env.namespaces.insert(ns.to_string(), Rc::new(new_exports));
                return Ok((vec![], new_env));
            }
        }
        // 找不到命名空间——忽略
        return Ok((vec![], env.clone()));
    }
    if flags.default && env.has_var(name) {
        return Ok((vec![], env.clone()));
    }
    let val = Self::eval_value(value, env)?;
    let new_env = env.bind(name.to_string(), val.clone());
    // !global 变量同时写入 global_writes，供 eval_rule 传播到外层
    if flags.global {
        let mut env = new_env;
        env.global_writes.insert(name.to_string(), val);
        Ok((vec![], env))
    } else {
        Ok((vec![], new_env))
    }
}

    /// 部分条件求值结果。
    fn partial_eval_condition(condition: &Value, env: &Env) -> Result<PartialCond> {
        match condition {
            // 括号——递归求值内部表达式
            // Paren 包裹 List 时不保留括号（如 (var(--not) css()) → var(--not) css()）
            // Paren 包裹单个 Calc 时保留括号（如 (css()) → (css())）
            Value::Paren(inner) => {
                match Self::partial_eval_condition(inner, env)? {
                    PartialCond::True => Ok(PartialCond::True),
                    PartialCond::False => Ok(PartialCond::False),
                    PartialCond::Css(s) => {
                        // 如果内部是 List，不加括号；否则加括号
                        if matches!(inner.as_ref(), Value::List(_, Separator::Space, _)) {
                            Ok(PartialCond::Css(s))
                        } else {
                            Ok(PartialCond::Css(format!("({s})")))
                        }
                    }
                }
            }
            Value::UnaryOp(UnaryOp::Not, inner) => {
                match Self::partial_eval_condition(inner, env)? {
                    PartialCond::True => Ok(PartialCond::False),
                    PartialCond::False => Ok(PartialCond::True),
                    PartialCond::Css(s) => Ok(PartialCond::Css(format!("not {s}"))),
                }
            }
            Value::BinOp(b) => match b.op {
                BinOpKind::And => {
                    match Self::partial_eval_condition(&b.left, env)? {
                        PartialCond::False => Ok(PartialCond::False),
                        PartialCond::True => Self::partial_eval_condition(&b.right, env),
                        PartialCond::Css(left_css) => {
                            match Self::partial_eval_condition(&b.right, env)? {
                                PartialCond::False => Ok(PartialCond::False),
                                PartialCond::True => Ok(PartialCond::Css(left_css)),
                                PartialCond::Css(right_css) => Ok(PartialCond::Css(format!("{left_css} and {right_css}"))),
                            }
                        }
                    }
                }
                BinOpKind::Or => {
                    match Self::partial_eval_condition(&b.left, env)? {
                        PartialCond::True => Ok(PartialCond::True),
                        PartialCond::False => Self::partial_eval_condition(&b.right, env),
                        PartialCond::Css(left_css) => {
                            match Self::partial_eval_condition(&b.right, env)? {
                                PartialCond::True => Ok(PartialCond::True),
                                PartialCond::False => Ok(PartialCond::Css(left_css)),
                                PartialCond::Css(right_css) => Ok(PartialCond::Css(format!("{left_css} or {right_css}"))),
                            }
                        }
                    }
                }
                _ => {
                    let val = Self::eval_value(condition, env)?;
                    if Self::is_truthy(&val) {
                        Ok(PartialCond::True)
                    } else {
                        Ok(PartialCond::False)
                    }
                }
            },
            // CSS 原生函数——不可求值
            Value::Calc(_) => Ok(PartialCond::Css(format!("{condition}"))),
            // css() 函数（可能通过插值得到函数名）——CSS 透传
            Value::Call(name, _args) if name == "css" => {
                Ok(PartialCond::Css(format!("{condition}")))
            }
            // 嵌套 if() 调用——如果返回 CSS 值则保留原始形式
            Value::Call(name, _args) if name == "if" => {
                let val = Self::eval_value(condition, env)?;
                if let Value::Calc(_) = val {
                    Ok(PartialCond::Css(format!("{condition}")))
                } else if Self::is_truthy(&val) {
                    Ok(PartialCond::True)
                } else {
                    Ok(PartialCond::False)
                }
            }
            // 空格分隔列表作为条件（如 var(--not) css()）
            Value::List(items, Separator::Space, _) => {
                // 检查是否包含 CSS 部分
                let mut has_css = false;
                let mut css_parts: Vec<String> = Vec::new();
                for item in items {
                    match Self::partial_eval_condition(item, env)? {
                        PartialCond::False => {
                            // false 条件在空格分隔列表中使整个条件为 false
                            return Ok(PartialCond::False);
                        }
                        PartialCond::True => {
                            // true 条件不影响 CSS 部分
                        }
                        PartialCond::Css(s) => {
                            has_css = true;
                            css_parts.push(s);
                        }
                    }
                }
                if has_css {
                    Ok(PartialCond::Css(css_parts.join(" ")))
                } else {
                    Ok(PartialCond::True)
                }
            }
            // sass() 函数——求值参数
            Value::Call(name, _args) if name == "sass" => {
                if env.plain_css {
                    return Err(SassError::Eval("sass() conditions aren't allowed in plain CSS".into()));
                }
                let val = Self::eval_value(condition, env)?;
                if Self::is_truthy(&val) {
                    Ok(PartialCond::True)
                } else {
                    Ok(PartialCond::False)
                }
            }
            // 插值——plain CSS 中不允许
            Value::Interp(s) => {
                if env.plain_css {
                    return Err(SassError::Eval("Interpolation isn't allowed in plain CSS.".into()));
                }
                // 用 eval_simple_expr 求值插值内容（正确处理字符串引号）
                let val = eval_simple_expr(s, env).unwrap_or_else(|_| Value::String(s.clone(), false));
                // 提取字符串内部值（去引号）进行比较
                let val_str = match &val {
                    Value::String(inner, _) => inner.clone(),
                    _ => val.to_string(),
                };
                // and/or/not 关键字通过插值得到——作为 CSS 透传
                if val_str == "and" || val_str == "or" || val_str == "not"
                    || val_str.starts_with("css(")
                    || val_str.starts_with("var(")
                    || val_str.starts_with("attr(")
                    || val_str.starts_with("calc(")
                    || val_str.starts_with("env(")
                    || val_str.starts_with("clamp(")
                {
                    Ok(PartialCond::Css(val_str))
                } else if let Value::Calc(_) = val {
                    Ok(PartialCond::Css(val.to_string()))
                } else if Self::is_truthy(&val) {
                    Ok(PartialCond::True)
                } else {
                    Ok(PartialCond::False)
                }
            }
            // 其他值——正常求值
            _ => {
                let val = Self::eval_value(condition, env)?;
                // 求值结果为 CSS 原生函数——透传
                if let Value::Calc(_) = val {
                    Ok(PartialCond::Css(val.to_string()))
                } else if Self::is_truthy(&val) {
                    Ok(PartialCond::True)
                } else {
                    Ok(PartialCond::False)
                }
            }
        }
    }

    /// 求值值表达式。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(value, env), fields(depth = env.depth), level = "trace"))]
    pub(crate) fn eval_value(value: &Value, env: &Env) -> Result<Value> {
        match value {
            Value::Number(..)
            | Value::Color(..)
            | Value::Bool(..)
            | Value::Null
            | Value::Calc(..) => Ok(value.clone()),
            Value::Paren(inner) => Self::eval_value(inner, env),
            Value::String(s, quoted) => {
                // 处理插值在字符串中
                if s.contains('#') && s.contains('{') {
                    Ok(Value::String(eval_interp_str(s, env), *quoted))
                } else if !*quoted {
                    // 非引号字符串：检查是否为 CSS 命名颜色（white, black, red 等）
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
                // 检查是否为命名空间变量 module.var
                if let Some(dot) = name.find('.') {
                    let ns = &name[..dot];
                    let var_name = &name[dot + 1..];
                    if let Some(module) = env.get_namespace(ns)
                        && let Some(val) = module.vars.get(var_name) {
                            return Ok(val.clone());
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
                // 空格分隔列表可能需要进一步处理
                Ok(Value::List(evaluated, sep.clone(), *bracketed))
            }
            Value::Map(pairs) => {
                let evaluated: Vec<(Value, Value)> = pairs
                    .iter()
                    .map(|(k, v)| Ok((Self::eval_value(k, env)?, Self::eval_value(v, env)?)))
                    .collect::<Result<_>>()?;
                Ok(Value::Map(evaluated))
            }
            Value::Call(name, args) => {
                // if() 惰性求值：只求值选中的分支，避免副作用和类型错误
                if name == "if" && args.len() == 3 && args.iter().all(|a| a.name.is_none() && a.condition.is_none()) {
                    let cond = Self::eval_value(&args[0].value, env)?;
                    if Self::is_truthy(&cond) {
                        return Self::eval_value(&args[1].value, env);
                    } else {
                        return Self::eval_value(&args[2].value, env);
                    }
                }
                // if() 冒号语法：if(cond1: val1; cond2: val2; else: default)
                // 使用 partial_eval_condition 进行部分求值：
                // - sass() 部分短路求值
                // - css() 部分保留为 CSS 透传
                if name == "if" && args.iter().any(|a| a.condition.is_some()) {
                    let else_arg = args.iter().find(|a| a.name.as_deref() == Some("else"));
                    // 逐个部分求值条件
                    for (i, cond_arg) in args.iter().enumerate().filter(|(_, a)| a.condition.is_some()) {
                        let condition = cond_arg.condition.as_ref().expect("已检查");
                        match Self::partial_eval_condition(condition, env)? {
                            PartialCond::True => {
                                // 条件为 true → 返回对应值（短路，不求值后续条件）
                                return Self::eval_value(&cond_arg.value, env);
                            }
                            PartialCond::False => {
                                // 条件为 false → 继续检查下一个条件
                                continue;
                            }
                            PartialCond::Css(css_str) => {
                                // 条件包含 CSS → CSS 透传
                                // 构建输出：当前条件用部分求值后的 CSS 字符串，后续条件保持原样
                                let mut parts: Vec<String> = Vec::new();
                                // 已求值条件（之前的 false 条件被跳过）
                                parts.push(format!("{css_str}: {}", cond_arg.value));
                                // 后续未求值条件保持原样
                                for (_, a) in args.iter().enumerate().filter(|(j, a)| {
                                    *j > i && a.condition.is_some()
                                }) {
                                    let cond = a.condition.as_ref().expect("已检查");
                                    parts.push(format!("{cond}: {}", a.value));
                                }
                                // else 分支
                                if let Some(else_a) = else_arg {
                                    parts.push(format!("else: {}", else_a.value));
                                }
                                return Ok(Value::String(
                                    format!("if({})", parts.join("; ")),
                                    false,
                                ));
                            }
                        }
                    }
                    // 所有条件都为 false，返回 else 或 Null
                    if let Some(else_a) = else_arg {
                        return Self::eval_value(&else_a.value, env);
                    } else {
                        return Ok(Value::Null);
                    }
                }
                // if(else: value) — else-only 语法，始终返回 value
                if name == "if" && !args.is_empty() && args.iter().all(|a| a.name.as_deref() == Some("else")) {
                    return Self::eval_value(&args[0].value, env);
                }
                // 分离位置参数和关键字参数，展开 spread
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
                // 尝试作为 Sass 函数调用，未定义时 CSS 透传
                match Self::call_function(name, &pos_args, &kw_args, env) {
                    Ok(result) => Ok(result),
                    Err(SassError::UndefinedFunction(_))
                        if !name.contains('.') && !Self::is_known_builtin(name) => {
                        // 真正未定义的非模块限定函数 → CSS 透传（如 c(%), my-func(1px) 等）
                        let mut parts: Vec<String> = pos_args.iter().map(|v| v.to_string()).collect();
                        for (k, v) in &kw_args {
                            parts.push(format!("{k}={v}"));
                        }
                        Ok(Value::String(format!("{name}({})", parts.join(", ")), false))
                    }
                    Err(SassError::Eval(_))
                        if !name.contains('.')
                            && matches!(name.as_str(), "min" | "max" | "clamp") => {
                        // min/max/clamp 参数非数字时，作为 CSS 原生函数透传
                        let arg_str = pos_args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
                        Ok(Value::Calc(format!("{name}({arg_str})")))
                    }
                    Err(e) => Err(e),
                }
            }
            Value::Interp(s) => Ok(Value::String(eval_interp_str(s, env), false)),
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

    /// 求值二元运算。
    pub(crate) fn eval_binop(
        op: &BinOpKind,
        left: &Value,
        right: &Value,
        env: &Env,
    ) -> Result<Value> {
        // 短路求值：and / or 先求值左侧，根据结果决定是否求值右侧
        match op {
            BinOpKind::And => {
                let l = Self::eval_value(left, env)?;
                if !Self::is_truthy(&l) {
                    return Ok(l);  // falsy → 返回左侧
                }
                let r = Self::eval_value(right, env)?;
                return Ok(r);  // truthy → 返回右侧
            }
            BinOpKind::Or => {
                let l = Self::eval_value(left, env)?;
                if Self::is_truthy(&l) {
                    return Ok(l);  // truthy → 返回左侧
                }
                return Self::eval_value(right, env);  // falsy → 返回右侧
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
