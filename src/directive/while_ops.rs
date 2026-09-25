//! @while 展开 + 条件求值 — 从 ops.rs 拆出
//!
//! 包含: while 循环展开, 简单表达式求值, 变量赋值解析, while 条件比较

use super::state::CompileState;

/// @while 展开: 求值 cond, 若 true 则展开 body, 跳过 $@var 赋值, 直到 false 或 body 耗尽
pub(super) fn expand_while(state: &CompileState, cond: &str, body: &[String]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let max_iter = 100; // 防止无限循环的安全上限
    let mut current_state = state.clone();

    for _ in 0..max_iter {
        // 求值条件
        let cond_substituted = super::parse::substitute_vars(&current_state, cond);
        if !eval_while_condition(&cond_substituted) {
            break;
        }

        // 展开 body: 处理变量赋值和 CSS 行
        for line in body {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }

            // 变量赋值: "$i: $i + 1;" → 更新 state
            if trimmed.starts_with('$') && trimmed.contains(':') {
                if let Some((name, value)) = parse_var_assignment(trimmed) {
                    let evaluated = eval_expr_simple(&current_state, &value);
                    current_state.scope.variables.insert(name, evaluated);
                }
                continue;
            }

            // CSS 行: 替换变量后 emit
            let substituted = super::parse::substitute_vars(&current_state, trimmed);
            result.push(substituted);
        }
    }

    result
}

/// 简单表达式求值: "$i + 1" / "$i - 1" 等
fn eval_expr_simple(state: &CompileState, expr: &str) -> String {
    let substituted = super::parse::substitute_vars(state, expr);
    let t = substituted.trim();

    // 尝试简单算术: "$i + 1" / "5 + 1"
    if let Some(idx) = t.find('+') {
        let (l, r) = t.split_at(idx);
        if let (Ok(a), Ok(b)) = (l.trim().parse::<f64>(), r[1..].trim().parse::<f64>()) {
            let sum = a + b;
            return if sum.fract() == 0.0 { format!("{:.0}", sum) } else { sum.to_string() };
        }
    }
    if let Some(idx) = t.find('-') {
        let (l, r) = t.split_at(idx);
        if let (Ok(a), Ok(b)) = (l.trim().parse::<f64>(), r[1..].trim().parse::<f64>()) {
            let diff = a - b;
            return if diff.fract() == 0.0 { format!("{:.0}", diff) } else { diff.to_string() };
        }
    }
    t.to_string()
}

/// 解析变量赋值: "$i: $i + 1;" → ("$i", "$i + 1")
fn parse_var_assignment(line: &str) -> Option<(String, String)> {
    if !line.starts_with('$') { return None; }
    let (name, value_part) = line[1..].split_once(':')?;
    let name = format!("${}", name.trim());
    let value = value_part.trim().trim_end_matches(';').trim().to_string();
    if value.is_empty() { return None; }
    Some((name, value))
}

/// @while 条件求值
fn eval_while_condition(cond: &str) -> bool {
    let t = cond.trim().replace("#{", "").replace('}', "");

    match t.as_str() {
        "false" | "null" | "0" | "" => return false,
        "true" => return true,
        _ => {}
    }

    // 按优先级尝试操作符 (双字符优先)
    if let Some(r) = try_compare(&t, "<=") { return r; }
    if let Some(r) = try_compare(&t, ">=") { return r; }
    if let Some(r) = try_compare(&t, "==") { return r; }
    if let Some(r) = try_compare(&t, "!=") { return r; }
    if let Some(r) = try_compare(&t, "<") { return r; }
    if let Some(r) = try_compare(&t, ">") { return r; }

    true
}

fn try_compare(t: &str, op: &str) -> Option<bool> {
    let idx = t.find(op)?;
    let (l, r) = t.split_at(idx);
    let r = &r[op.len()..];
    let l_val: f64 = l.trim().parse().ok()?;
    let r_val: f64 = r.trim().parse().ok()?;
    Some(match op {
        "<=" => l_val <= r_val,
        ">=" => l_val >= r_val,
        "<" => l_val < r_val,
        ">" => l_val > r_val,
        "==" => (l_val - r_val).abs() < f64::EPSILON,
        "!=" => (l_val - r_val).abs() > f64::EPSILON,
        _ => return None,
    })
}

/// @if 条件求值 (提取到 while_ops 因为与 while 求值逻辑同源)
pub(super) fn eval_condition(state: &CompileState, expr: &str) -> bool {
    let expr = super::parse::substitute_vars(state, expr);
    let t = expr.trim();
    if t.is_empty() || t == "false" || t == "null" || t == "0" {
        return false;
    }
    if t == "true" {
        return true;
    }
    if let Some(idx) = t.find("==") {
        let (l, r) = t.split_at(idx);
        return l.trim() == r[2..].trim().trim_matches('"');
    }
    if let Some(idx) = t.find("!=") {
        let (l, r) = t.split_at(idx);
        return l.trim() != r[2..].trim().trim_matches('"');
    }
    if let Some(rest) = t.strip_prefix("not ") {
        return !eval_condition(state, rest);
    }
    true
}
