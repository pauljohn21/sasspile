//! @media/@supports 参数求值——插值、变量替换、表达式求值。
//!
//! 对 `@media` 和 `@supports` 的参数做以下处理：
//! - `#{...}` 插值求值
//! - `$var` 变量替换
//! - `(expr: expr)` declaration 两侧表达式求值
//! - media feature range 语法中的表达式求值

use super::*;

impl Evaluator {
    /// 求值 @media/@supports 参数中的插值 (#{...})、变量 ($var) 和表达式。
    ///
    /// 对于 @supports，参数中的 `(expr: expr)` declaration 两侧的表达式需要求值。
    /// 对于 @media，media feature 中的表达式（如 `500px + 100px`）需要求值。
    /// 同时处理 `#{...}` 插值和 `$var` 变量替换。
    pub(crate) fn eval_at_params(at_rule: &str, params: &str, env: &Env) -> String {
        // 快速路径：不含 #{} 或 $ 或 + 或 - 或数字的参数直接返回
        // 但 @media/@supports 始终需要逻辑操作符空格规范化
        let needs_eval = params.contains("#{")
            || params.contains('$')
            || (matches!(at_rule, "supports" | "media") && Self::params_has_expr(params));
        match needs_eval {
            false => {
                // 即使无表达式求值，@media/@supports 也需要空格规范化
                return match matches!(at_rule, "supports" | "media") {
                    true => Self::normalize_media_params(params.to_string()),
                    false => params.to_string(),
                };
            }
            true => {}
        }

        // 先做 #{} 插值和 $var 替换
        let after_interp = crate::eval::value::eval_interp_str(params, env);

        // 如果是 @supports 或 @media，再对括号内 declaration 做表达式求值
        let evaluated = match matches!(at_rule, "supports" | "media") {
            true => Self::eval_expr_in_params(&after_interp, env),
            false => after_interp,
        };

        // 对 @media/@supports 参数做逻辑操作符空格规范化
        match matches!(at_rule, "supports" | "media") {
            true => Self::normalize_media_params(evaluated),
            false => evaluated,
        }
    }

