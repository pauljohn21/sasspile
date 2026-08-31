use super::*;

/// inspect() 专用格式化——比 Display 更详细。
pub(crate) fn inspect_value(v: &Value) -> String {
    match v {
        Value::List(elements, sep, bracketed) => {
            if elements.is_empty() {
                if *bracketed {
                    return "[]".to_string();
                }
                // 空列表 inspect 输出 ()——不管什么 separator
                return "()".to_string();
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
                .map(|e| {
                    match e {
                        Value::List(inner_items, inner_sep, false) if inner_items.len() > 1 => {
                            let needs_paren = match inner_sep {
                                Separator::Comma => true,
                                _ => inner_sep == sep,
                            };
                            if needs_paren {
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
                            } else {
                                inspect_value(e)
                            }
                        }
                        _ => inspect_value(e),
                    }
                })
                .collect();
            let inner = if elements.len() == 1 {
                match sep {
                    Separator::Comma => {
                        if *bracketed {
                            format!("{},", parts[0])
                        } else {
                            format!("({},)", parts[0])
                        }
                    }
                    Separator::Slash => {
                        if *bracketed {
                            format!("{} /", parts[0])
                        } else {
                            format!("({} /)", parts[0])
                        }
                    }
                    Separator::SlashLiteral => {
                        if *bracketed {
                            format!("{}/", parts[0])
                        } else {
                            format!("({}/)", parts[0])
                        }
                    }
                    // Space 和 Undecided 单元素不需要特殊处理
                    _ => parts.join(sep_str),
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
            if pairs.is_empty() {
                return "()".to_string();
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
                    format!("{}: {}", key_str, val_str)
                })
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
    if !s.contains("#{") && !s.contains('$') {
        return s.to_string();
    }
    // 不含 #{ 嵌套但含 $ → 尝试整体求值（纯变量 $a、表达式 1+2 等）
    if !s.contains("#{") {
        if let Ok(val) = super::eval_simple_expr(s, env) {
            return match &val {
                Value::String(inner, _) => inner.clone(),
                _ => val.to_string(),
            };
        }
    }
    // 回退：逐字符扫描 #{} 嵌套 + $var 变量引用
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
                let s = match val {
                    Value::String(s, _) => s.clone(),
                    _ => val.to_string(),
                };
                result.push_str(&s);
            } else {
                result.push_str(&format!("${var_name}"));
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
