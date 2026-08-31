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
        let needs_eval = params.contains("#{")
            || params.contains('$')
            || (matches!(at_rule, "supports" | "media")
                && Self::params_has_expr(params));
        if !needs_eval {
            return params.to_string();
        }

        // 先做 #{} 插值和 $var 替换
        let after_interp = crate::eval::value::eval_interp_str(params, env);

        // 如果是 @supports 或 @media，再对括号内 declaration 做表达式求值
        if matches!(at_rule, "supports" | "media") {
            Self::eval_expr_in_params(&after_interp, env)
        } else {
            after_interp
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
            if c == '(' {
                paren_depth += 1;
                if paren_depth == 1 {
                    paren_content.clear();
                    continue;
                }
            } else if c == ')' {
                paren_depth -= 1;
                if paren_depth == 0 {
                    let evaluated = Self::eval_paren_content(&paren_content, env);
                    result.push('(');
                    result.push_str(&evaluated);
                    result.push(')');
                    continue;
                }
            }
            if paren_depth >= 1 {
                paren_content.push(c);
            } else {
                result.push(c);
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
        if trimmed == "not" || trimmed == "and" || trimmed == "or" {
            return content.to_string();
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
        if !content.contains('+')
            && !content.contains('-')
            && !content.contains('*')
            && !content.contains('/')
            && !content.contains('$')
            && !content.contains(" < ")
            && !content.contains(" > ")
            && !content.contains(" = ")
            && !content.contains(" <= ")
            && !content.contains(" >= ")
        {
            return content.to_string();
        }

        // 按 <, >, = 分割，对各段分别做表达式求值
        let mut result = String::new();
        let mut current_seg = String::new();
        let chars: Vec<char> = content.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '<' || c == '>' || c == '=' {
                // 对前面的 segment 做表达式求值（保留前导空格）
                let leading_ws: String = current_seg
                    .chars()
                    .take_while(|&c| c == ' ')
                    .collect();
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
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    op.push('=');
                    i += 1;
                }
                result.push_str(&op);

                // 收集运算符后的空格
                current_seg.clear();
                while i + 1 < chars.len() && chars[i + 1] == ' ' {
                    current_seg.push(' ');
                    i += 1;
                }
            } else {
                current_seg.push(c);
            }
            i += 1;
        }
        // 对最后一个 segment 做表达式求值
        let leading_ws: String = current_seg
            .chars()
            .take_while(|&c| c == ' ')
            .collect();
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
        if expr.is_empty() {
            return String::new();
        }
        // 纯标识符（如 a, b, --a, width）不需要求值
        if expr.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
            && !expr.contains('+')
            && !expr.contains('*')
            && !expr.contains('/')
            && !expr.contains('$')
        {
            return expr.to_string();
        }
        // 尝试作为 Sass 表达式求值
        match crate::eval::value::eval_simple_expr(expr, env) {
            Ok(val) => {
                let s = match &val {
                    crate::eval::Value::String(s, _) => s.clone(),
                    _ => val.to_string(),
                };
                if s == expr {
                    expr.to_string()
                } else {
                    s
                }
            }
            Err(_) => expr.to_string(),
        }
    }
}