    /// 规范化 @media/@supports 参数中逻辑操作符的空格。
    ///
    /// - `not(` → `not (` (not 后必须有空格)
    /// - `and(` → `and (` (and 后必须有空格)
    /// - `or(` → `or (` (or 后必须有空格)
    /// - `)and` → `) and` (and/or 前必须是空格)
    /// - `)or` → `) or` (and/or 前必须是空格)
    fn normalize_media_params(params: String) -> String {
        let chars: Vec<char> = params.chars().collect();
        let mut result = String::with_capacity(params.len() + 4);
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            // 检查是否是 not/and/or 单词（独立单词，非其他标识符一部分）
            let word = if c == 'n' || c == 'a' || c == 'o' {
                let rest: String = chars[i..].iter().take(4).collect();
                if rest.starts_with("not") && Self::is_word_boundary(chars.get(i + 3)) {
                    Some("not")
                } else if rest.starts_with("and") && Self::is_word_boundary(chars.get(i + 3)) {
                    Some("and")
                } else if rest.starts_with("or") && Self::is_word_boundary(chars.get(i + 2)) {
                    Some("or")
                } else {
                    None
                }
            } else {
                None
            };

            match word {
                Some(op) => {
                    result.push_str(op);
                    let op_len = op.len();
                    // 跳过操作符后的已有空格，定位到下一个非空格字符
                    let mut j = i + op_len;
                    while j < chars.len() && chars[j] == ' ' {
                        j += 1;
                    }
                    // 如果跳过了空格或下一个是 (，确保有且仅有一个空格
                    let skipped_spaces = j > i + op_len;
                    if skipped_spaces || (j < chars.len() && chars[j] == '(') {
                        result.push(' ');
                    }
                    // 跳过所有已处理的字符（操作符 + 空格），下次从 j 继续
                    i = j;
                }
                None => {
                    // 检查 ) 后是否直接跟 and/or（需要插入空格）
                    if c == ')' {
                        result.push(')');
                        // 跳过 ) 后的已有空格
                        let mut j = i + 1;
                        while j < chars.len() && chars[j] == ' ' {
                            j += 1;
                        }
                        // 如果跳过了空格或下一个是 and/or，确保有且仅有一个空格
                        let skipped_spaces = j > i + 1;
                        let is_logical = j < chars.len()
                            && if chars[j] == 'n' || chars[j] == 'a' {
                                let rest: String = chars[j..].iter().take(4).collect();
                                rest.starts_with("not") && Self::is_word_boundary(chars.get(j + 3))
                                    || rest.starts_with("and") && Self::is_word_boundary(chars.get(j + 3))
                            } else if chars[j] == 'o' {
                                let rest: String = chars[j..].iter().take(3).collect();
                                rest.starts_with("or") && Self::is_word_boundary(chars.get(j + 2))
                            } else {
                                false
                            };
                        if skipped_spaces || is_logical {
                            result.push(' ');
                        }
                        // 跳过所有已处理的字符（) + 空格），下次从 j 继续
                        i = j;
                    } else {
                        result.push(c);
                        i += 1;
                    }
                }
            }
        }
        result
    }

    /// 检查给定位置的字符是否为单词边界（空格、括号、字符串结束等）。
    fn is_word_boundary(c: Option<&char>) -> bool {
        match c {
            None => true,
            Some(ch) => !ch.is_alphanumeric() && *ch != '-' && *ch != '_',
        }
    }

    /// 检查参数是否可能包含需要求值的表达式。
    fn params_has_expr(params: &str) -> bool {
        params.contains('+')
            || params.contains('-')
            || params.contains(" * ")
            || params.contains(" / ")
            || params.contains(" < ")
            || params.contains(" > ")
            || params.contains(" = ")
    }

    /// 对 @supports/@media 参数中的括号内表达式做求值。
    fn eval_expr_in_params(params: &str, env: &Env) -> String {
        let mut result = String::new();
        let chars = params.chars().peekable();
        let mut paren_depth = 0;
        let mut paren_content = String::new();

        for c in chars {
            match c {
                '(' => {
                    paren_depth += 1;
                    match paren_depth == 1 {
                        true => {
                            paren_content.clear();
                            continue;
                        }
                        false => {}
                    }
                }
                ')' => {
                    paren_depth -= 1;
                    match paren_depth == 0 {
                        true => {
                            let evaluated = Self::eval_paren_content(&paren_content, env);
                            result.push('(');
                            result.push_str(&evaluated);
                            result.push(')');
                            continue;
                        }
                        false => {}
                    }
                }
                _ => {}
            }
            match paren_depth >= 1 {
                true => paren_content.push(c),
                false => result.push(c),
            }
        }

        result
    }

    /// 求值括号内的内容——按冒号或 range 运算符分割，对各部分做表达式求值。
    fn eval_paren_content(content: &str, env: &Env) -> String {
        // 处理嵌套括号——先找最外层冒号
        let mut depth = 0;
        let mut colon_pos = None;
        let chars: Vec<char> = content.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                ':' if depth == 0 => {
                    colon_pos = Some(i);
                    break;
                }
                _ => {}
            }
        }

        // 检查是否是 `not`、`and`、`or` 前缀
        let trimmed = content.trim();
        match trimmed == "not" || trimmed == "and" || trimmed == "or" {
            true => return content.to_string(),
            false => {}
        }

        if let Some(pos) = colon_pos {
            let lhs = content[..pos].trim();
            let rhs = content[pos + 1..].trim();
            let eval_lhs = Self::try_eval_expr(lhs, env);
            let eval_rhs = Self::try_eval_expr(rhs, env);
            format!("{eval_lhs}: {eval_rhs}")
        } else {
            // 没有冒号——可能是 media feature range 或简单标识符
            Self::eval_media_feature(content, env)
        }
    }

    /// 求值 media feature 中的表达式。
    ///
    /// 处理 range 语法：`width < 500px + 100px` → `width < 600px`
    fn eval_media_feature(content: &str, env: &Env) -> String {
        let content = content.trim();

        // 如果没有运算符，直接返回
        let has_op = content.contains('+')
            || content.contains('-')
            || content.contains('*')
            || content.contains('/')
            || content.contains('$')
            || content.contains(" < ")
            || content.contains(" > ")
            || content.contains(" = ")
            || content.contains(" <= ")
            || content.contains(" >= ");
        match has_op {
            false => return content.to_string(),
            true => {}
        }

        // 按 <, >, = 分割，对各段分别做表达式求值
        let mut result = String::new();
        let mut current_seg = String::new();
        let chars: Vec<char> = content.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            match c == '<' || c == '>' || c == '=' {
                true => {
                // 对前面的 segment 做表达式求值（保留前导空格）
                let leading_ws: String = current_seg.chars().take_while(|&c| c == ' ').collect();
                let trailing_ws: String = current_seg
                    .chars()
                    .rev()
                    .take_while(|&c| c == ' ')
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                let seg_core = current_seg.trim();
                let eval_seg = Self::try_eval_expr(seg_core, env);
                result.push_str(&leading_ws);
                result.push_str(&eval_seg);
                result.push_str(&trailing_ws);

                // 收集运算符
                let mut op = String::new();
                op.push(c);
                match i + 1 < chars.len() && chars[i + 1] == '=' {
                    true => {
                        op.push('=');
                        i += 1;
                    }
                    false => {}
                }
                result.push_str(&op);

                // 收集运算符后的空格
                    current_seg.clear();
                    while i + 1 < chars.len() && chars[i + 1] == ' ' {
                        current_seg.push(' ');
                        i += 1;
                    }
                }
                false => {
                    current_seg.push(c);
                }
            }
            i += 1;
        }
        // 对最后一个 segment 做表达式求值
        let leading_ws: String = current_seg.chars().take_while(|&c| c == ' ').collect();
        let trailing_ws: String = current_seg
            .chars()
            .rev()
            .take_while(|&c| c == ' ')
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        let seg_core = current_seg.trim();
        let eval_seg = Self::try_eval_expr(seg_core, env);
        result.push_str(&leading_ws);
        result.push_str(&eval_seg);
        result.push_str(&trailing_ws);

        result.trim().to_string()
    }

    /// 尝试求值表达式，失败则返回原文。
    fn try_eval_expr(expr: &str, env: &Env) -> String {
        let expr = expr.trim();
        match expr.is_empty() {
            true => return String::new(),
            false => {}
        }
        // 纯标识符（如 a, b, --a, width）不需要求值
        let is_pure_ident = expr
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
            && !expr.contains('+')
            && !expr.contains('*')
            && !expr.contains('/')
            && !expr.contains('$');
        match is_pure_ident {
            true => return expr.to_string(),
            false => {}
        }
        // 尝试作为 Sass 表达式求值
        match crate::eval::value::eval_simple_expr(expr, env) {
            Ok(val) => {
                let s = match &val {
                    crate::eval::Value::String(s, _) => s.clone(),
                    _ => val.to_string(),
                };
                match s == expr {
                    true => expr.to_string(),
                    false => s,
                }
            }
            Err(_) => expr.to_string(),
        }
    }
}
