use super::*;
use std::fmt::Write;

/// `inspect()` 专用格式化——比 Display 更详细。
pub(crate) fn inspect_value(v: &Value) -> String {
    match v {
        Value::List(elements, sep, bracketed) => {
            match elements.is_empty() {
                true => {
                    match *bracketed {
                        true => return "[]".to_string(),
                        false => return "()".to_string(),
                    }
                }
                false => {}
            }
            let sep_str = match sep {
                Separator::Comma => ", ",
                Separator::Space => " ",
                Separator::Slash => " / ",
                Separator::SlashLiteral => "/",
                Separator::Undecided => " ",
            };
            // 嵌套列表元素：如果元素是列表，根据 separator 规则决定是否用括号包裹
            // 规则：comma-separated 内层列表总是需要括号；
            //       space/slash 内层列表在外层 separator 相同时需要括号
            let parts: Vec<String> = elements
                .iter()
                .map(|e| match e {
                    Value::List(inner_items, inner_sep, false) if inner_items.len() > 1 => {
                        let needs_paren = match inner_sep {
                            Separator::Comma => true,
                            _ => inner_sep == sep,
                        };
                        match needs_paren {
                            true => {
                            let inner_parts: Vec<String> =
                                inner_items.iter().map(inspect_value).collect();
                            let inner_sep_str = match inner_sep {
                                Separator::Comma => ", ",
                                Separator::Space => " ",
                                Separator::Slash => " / ",
                                Separator::SlashLiteral => "/",
                                Separator::Undecided => " ",
                            };
                            format!("({})", inner_parts.join(inner_sep_str))
                            }
                            false => inspect_value(e),
                        }
                    }
                    _ => inspect_value(e),
                })
                .collect();
            let inner = match elements.len() == 1 {
                true => match sep {
                    Separator::Comma => {
                        match *bracketed {
                            true => format!("{},", parts[0]),
                            false => format!("({},)", parts[0]),
                        }
                    }
                    Separator::Slash => {
                        match *bracketed {
                            true => format!("{} /", parts[0]),
                            false => format!("({} /)", parts[0]),
                        }
                    }
                    Separator::SlashLiteral => {
                        match *bracketed {
                            true => format!("{}/", parts[0]),
                            false => format!("({}/)", parts[0]),
                        }
                    }
                    _ => parts.join(sep_str),
                },
                false => parts.join(sep_str),
            };
            match *bracketed {
                true => format!("[{inner}]"),
                false => inner,
            }
        }
        Value::Map(pairs) => {
            match pairs.is_empty() {
                true => return "()".to_string(),
                false => {}
            }
            let parts: Vec<String> = pairs
                .iter()
                .map(|(k, v)| {
                    // Map 键：comma-separated 列表需要括号包裹
                    let key_str = match k {
                        Value::List(items, Separator::Comma, false) if items.len() > 1 => {
                            let inner: Vec<String> = items.iter().map(inspect_value).collect();
                            format!("({})", inner.join(", "))
                        }
                        _ => inspect_value(k),
                    };
                    // Map 值：comma-separated 列表需要括号包裹
                    let val_str = match v {
                        Value::List(items, Separator::Comma, false) if items.len() > 1 => {
                            let inner: Vec<String> = items.iter().map(inspect_value).collect();
                            format!("({})", inner.join(", "))
                        }
                        _ => inspect_value(v),
                    };
                    format!("{key_str}: {val_str}")
                })
                .collect();
            format!("({})", parts.join(", "))
        }
        Value::String(s, quoted) => {
            match *quoted {
                true => format!("\"{s}\""),
                false => s.clone(),
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
    match !property.contains('$') && !property.contains("#{") {
        true => return property.to_string(),
        false => {}
    }
    // 处理 #{} 插值
    let mut result = String::new();
    let mut chars = property.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '#' if chars.peek() == Some(&'{') => {
                chars.next(); // 消费 {
                let mut expr = String::new();
                let mut depth = 1;
                for ch in chars.by_ref() {
                    match ch {
                        '{' => {
                            depth += 1;
                            expr.push(ch);
                        }
                        '}' => {
                            depth -= 1;
                            match depth == 0 {
                                true => break,
                                false => {}
                            }
                            expr.push(ch);
                        }
                        _ => expr.push(ch),
                    }
                }
                match super::eval_simple_expr(&expr, env) {
                    Ok(val) => result.push_str(&val.to_string()),
                    Err(_) => result.push_str(&expr),
                }
            }
            '$' => {
                // 读取变量名
                let mut var_name = String::new();
                while let Some(&ch) = chars.peek() {
                    match ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                        true => {
                            var_name.push(ch);
                            chars.next();
                        }
                        false => break,
                    }
                }
                match env.lookup(&var_name) {
                    Some(val) => result.push_str(&val.to_string()),
                    None => { let _ = write!(result, "${var_name}"); }
                }
            }
            _ => result.push(c),
        }
    }
    result
}

