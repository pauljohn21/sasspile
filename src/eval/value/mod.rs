use super::*;
use crate::css::node::CssNode;
use crate::error::{Result, SassError};
use crate::parse::ast::BinOpKind;
use crate::__tracing::{debug, warn};

mod display;
mod ops;

pub(crate) use display::{eval_interp_segments, eval_interp_str, eval_property_name, eval_simple_expr, inspect_value};
pub(crate) use ops::{add, compare, div, modulo, mul, sub, units_compatible, values_eq};

/// 部分条件求值结果。
enum PartialCond {
    True,
    False,
    Css(String),
}

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
        // 分割 namespace.var_name
        let parts: Vec<&str> = name.splitn(2, '.').collect();
        if parts.len() == 2 {
            let ns = parts[0];
            let var_name = parts[1];
            // 更新命名空间模块中的变量
            if env.get_namespace(ns).is_some() {
                let env = env.with_namespace_var(ns, var_name, val);
                return Ok((vec![], env));
            }
        }
        // 找不到命名空间——忽略
        return Ok((vec![], env));
    }
    if flags.default {
        // !default 赋值：先检查 pending_config（with 配置覆盖值）
        // Sass 中 - 和 _ 在变量名中等价
        let normalized = name.replace('-', "_");
        let config_val = env.get_pending_config().get(&normalized)
            .or_else(|| env.get_pending_config().get(name))
            .cloned();
        if let Some(val) = config_val {
            let consumed_key = env.get_pending_config()
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
        // 无配置覆盖：已有变量则跳过
        if env.has_var(name) {
            return Ok((vec![], env));
        }
    }
    let val = Self::eval_value(value, &env)?;
    let new_env = env.bind(name.to_string(), val.clone());
    // !global 变量同时写入 global_writes，供 eval_rule 传播到外层
    if flags.global {
        Ok((vec![], new_env.add_global_write(name.to_string(), val)))
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
                if env.is_plain_css() {
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
            Value::Interp(segments) => {
                if env.is_plain_css() {
                    return Err(SassError::Eval("Interpolation isn't allowed in plain CSS.".into()));
                }
                // 求值插值片段，拼接为字符串
                let val_str = eval_interp_segments(segments, env);
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
                } else {
                    let val = Value::String(val_str, false);
                    if Self::is_truthy(&val) {
                        Ok(PartialCond::True)
                    } else {
                        Ok(PartialCond::False)
                    }
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
                    if let Some(module) = env.get_namespace(ns) {
                        if let Some(val) = module.all_vars().find(|(k, _)| *k == var_name).map(|(_, v)| v) {
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
                // if() 命名参数语法：if(condition, $if-true: val1, $if-false: val2)
                if name == "if" && args.iter().any(|a| a.name.is_some()) {
                    // 第一个参数应该是条件（位置参数）
                    let pos_args: Vec<&Arg> = args.iter().filter(|a| a.name.is_none() && a.condition.is_none()).collect();
                    if pos_args.len() == 1 {
                        let cond = Self::eval_value(&pos_args[0].value, env)?;
                        let if_true = args.iter().find(|a| a.name.as_deref() == Some("if-true") || a.name.as_deref() == Some("$if-true"));
                        let if_false = args.iter().find(|a| a.name.as_deref() == Some("if-false") || a.name.as_deref() == Some("$if-false"));
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
            Value::Interp(segments) => Ok(Value::String(eval_interp_segments(segments, env), false)),
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

    /// 简化 calc() 表达式——纯数字时去掉 calc() 包装。
    ///
    /// `calc(1px)` → `Value::Number(1, "px")`
    /// `calc(1px + 2px)` → `Value::Number(3, "px")`（同单位简化）
    /// `calc(1px + 2%)` → `Value::Calc("calc(1px + 2%)")`（不同单位保留）
    fn simplify_calc(s: &str) -> Value {
        // 尝试提取 calc(内容) 的内部表达式
        let inner = s.strip_prefix("calc(").and_then(|s| s.strip_suffix(")"));
        let inner = match inner {
            Some(i) => i.trim(),
            None => return Value::Calc(s.to_string()),
        };
        // CSS 常量替换：pi → 3.1415926536, e → 2.7182818285
        let inner = match inner {
            "pi" => return Value::Number(std::f64::consts::PI, None),
            "e" => return Value::Number(std::f64::consts::E, None),
            _ => inner,
        };
        // 尝试解析为纯数字 + 可选单位
        if let Some(v) = Self::parse_simple_number(inner) {
            return v;
        }
        // 尝试解析同单位加减法：1px + 2px, 1px - 2px
        if let Some(v) = Self::try_simplify_same_unit_arith(inner) {
            return v;
        }
        Value::Calc(s.to_string())
    }

    /// 尝试简化同单位加减法：`1px + 2px` → `3px`，`1px - 2px` → `-1px`。
    /// 只处理简单的 `数字单位 op 数字单位` 形式。
    fn try_simplify_same_unit_arith(s: &str) -> Option<Value> {
        let s = s.trim();
        // 查找 + 或 - 作为运算符（前后有空格的）
        // 注意：- 也可能是负号，所以只匹配 " + " 和 " - "
        let op_idx = Self::find_calc_operator(s)?;
        let op_str: &str = s[op_idx..op_idx + 3].trim();
        let left = s[..op_idx].trim();
        let right = s[op_idx + 3..].trim();
        let left_val = Self::parse_simple_number(left)?;
        let right_val = Self::parse_simple_number(right)?;
        match (&left_val, &right_val) {
            // 乘法：数字 * 无单位数字 → 保留单位
            (Value::Number(a, ua), Value::Number(b, None)) if op_str == "*" => {
                Some(Value::Number(a * b, ua.clone()))
            }
            (Value::Number(a, None), Value::Number(b, ub)) if op_str == "*" => {
                Some(Value::Number(a * b, ub.clone()))
            }
            // 除法：数字 / 无单位数字 → 保留单位
            (Value::Number(a, ua), Value::Number(b, None)) if op_str == "/" && *b != 0.0 => {
                Some(Value::Number(a / b, ua.clone()))
            }
            // 同单位加减法
            (Value::Number(a, ua), Value::Number(b, ub)) if ua == ub => {
                match op_str {
                    "+" => Some(Value::Number(a + b, ua.clone())),
                    "-" => Some(Value::Number(a - b, ua.clone())),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// 查找 calc 表达式中的运算符位置（" + ", " - ", " * ", " / "）。
    /// 返回运算符前空格的索引位置。
    fn find_calc_operator(s: &str) -> Option<usize> {
        let mut depth = 0i32;
        for (i, c) in s.char_indices() {
            match c {
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                ' ' if depth == 0 => {
                    let rest = &s[i..];
                    if rest.starts_with(" + ") || rest.starts_with(" - ")
                        || rest.starts_with(" * ") || rest.starts_with(" / ") {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// 尝试将字符串解析为纯数字（含单位）。
    /// `1px` → `Value::Number(1, "px")`，`42` → `Value::Number(42, None)`
    fn parse_simple_number(s: &str) -> Option<Value> {
        let s = s.trim();
        // CSS 常量
        match s {
            "pi" => return Some(Value::Number(std::f64::consts::PI, None)),
            "e" => return Some(Value::Number(std::f64::consts::E, None)),
            _ => {}
        }
        // 去掉前导 +
        let s = s.strip_prefix('+').unwrap_or(s);
        // 找到数字部分的结尾
        let split = s.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-');
        match split {
            None => {
                // 纯数字
                s.parse::<f64>().ok().map(|n| Value::Number(n, None))
            }
            Some(idx) if idx > 0 => {
                let (num_str, unit) = s.split_at(idx);
                let n = num_str.parse::<f64>().ok()?;
                let unit = unit.trim();
                // 单位必须是纯字母标识符（不含空格、运算符等）
                if unit.is_empty() {
                    return Some(Value::Number(n, None));
                }
                if !unit.chars().all(|c| c.is_ascii_alphabetic()) {
                    return None;
                }
                Some(Value::Number(n, Some(unit.to_string())))
            }
            _ => None,
        }
    }
}
