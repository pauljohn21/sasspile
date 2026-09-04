//! `calc()` / `clamp()` / `min()` / `max()` 表达式简化。
//!
//! `simplify_calc` 对 `calc()` 内容做纯数字简化：
//! - 纯数字/常量 → `Value::Number`
//! - 同单位算术 → 计算结果
//! - 嵌套 min/max → 简化
//! - 科学计数法、pi/e 常量替换
//! - 多余括号去除

use super::*;

impl Evaluator {
    /// 简化 `calc()` 表达式——纯数字时去掉 `calc()` 包装。
    ///
    /// `calc(1px)` → `Value::Number(1, "px")`
    /// `calc(1px + 2px)` → `Value::Number(3, "px")`（同单位简化）
    /// `calc(1px + 2%)` → `Value::Calc("calc(1px + 2%)")`（不同单位保留）
    pub(crate) fn simplify_calc(s: &str) -> Value {
        match Self::try_ast_simplify(s) {
            Some(result) => result,
            None => Self::simplify_calc_str(s),
        }
    }

    /// AST 简化——解析为 CalcNode，简化，返回 Value。
    fn try_ast_simplify(s: &str) -> Option<Value> {
        let inner = if s.len() >= 6
            && s.get(..5).is_some_and(|p| p.eq_ignore_ascii_case("calc("))
            && s.ends_with(')')
        {
            &s[5..s.len() - 1]
        } else {
            s
        };
        let node = super::calc_ast::parse_calc_expr(inner)?;
        let simplified = super::calc_simplify::simplify_calc_node(node).ok()?;
        match simplified {
            super::calc_ast::CalcNode::Number(n, unit) => {
                Some(Value::Number(n, unit))
            }
            other => {
                let s = other.to_string();
                match s.starts_with("calc(") || s.contains('(') {
                    true => Some(Value::Calc(s)),
                    false => Some(Value::Calc(format!("calc({s})"))),
                }
            }
        }
    }

    /// 字符串简化——降级路径。
    fn simplify_calc_str(s: &str) -> Value {
        // 尝试提取 calc(内容) 的内部表达式——大小写不敏感
        let inner = if s.len() >= 6
            && s.get(..5).is_some_and(|p| p.eq_ignore_ascii_case("calc("))
            && s.ends_with(')')
        {
            Some(&s[5..s.len() - 1])
        } else {
            None
        };
        let inner = match inner {
            Some(i) => i.trim(),
            None => {
                match Self::try_simplify_clamp(s) {
                    Some(v) => return v,
                    None => return Value::Calc(s.to_string()),
                }
            }
        };
        // CSS 常量替换：pi/PI/pI → 3.1415926536, e/E → 2.7182818285
        let inner = match inner.to_lowercase().as_str() {
            "pi" => return Value::Number(std::f64::consts::PI, None),
            "e" => return Value::Number(std::f64::consts::E, None),
            _ => inner,
        };
        // 去除多余括号：((1px)) → (1px) → 1px
        let inner = Self::strip_parens(inner);
        match Self::parse_simple_number(inner) {
            Some(v) => return v,
            None => {}
        }
        match Self::try_simplify_same_unit_arith(inner) {
            Some(v) => return v,
            None => {}
        }
        match Self::try_simplify_min_max(inner) {
            Some(v) => return v,
            None => {}
        }
        // 常量替换：在表达式中将 pi/e 替换为数字值
        let substituted = Self::replace_calc_constants(inner);
        match substituted != inner {
            true => {
                match Self::parse_simple_number(&substituted) {
                    Some(v) => return v,
                    None => {}
                }
                match Self::try_simplify_same_unit_arith(&substituted) {
                    Some(v) => return v,
                    None => {}
                }
                return Value::Calc(format!("calc({substituted})"));
            }
            false => {}
        }
        // 去除多余的乘除法括号
        let simplified = Self::remove_unnecessary_parens(inner);
        match simplified != inner {
            true => return Value::Calc(format!("calc({simplified})")),
            false => {}
        }
        Value::Calc(s.to_string())
    }

    /// 尝试简化 clamp()：3 个同单位数字 → 实际计算 clamp 值。
    fn try_simplify_clamp(s: &str) -> Option<Value> {
        let inner = s.strip_prefix("clamp(").and_then(|s| s.strip_suffix(")"))?;
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        match parts.len() != 3 {
            true => return None,
            false => {}
        }
        let min = Self::parse_simple_number(parts[0])?;
        let val = Self::parse_simple_number(parts[1])?;
        let max = Self::parse_simple_number(parts[2])?;
        match (&min, &val, &max) {
            (Value::Number(mn, mu), Value::Number(v, vu), Value::Number(mx, xu))
                if mu == vu && vu == xu =>
            {
                let result = v.clamp(*mn, *mx);
                Some(Value::Number(result, mu.clone()))
            }
            _ => None,
        }
    }

