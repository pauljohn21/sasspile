use super::*;

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
            let parts: Vec<String> = elements.iter().map(inspect_value).collect();
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
                format!("[{inner}]")
            } else {
                inner
            }
        }
        Value::Map(pairs) => {
            let parts: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("{}: {}", inspect_value(k), inspect_value(v)))
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

/// 求值属性名——支持 $var 和 #{...} 插值。
///
/// 例如 `$prop: color; .foo { $prop: red; }` → `.foo { color: red; }`
/// `border-#{$side}: 1px;` → `border-left: 1px;`
pub(crate) fn eval_property_name(property: &str, env: &Env) -> String {
    // 快速路径：不含 $ 或 #{} 的属性名直接返回
    if !property.contains('$') && !property.contains("#{") {
        return property.to_string();
    }
    // 处理 #{} 插值
    let mut result = String::new();
    let mut chars = property.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '#' && chars.peek() == Some(&'{') {
            chars.next(); // 消费 {
            let mut expr = String::new();
            let mut depth = 1;
            for ch in chars.by_ref() {
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
            if let Ok(val) = super::eval_simple_expr(&expr, env) {
                result.push_str(&val.to_string());
            } else {
                result.push_str(&expr);
            }
        } else if c == '$' {
            // 读取变量名
            let mut var_name = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                    var_name.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            if let Some(val) = env.lookup(&var_name) {
                result.push_str(&val.to_string());
            } else {
                result.push_str(&format!("${var_name}"));
            }
        } else {
            result.push(c);
        }
    }
    result
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
            for ch in chars.by_ref() {
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
            if let Ok(val) = super::eval_simple_expr(&expr, env) {
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
pub(crate) fn eval_simple_expr(expr: &str, env: &Env) -> crate::error::Result<Value> {
    let expr = expr.trim();
    // 变量引用
    if let Some(name) = expr.strip_prefix('$') {
        return env
            .lookup(name)
            .cloned()
            .ok_or_else(|| crate::error::SassError::UndefinedVariable(name.to_string()));
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
    super::Evaluator::eval_value(&v, env)
}
