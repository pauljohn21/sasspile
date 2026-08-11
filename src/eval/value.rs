use super::*;
use crate::css::node::CssNode;
use crate::error::{Result, SassError};
use crate::parse::ast::BinOpKind;
use tracing::{instrument, warn};

impl Evaluator {
    pub(crate) fn eval_variable(
        name: &str,
        value: &Value,
        flags: &VarFlags,
        env: &Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        if flags.default && env.has_var(name) {
            return Ok((vec![], env.clone()));
        }
        let val = Self::eval_value(value, env)?;
        Ok((vec![], env.bind(name.to_string(), val)))
    }

    /// 求值值表达式。
    #[instrument(skip(value, env), fields(depth = env.depth), level = "trace")]
    pub(crate) fn eval_value(value: &Value, env: &Env) -> Result<Value> {
        match value {
            Value::Number(..)
            | Value::Color(..)
            | Value::Bool(..)
            | Value::Null
            | Value::Calc(..) => Ok(value.clone()),
            Value::String(s, quoted) => {
                // 处理插值在字符串中
                if s.contains('#') && s.contains('{') {
                    Ok(Value::String(Self::eval_interp_str(s, env), *quoted))
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
                        if let Some(val) = module.vars.get(var_name) {
                            return Ok(val.clone());
                        }
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
                // CSS 函数透传：带命名参数的调用（如 alpha(opacity=0)）作为 CSS 原样输出
                if args.iter().any(|a| a.name.is_some()) {
                    let parts: Vec<String> = args
                        .iter()
                        .map(|a| {
                            let v = Self::eval_value(&a.value, env)?;
                            Ok(if let Some(n) = &a.name {
                                format!("{n}={v}")
                            } else {
                                v.to_string()
                            })
                        })
                        .collect::<Result<_>>()?;
                    return Ok(Value::String(format!("{name}({})", parts.join(", ")), false));
                }
                // if() 惰性求值：只求值选中的分支，避免副作用和类型错误
                if name == "if" && args.len() == 3 {
                    let cond = Self::eval_value(&args[0].value, env)?;
                    if Self::is_truthy(&cond) {
                        return Self::eval_value(&args[1].value, env);
                    } else {
                        return Self::eval_value(&args[2].value, env);
                    }
                }
                let evaluated_args: Vec<Value> = args
                    .iter()
                    .map(|a| Self::eval_value(&a.value, env))
                    .collect::<Result<_>>()?;
                // 展开 spread 参数（arg.spread=true 的参数展开为多个位置参数）
                let mut final_args: Vec<Value> = Vec::new();
                for (i, arg) in args.iter().enumerate() {
                    if arg.spread {
                        match &evaluated_args[i] {
                            Value::List(items, _, _) => final_args.extend(items.iter().cloned()),
                            Value::Map(pairs) => {
                                for (k, v) in pairs {
                                    final_args.push(Value::List(vec![k.clone(), v.clone()], Separator::Space, false));
                                }
                            }
                            other => final_args.push(other.clone()),
                        }
                    } else {
                        final_args.push(evaluated_args[i].clone());
                    }
                }
                Self::call_function(name, &final_args, env)
            }
            Value::Interp(s) => Ok(Value::String(Self::eval_interp_str(s, env), false)),
            Value::BinOp(b) => Self::eval_binop(&b.op, &b.left, &b.right, env),
            Value::UnaryOp(op, v) => {
                let val = Self::eval_value(v, env)?;
                match op {
                    UnaryOp::Neg => match val {
                        Value::Number(n, u) => Ok(Value::Number(-n, u)),
                        _ => Err(SassError::Eval(format!("无法对 {val} 取负"))),
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
        tracing::trace!(
            target: "sasspile::binop",
            op = ?op,
            left = %l, right = %r,
            "binop operands evaluated"
        );
        let result = match op {
            BinOpKind::Add => Self::add(&l, &r),
            BinOpKind::Sub => Self::sub(&l, &r),
            BinOpKind::Mul => Self::mul(&l, &r),
            BinOpKind::Div => Self::div(&l, &r),
            BinOpKind::Mod => Self::modulo(&l, &r),
            BinOpKind::Eq => Ok(Value::Bool(Self::values_eq(&l, &r))),
            BinOpKind::NotEq => Ok(Value::Bool(!Self::values_eq(&l, &r))),
            BinOpKind::And => Ok(r),
            BinOpKind::Or => Ok(r),
            BinOpKind::Lt | BinOpKind::Gt | BinOpKind::LtEq | BinOpKind::GtEq => {
                Self::compare(op, &l, &r)
            }
        };
        if let Ok(v) = &result {
            tracing::trace!(
                target: "sasspile::binop",
                op = ?op,
                result = %v,
                "binop result"
            );
        }
        result
    }

    pub(crate) fn add(l: &Value, r: &Value) -> Result<Value> {
        let l = l.clone();
        let r = r.clone();
        match (l, r) {
            (Value::Number(a, u1), Value::Number(b, u2)) => {
                let unit = u1.or(u2);
                Ok(Value::Number(a + b, unit))
            }
            // 字符串拼接——结果引号跟随左侧
            (Value::String(a, qa), Value::String(b, _)) => Ok(Value::String(format!("{a}{b}"), qa)),
            (Value::String(a, qa), Value::Number(n, u)) => Ok(Value::String(
                format!("{a}{}{}", n, u.as_deref().unwrap_or("")),
                qa,
            )),
            (Value::String(a, qa), Value::Color(c)) => Ok(Value::String(
                format!("{a}#{:02x}{:02x}{:02x}", c.r, c.g, c.b),
                qa,
            )),
            (Value::String(a, qa), Value::Null) => Ok(Value::String(a, qa)),
            (Value::Number(n, u), Value::String(b, qb)) => Ok(Value::String(
                format!("{}{}{b}", n, u.as_deref().unwrap_or("")),
                qb,
            )),
            (Value::Color(c), Value::String(b, qb)) => Ok(Value::String(
                format!("#{:02x}{:02x}{:02x}{b}", c.r, c.g, c.b),
                qb,
            )),
            (Value::Null, Value::String(b, qb)) => Ok(Value::String(b, qb)),
            // String + Calc / Calc + String — 拼接字符串表示
            (Value::String(a, qa), Value::Calc(c)) => Ok(Value::String(format!("{a}{c}"), qa)),
            (Value::Calc(c), Value::String(b, qb)) => Ok(Value::String(format!("{c}{b}"), qb)),
            (Value::Calc(a), Value::Calc(b)) => Ok(Value::String(format!("{a}{b}"), false)),
            // String + Bool / Bool + String
            (Value::String(a, qa), Value::Bool(b)) => Ok(Value::String(format!("{a}{b}"), qa)),
            (Value::Bool(a), Value::String(b, qb)) => Ok(Value::String(format!("{a}{b}"), qb)),
            // 列表拼接
            (Value::List(mut items, sep, _), Value::List(items2, _, _)) => {
                items.extend(items2);
                Ok(Value::List(items, sep, false))
            }
            (Value::List(mut items, sep, _), other) => {
                items.push(other);
                Ok(Value::List(items, sep, false))
            }
            (other, Value::List(items, sep, false)) => {
                let mut new_items = vec![other];
                new_items.extend(items);
                Ok(Value::List(new_items, sep, false))
            }
            _ => Err(SassError::Eval("不支持的 + 运算".into())),
        }
    }
    pub(crate) fn sub(l: &Value, r: &Value) -> Result<Value> {
        let l = l.clone();
        let r = r.clone();
        match (l, r) {
            (Value::Number(a, u1), Value::Number(b, u2)) => {
                let unit = u1.or(u2);
                Ok(Value::Number(a - b, unit))
            }
            // 字符串拼接——用 - 连接
            (Value::String(a, qa), Value::String(b, _)) => {
                Ok(Value::String(format!("{a}-{b}"), qa))
            }
            (Value::String(a, qa), Value::Number(n, u)) => Ok(Value::String(
                format!("{a}-{}{}", n, u.as_deref().unwrap_or("")),
                qa,
            )),
            (Value::String(a, qa), Value::Color(c)) => Ok(Value::String(
                format!("{a}-#{:02x}{:02x}{:02x}", c.r, c.g, c.b),
                qa,
            )),
            (Value::Number(n, u), Value::String(b, qb)) => Ok(Value::String(
                format!("{}{}-{b}", n, u.as_deref().unwrap_or("")),
                qb,
            )),
            (Value::Color(c), Value::String(b, qb)) => Ok(Value::String(
                format!("#{:02x}{:02x}{:02x}-{b}", c.r, c.g, c.b),
                qb,
            )),
            _ => Err(SassError::Eval("不支持的 - 运算".into())),
        }
    }
    pub(crate) fn mul(l: &Value, r: &Value) -> Result<Value> {
        match (l, r) {
            (Value::Number(a, u1), Value::Number(b, u2)) => {
                let unit = if u1.is_some() { u1.clone() } else { u2.clone() };
                Ok(Value::Number(a * b, unit))
            }
            _ => Err(SassError::Eval(format!("无法 {l} * {r}"))),
        }
    }
    pub(crate) fn div(l: &Value, r: &Value) -> Result<Value> {
        match (l, r) {
            (Value::Number(a, u1), Value::Number(b, _)) => {
                if *b == 0.0 {
                    // SCSS: 1/0 = Infinity, -1/0 = -Infinity, 0/0 = NaN
                    if *a == 0.0 {
                        return Ok(Value::Number(f64::NAN, u1.clone()));
                    }
                    return Ok(Value::Number(a / b, u1.clone())); // f64 除零产生 Infinity
                }
                Ok(Value::Number(a / b, u1.clone()))
            }
            // 非数字 / —— 作为斜杠分隔列表保留（如 font: 16px/24px）
            _ => Ok(Value::String(format!("{l}/{r}"), false)),
        }
    }
    pub(crate) fn modulo(l: &Value, r: &Value) -> Result<Value> {
        match (l, r) {
            (Value::Number(a, u), Value::Number(b, _)) => {
                if *b == 0.0 {
                    return Err(SassError::DivideByZero);
                }
                Ok(Value::Number(a % b, u.clone()))
            }
            // Null RHS — % 不是运算符，作为字符串保留
            (l, Value::Null) => Ok(Value::List(
                vec![l.clone(), Value::String("%".to_string(), false)],
                Separator::Space,
                false,
            )),
            // 非数字 % —— 作为空格分隔列表保留
            _ => Ok(Value::List(
                vec![l.clone(), r.clone()],
                Separator::Space,
                false,
            )),
        }
    }
    pub(crate) fn compare(op: &BinOpKind, l: &Value, r: &Value) -> Result<Value> {
        match (l, r) {
            (Value::Number(a, _), Value::Number(b, _)) => {
                let result = match op {
                    BinOpKind::Lt => a < b,
                    BinOpKind::Gt => a > b,
                    BinOpKind::LtEq => a <= b,
                    BinOpKind::GtEq => a >= b,
                    _ => false,
                };
                Ok(Value::Bool(result))
            }
            _ => Err(SassError::Eval(format!("无法比较 {l} 和 {r}"))),
        }
    }

    /// 检查两个单位是否兼容（属于同一物理量类别）。
    pub(crate) fn units_compatible(u1: Option<&str>, u2: Option<&str>) -> bool {
        if u1 == u2 {
            return true;
        }
        if u1.is_none() || u2.is_none() {
            return true;
        }
        // 单位兼容组——同组的单位互相兼容
        const GROUPS: &[&[&str]] = &[
            &["px", "in", "cm", "mm", "pt", "pc", "q"], // 长度
            &["deg", "grad", "rad", "turn"],            // 角度
            &["s", "ms"],                               // 时间
            &["hz", "khz"],                             // 频率
            &["dpi", "dpcm", "dppx"],                   // 分辨率
        ];
        for group in GROUPS {
            let has1 = group.contains(&u1.unwrap());
            let has2 = group.contains(&u2.unwrap());
            if has1 && has2 {
                return true;
            }
        }
        false
    }

    /// inspect() 专用格式化——比 Display 更详细。
    pub(crate) fn inspect_value(v: &Value) -> String {
        match v {
            Value::List(elements, sep, bracketed) => {
                if elements.is_empty() {
                    if *bracketed {
                        return "[]".to_string();
                    }
                    if matches!(sep, Separator::Comma) {
                        return "()".to_string();
                    }
                    return String::new();
                }
                let sep_str = match sep {
                    Separator::Comma => ", ",
                    Separator::Space => " ",
                    Separator::Slash => " / ",
                    Separator::Undecided => " ",
                };
                let parts: Vec<String> = elements.iter().map(Self::inspect_value).collect();
                let inner = if elements.len() == 1 && matches!(sep, Separator::Comma) {
                    if *bracketed {
                        format!("{},", parts[0])
                    } else {
                        format!("({},)", parts[0])
                    }
                } else {
                    parts.join(sep_str)
                };
                if *bracketed {
                    format!("[{}]", inner)
                } else {
                    inner
                }
            }
            Value::Map(pairs) => {
                let parts: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", Self::inspect_value(k), Self::inspect_value(v)))
                    .collect();
                format!("({})", parts.join(", "))
            }
            Value::String(s, quoted) => {
                if *quoted {
                    format!("\"{s}\"")
                } else {
                    s.clone()
                }
            }
            Value::Null => "null".to_string(),
            _ => v.to_string(),
        }
    }

    pub(crate) fn values_eq(l: &Value, r: &Value) -> bool {
        match (l, r) {
            (Value::Number(a, _), Value::Number(b, _)) => {
                if a.is_nan() && b.is_nan() {
                    return true;
                }
                if a.is_infinite() && b.is_infinite() && a.signum() == b.signum() {
                    return true;
                }
                (a - b).abs() < f64::EPSILON
            }
            (Value::String(a, _), Value::String(b, _)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Color(a), Value::Color(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::List(a, _, _), Value::List(b, _, _)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| Self::values_eq(x, y))
            }
            (Value::Map(a), Value::Map(b)) => {
                a.len() == b.len()
                    && a.iter().all(|(k, v)| {
                        b.iter()
                            .any(|(k2, v2)| Self::values_eq(k, k2) && Self::values_eq(v, v2))
                    })
            }
            _ => false,
        }
    }

    /// 求值插值字符串 #{...}。
    pub(crate) fn eval_interp_str(s: &str, env: &Env) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '#' && chars.peek() == Some(&'{') {
                chars.next(); // 消费 {
                let mut expr = String::new();
                let mut depth = 1;
                while let Some(ch) = chars.next() {
                    if ch == '{' {
                        depth += 1;
                        expr.push(ch);
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        expr.push(ch);
                    } else {
                        expr.push(ch);
                    }
                }
                // 尝试求值表达式
                if let Ok(val) = Self::eval_simple_expr(&expr, env) {
                    // 插值上下文中字符串去引号
                    let s = match &val {
                        Value::String(s, _) => s.clone(),
                        _ => val.to_string(),
                    };
                    result.push_str(&s);
                } else {
                    result.push_str(&expr);
                }
            } else {
                result.push(c);
            }
        }
        result
    }

    /// 简单表达式求值（用于插值）。
    pub(crate) fn eval_simple_expr(expr: &str, env: &Env) -> Result<Value> {
        let expr = expr.trim();
        // 变量引用
        if let Some(name) = expr.strip_prefix('$') {
            return env
                .lookup(name)
                .cloned()
                .ok_or_else(|| SassError::UndefinedVariable(name.to_string()));
        }
        // 尝试作为数字
        if let Ok(n) = expr.parse::<f64>() {
            return Ok(Value::Number(n, None));
        }
        // 尝试词法分析 + 解析
        let tokens: Vec<_> = crate::lex::Lexer::new(expr)
            .filter(|t| {
                !matches!(
                    t.as_ref(),
                    Ok(crate::lex::token::Token::Whitespace) | Ok(crate::lex::token::Token::Eof)
                )
            })
            .collect::<crate::error::Result<Vec<_>>>()?;
        let mut parser = crate::parse::Parser::new(&tokens);
        let v = parser.parse_value()?;
        Self::eval_value(&v, env)
    }
}