    /// 尝试简化嵌套 min()/max()。只在所有参数都是同单位纯数字时简化。
    fn try_simplify_min_max(s: &str) -> Option<Value> {
        let s = s.trim();
        let (func, inner) = match (s.starts_with("min(") && s.ends_with(')'), s.starts_with("max(") && s.ends_with(')')) {
            (true, _) => ("min", &s[4..s.len() - 1]),
            (_, true) => ("max", &s[4..s.len() - 1]),
            _ => return None,
        };
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        match parts.len() < 2 {
            true => return None,
            false => {}
        }
        let nums: Vec<Value> = parts
            .iter()
            .map(|p| Self::parse_simple_number(p))
            .collect::<Option<Vec<_>>>()?;
        let all_same_unit = nums.windows(2).all(|w| match (&w[0], &w[1]) {
            (Value::Number(_, u1), Value::Number(_, u2)) => u1 == u2,
            _ => false,
        });
        match !all_same_unit {
            true => return None,
            false => {}
        }
        match nums.first() {
            Some(Value::Number(_, unit)) => {
                let init = if func == "min" {
                    f64::INFINITY
                } else {
                    f64::NEG_INFINITY
                };
                let result = nums.iter().try_fold(init, |acc, v| match v {
                    Value::Number(n, _) => Some(if func == "min" {
                        acc.min(*n)
                    } else {
                        acc.max(*n)
                    }),
                    _ => None,
                })?;
                Some(Value::Number(result, unit.clone()))
            }
            _ => None,
        }
    }

    /// 尝试简化同单位加减法：`1px + 2px` → `3px`。
    fn try_simplify_same_unit_arith(s: &str) -> Option<Value> {
        let s = s.trim();
        let op_idx = Self::find_calc_operator(s)?;
        let op_str: &str = s[op_idx..op_idx + 3].trim();
        let left = s[..op_idx].trim();
        let right = s[op_idx + 3..].trim();
        let left_val = Self::parse_simple_number(left)?;
        let right_val = Self::parse_simple_number(right)?;
        match (&left_val, &right_val) {
            // 乘法：数字 * 无单位数字
            (Value::Number(a, ua), Value::Number(b, None)) if op_str == "*" => {
                Some(Value::Number(a * b, ua.clone()))
            }
            (Value::Number(a, None), Value::Number(b, ub)) if op_str == "*" => {
                Some(Value::Number(a * b, ub.clone()))
            }
            // 除法：数字 / 无单位数字
            (Value::Number(a, ua), Value::Number(b, None)) if op_str == "/" && *b != 0.0 => {
                Some(Value::Number(a / b, ua.clone()))
            }
            // 同单位加减法
            (Value::Number(a, ua), Value::Number(b, ub)) if ua == ub => match op_str {
                "+" => Some(Value::Number(a + b, ua.clone())),
                "-" => Some(Value::Number(a - b, ua.clone())),
                _ => None,
            },
            _ => None,
        }
    }