/// 求值插值片段列表——保留表达式与文本的边界。
///
/// 对每个 `Expr` 片段调用 `eval_simple_expr` 求值并去引号，
/// 对 `Text` 片段直接输出。
#[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", skip(env), fields(segments = ?segments.len())))]
pub(crate) fn eval_interp_segments(segments: &[InterpSegment], env: &Env) -> String {
    let mut result = String::new();
    for seg in segments {
        match seg {
            InterpSegment::Expr(expr) => {
                if let Ok(val) = super::eval_simple_expr(expr, env) {
                    let s = match &val {
                        Value::String(s, _) => s.clone(),
                        Value::Null => continue, // #{null} 输出为空
                        _ => val.to_string(),
                    };
                    result.push_str(&s);
                } else {
                    // 求值失败——回退到逐字符处理
                    result.push_str(&eval_interp_str(expr, env));
                }
            }
            InterpSegment::Text(text) => {
                result.push_str(text);
            }
        }
    }
    result
}

/// 求值插值字符串 #{...}。
///
/// 先尝试用 `eval_simple_expr` 整体求值（处理纯变量 `$a`、数字、表达式），
/// 失败时回退到逐字符扫描嵌套 `#{}` 模式（处理混合文本 `prefix#{expr}suffix`）。
#[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", skip(env), fields(input = %s)))]
pub(crate) fn eval_interp_str(s: &str, env: &Env) -> String {
    // 快速路径：不含 #{ 也不含 $ 的纯文本直接返回
    match !s.contains("#{") && !s.contains('$') {
        true => return s.to_string(),
        false => {}
    }
    // 不含 #{ 嵌套但含 $ → 尝试整体求值（纯变量 $a、表达式 1+2 等）
    match (!s.contains("#{"))
        .then(|| super::eval_simple_expr(s, env))
        .and_then(|res| res.ok())
    {
        Some(val) => {
            return match &val {
                Value::String(inner, _) => inner.clone(),
                _ => val.to_string(),
            };
        }
        None => {}
    }
    // 回退：逐字符扫描 #{} 嵌套 + $var 变量引用
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '#' if chars.peek() == Some(&'{') => {
                chars.next(); // 消费 {
                let mut expr = String::new();
                let mut depth = 1;
                for ch in chars.by_ref() {
                    match ch {
                        '{' => {
                            depth += 1;
                            expr.push(ch);
                        }
                        '}' => {
                            depth -= 1;
                            match depth == 0 {
                                true => break,
                                false => {}
                            }
                            expr.push(ch);
                        }
                        _ => expr.push(ch),
                    }
                }
                // 尝试求值表达式
                match super::eval_simple_expr(&expr, env) {
                    Ok(val) => {
                        // 插值上下文中字符串去引号
                        let s = match &val {
                            Value::String(s, _) => s.clone(),
                            _ => val.to_string(),
                        };
                        result.push_str(&s);
                    }
                    Err(_) => result.push_str(&expr),
                }
            }
            '$' => {
                // 读取变量名
                let mut var_name = String::new();
                while let Some(&ch) = chars.peek() {
                    match ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                        true => {
                            var_name.push(ch);
                            chars.next();
                        }
                        false => break,
                    }
                }
                match env.lookup(&var_name) {
                    Some(val) => {
                        let s = match val {
                            Value::String(s, _) => s.clone(),
                            _ => val.to_string(),
                        };
                        result.push_str(&s);
                    }
                    None => { let _ = write!(result, "${var_name}"); }
                }
            }
            _ => result.push(c),
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
                Ok(crate::lex::token::Token::Whitespace | crate::lex::token::Token::Eof)
            )
        })
        .collect::<crate::error::Result<Vec<_>>>()?;
    let mut parser = crate::parse::Parser::new(&tokens);
    let v = parser.parse_value()?;
    super::Evaluator::eval_value(&v, env)
}