    /// 查找 calc 表达式中的运算符位置（" + ", " - ", " * ", " / "）。
    fn find_calc_operator(s: &str) -> Option<usize> {
        let mut depth = 0i32;
        for (i, c) in s.char_indices() {
            match c {
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                ' ' if depth == 0 => {
                    let rest = &s[i..];
                    match rest.starts_with(" + ")
                        || rest.starts_with(" - ")
                        || rest.starts_with(" * ")
                        || rest.starts_with(" / ")
                    {
                        true => return Some(i),
                        false => {}
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// 去除字符串外层的多余括号：((1px)) → 1px，但 (var(--c)) 保留。
    pub(crate) fn strip_parens(s: &str) -> &str {
        let s = s.trim();
        match s.starts_with('(') && s.ends_with(')') {
            true => {
                let inner = &s[1..s.len() - 1];
                let mut depth = 0i32;
                let mut ok = true;
                for c in inner.chars() {
                    match c {
                        '(' | '[' => depth += 1,
                        ')' | ']' => {
                            depth -= 1;
                            match depth < 0 {
                                true => {
                                    ok = false;
                                    break;
                                }
                                false => {}
                            }
                        }
                        _ => {}
                    }
                }
                match (ok, depth == 0) {
                    (true, true) => {
                        let inner_trimmed = inner.trim();
                        match Self::parse_simple_number(inner_trimmed).is_some() {
                            true => return inner_trimmed,
                            false => {}
                        }
                    }
                    _ => {}
                }
            }
            false => {}
        }
        s
    }

    /// 在 calc 表达式中替换独立常量 pi/e 为数字值。
    fn replace_calc_constants(s: &str) -> String {
        let mut result = s.to_string();
        for (word, val) in [("pi", "3.1415926536"), ("e", "2.7182818285")] {
            let mut idx = 0;
            loop {
                let lower = result[idx..].to_lowercase();
                if let Some(pos_rel) = lower.find(word) {
                    let pos = idx + pos_rel;
                    let before = pos
                        .checked_sub(1)
                        .and_then(|i| result[i..=i].chars().next());
                    let after = result
                        .get(pos + word.len()..pos + word.len() + 1)
                        .and_then(|s| s.chars().next());
                    let is_standalone_before = matches!(before, None | Some(' ' | '(' | '['));
                    let is_standalone_after =
                        matches!(after, None | Some(' ' | ')' | ']' | '+' | '-' | '*' | '/'));
                    match is_standalone_before && is_standalone_after {
                        true => {
                            result = format!("{}{val}{}", &result[..pos], &result[pos + word.len()..]);
                            idx = pos + val.len();
                        }
                        false => idx = pos + word.len(),
                    }
                } else {
                    break;
                }
            }
        }
        result
    }

    /// 去除 calc 表达式中多余的括号。
    fn remove_unnecessary_parens(s: &str) -> String {
        let mut result = s.to_string();
        loop {
            let new = Self::strip_one_unnecessary_paren(&result);
            match new == result {
                true => break,
                false => result = new,
            }
        }
        result
    }

    /// 去除一个多余的乘除法括号。
    fn strip_one_unnecessary_paren(s: &str) -> String {
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < s.len() {
            match bytes[i] == b'(' {
                true => {
                    let mut depth = 1i32;
                    let mut j = i + 1;
                    while j < s.len() && depth > 0 {
                        match bytes[j] {
                            b'(' => depth += 1,
                            b')' => depth -= 1,
                            _ => {}
                        }
                        match depth > 0 {
                            true => j += 1,
                            false => {}
                        }
                    }
                    match (depth == 0, j < s.len()) {
                        (true, true) => {
                            let inner = &s[i + 1..j];
                            match Self::is_simple_mul_div(inner) {
                                true => {
                                    let before = match i > 0 { true => &s[..i], false => "" };
                                    let after = match j + 1 < s.len() { true => &s[j + 1..], false => "" };
                                    let is_in_addsub = before.ends_with(" + ")
                                        || before.ends_with(" - ")
                                        || before.ends_with('(')
                                        || before.is_empty()
                                        || after.starts_with(" + ")
                                        || after.starts_with(" - ")
                                        || after.is_empty();
                                    match is_in_addsub {
                                        true => return format!("{before}{inner}{after}"),
                                        false => {}
                                    }
                                }
                                false => {}
                            }
                        }
                        _ => {}
                    }
                }
                false => {}
            }
            i += 1;
        }
        s.to_string()
    }

    /// 检查字符串是否是简单的乘除法表达式（`A * B` 或 `A / B`）。
    fn is_simple_mul_div(s: &str) -> bool {
        let s = s.trim();
        let mut depth = 0i32;
        let mut found = false;
        for c in s.chars() {
            match c {
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                '*' | '/' if depth == 0 => match found {
                    true => return false,
                    false => found = true,
                },
                _ => {}
            }
        }
        found
    }

    /// 尝试将字符串解析为纯数字（含单位）。
    pub(crate) fn parse_simple_number(s: &str) -> Option<Value> {
        let s = s.trim();
        match s {
            "pi" => return Some(Value::Number(std::f64::consts::PI, None)),
            "e" => return Some(Value::Number(std::f64::consts::E, None)),
            _ => {}
        }
        let s = s.strip_prefix('+').unwrap_or(s);
        let split = s.find(|c: char| {
            !c.is_ascii_digit() && c != '.' && c != '-' && c != 'e' && c != 'E' && c != '+'
        });
        match split {
            None => s.parse::<f64>().ok().map(|n| Value::Number(n, None)),
            Some(idx) if idx > 0 => {
                let (num_str, unit) = s.split_at(idx);
                let n = num_str.parse::<f64>().ok()?;
                let unit = unit.trim();
                match unit.is_empty() {
                    true => return Some(Value::Number(n, None)),
                    false => {}
                }
                match !unit.chars().all(|c| c.is_ascii_alphabetic()) {
                    true => return None,
                    false => {}
                }
                Some(Value::Number(n, Some(unit.to_string())))
            }
            _ => None,
        }
    }
}
