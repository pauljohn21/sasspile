//! calc() 代数运算——单位追踪与抵消。
//!
//! 当 calc($var op expr) 中 $var 是含单位的 calc 表达式时，
//! Dart Sass 会解析单位并执行代数抵消。
//!
//! 规则：
//! - 除法（/）：总是展开并抵消单位
//! - 乘法（*）：仅当右操作数是复合表达式（多个单位因子）时展开
//!   单个单位乘法如 calc($n * 1s) 保持原样

use crate::eval::value::units_compatible;

/// 单位转换因子表——每个单位对应 (类别, 转换因子)。
/// 转换因子表示 1 个该单位 = 多少基准单位。
/// 两个兼容单位的系数比 = (coef1 * factor1) / (coef2 * factor2)。
fn unit_conversion_table() -> std::collections::HashMap<&'static str, (&'static str, f64)> {
    let mut map = std::collections::HashMap::new();
    // 长度单位（基准: px）
    map.insert("px", ("length", 1.0));
    map.insert("in", ("length", 96.0));
    map.insert("cm", ("length", 37.795_275_590_6));
    map.insert("mm", ("length", 3.779_527_559_06));
    map.insert("pt", ("length", 1.333_333_333_33));
    map.insert("pc", ("length", 16.0));
    map.insert("q", ("length", 0.944_881_889_765));
    // 时间单位（基准: ms）
    map.insert("ms", ("time", 1.0));
    map.insert("s", ("time", 1000.0));
    // 频率单位（基准: hz）
    map.insert("hz", ("freq", 1.0));
    map.insert("khz", ("freq", 1000.0));
    // 角度单位（基准: deg）
    map.insert("deg", ("angle", 1.0));
    map.insert("rad", ("angle", 57.295_779_513_1));
    map.insert("grad", ("angle", 0.9));
    map.insert("turn", ("angle", 360.0));
    // 分辨率单位（基准: dpi）
    map.insert("dpi", ("res", 1.0));
    map.insert("dpcm", ("res", 2.54));
    map.insert("dppx", ("res", 96.0));
    map
}

/// 计算兼容单位之间的转换系数。
/// 返回 (from_unit_quantity * factor) = to_unit_quantity 的 factor。
fn unit_conversion_factor(from: &str, to: &str) -> Option<f64> {
    let table = unit_conversion_table();
    match (table.get(from), table.get(to)) {
        (Some((_, f1)), Some((_, f2))) => Some(f1 / f2),
        _ => None,
    }
}

/// 解析 calc(...) 字符串，提取分子和分母的单位列表。
/// 返回 (分子单位列表, 分母单位列表)，每个元素是 (系数, 单位名)。
pub(crate) fn parse_calc_units(s: &str) -> (Vec<(f64, String)>, Vec<(f64, String)>) {
    let inner = s
        .strip_prefix("calc(")
        .and_then(|s| s.strip_suffix(")"))
        .unwrap_or(s);

    let mut numer = Vec::new();
    let mut denom = Vec::new();

    // 按 * 和 / 分割
    let mut tokens = Vec::new();
    let mut ops = Vec::new();
    let mut current = String::new();

    for c in inner.chars() {
        if c == '*' || c == '/' {
            if !current.trim().is_empty() {
                tokens.push(current.trim().to_string());
            }
            ops.push(c);
            current = String::new();
        } else {
            current.push(c);
        }
    }
    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }

    let mut target = &mut numer;
    for (i, tok) in tokens.into_iter().enumerate() {
        if i > 0 && ops.get(i - 1) == Some(&'/') {
            target = &mut denom;
        }
        if let Some((coef, unit)) = parse_factor(&tok) {
            target.push((coef, unit));
        }
    }

    (numer, denom)
}

/// 解析单个因子，如 "1px" → (1.0, "px")，"96px" → (96.0, "px")。
fn parse_factor(s: &str) -> Option<(f64, String)> {
    let s = s.trim();
    if s.starts_with('$') {
        return None;
    }
    // 分离数字前缀和单位后缀
    let num_end = s
        .find(|c: char| c.is_alphabetic() || c == '%')
        .unwrap_or(s.len());
    let (num_str, unit) = s.split_at(num_end);
    let coef: f64 = num_str.parse().ok()?;
    Some((coef, unit.to_string()))
}

/// 尝试简化纯 calc 表达式（无变量）。
/// 处理如 calc(1 / (1 / 1px / 1rad)) → calc(1px * 1rad)
fn simplify_pure_calc(inner: &str) -> Option<String> {
    // 查找模式: X / (Y) 其中 Y 是一个分数表达式
    // 即: numer / denom 其中 denom 包含括号

    // 简化处理: 如果表达式是 "A / (B)" 形式，尝试翻转 B
    if let Some(pos) = inner.find("/ (") {
        let left = inner[..pos].trim();
        let right_start = pos + 3; // skip "/ ("
        if let Some(rest) = inner.get(right_start..) {
            // 找到匹配的右括号
            let mut depth = 1;
            let mut end = 0;
            for (i, c) in rest.char_indices() {
                match c {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if depth == 0 {
                let inner_expr = &rest[..end];
                let after = rest[end + 1..].trim();

                // 解析左侧（分子）
                let lhs_calc = format!("calc({})", left);
                let (mut lhs_numer, mut lhs_denom) = parse_calc_units(&lhs_calc);

                // 解析右侧内部
                let rhs_calc = format!("calc({})", inner_expr);
                let (rhs_numer, rhs_denom) = parse_calc_units(&rhs_calc);

                // 除法：翻转右侧
                // 左分子 + 右分母 → 新分子
                // 左分母 + 右分子 → 新分母
                lhs_numer.extend(rhs_denom);
                lhs_denom.extend(rhs_numer);

                // 如果右侧后面还有内容，需要继续处理
                if !after.is_empty() {
                    // 暂不处理复杂情况
                    return None;
                }

                cancel_units(&mut lhs_numer, &mut lhs_denom);
                return Some(format_result(&lhs_numer, &lhs_denom));
            }
        }
    }
    None
}

/// 尝试简化 calc 表达式中的单位抵消。
/// 输入: calc 字符串，如 "calc($number / (1 / 1ms))"
/// 变量值通过 params 传入（变量名 → calc 字符串）
pub(crate) fn simplify_calc_with_vars(
    expr: &str,
    var_values: &std::collections::HashMap<String, String>,
) -> Option<String> {
    // 去掉 calc(...) 外壳，获取内部表达式
    let inner = expr
        .strip_prefix("calc(")
        .and_then(|s| s.strip_suffix(")"))
        .unwrap_or(expr);

    // 查找变量引用
    for (var_name, var_calc) in var_values {
        if !inner.contains(var_name) {
            continue;
        }

        // 确定运算符位置
        let after_var = inner[var_name.len()..].trim_start();
        let (op, rest) = if after_var.starts_with('/') {
            ("/", after_var[1..].trim())
        } else if after_var.starts_with('*') {
            ("*", after_var[1..].trim())
        } else {
            continue;
        };

        // 解析变量的单位
        let (var_numer, var_denom) = parse_calc_units(var_calc);

        // 解析右侧表达式（去掉外层括号）
        let rest_clean = rest
            .trim_start_matches('(')
            .trim_end_matches(')')
            .trim();
        let rhs_calc = format!("calc({rest_clean})");
        let (rhs_numer, rhs_denom) = parse_calc_units(&rhs_calc);

        let result = match op {
            "/" => {
                // 除法：右操作数的分子变分母，分母变分子
                let mut numer = var_numer.clone();
                let mut denom = var_denom.clone();
                // 除数分子 → 分母
                denom.extend(rhs_numer);
                // 除数分母 → 分子
                numer.extend(rhs_denom);
                cancel_units(&mut numer, &mut denom);
                format_result(&numer, &denom)
            }
            "*" => {
                // 乘法：总是展开并尝试抵消
                let mut numer = var_numer.clone();
                let mut denom = var_denom.clone();
                numer.extend(rhs_numer);
                denom.extend(rhs_denom);
                cancel_units(&mut numer, &mut denom);
                format_result(&numer, &denom)
            }
            _ => continue,
        };

        return Some(result);
    }

    // 如果没有变量匹配，尝试纯 calc 简化
    simplify_pure_calc(inner)
}

/// 抵消分子分母中的兼容单位。
fn cancel_units(numer: &mut Vec<(f64, String)>, denom: &mut Vec<(f64, String)>) {
    let mut i = 0;
    while i < numer.len() {
        let (coef_n, ref unit_n) = numer[i];
        if unit_n.is_empty() {
            i += 1;
            continue;
        }
        let mut matched = false;
        for j in 0..denom.len() {
            let (coef_d, ref unit_d) = denom[j];
            if unit_d.is_empty() {
                continue;
            }
            if units_compatible(Some(unit_n), Some(unit_d)) {
                // 计算转换后的系数比
                // 对于兼容单位，实际比值 = (coef_n * factor_n) / (coef_d * factor_d)
                let factor = unit_conversion_factor(unit_n, unit_d).unwrap_or(1.0);
                let effective_n = coef_n * factor;
                let effective_d = coef_d;
                let new_coef = effective_n / effective_d;

                // 移除两个因子
                numer.remove(i);
                denom.remove(j);

                // 如果转换后系数不为 1，保留为纯数字系数
                if (new_coef - 1.0).abs() > 1e-10 {
                    numer.push((new_coef, String::new()));
                }
                matched = true;
                break;
            }
        }
        if !matched {
            i += 1;
        }
    }
}

/// 格式化结果为字符串。
fn format_result(numer: &[(f64, String)], denom: &[(f64, String)]) -> String {
    // 计算纯数字系数
    let numer_coef: f64 = numer.iter().filter(|(_, u)| u.is_empty()).map(|(c, _)| c).product();
    let denom_coef: f64 = denom.iter().filter(|(_, u)| u.is_empty()).map(|(c, _)| c).product();

    // 收集带单位的因子
    let numer_units: Vec<(f64, &str)> = numer
        .iter()
        .filter(|(_, u)| !u.is_empty())
        .map(|(c, u)| (*c, u.as_str()))
        .collect();
    let denom_units: Vec<(f64, &str)> = denom
        .iter()
        .filter(|(_, u)| !u.is_empty())
        .map(|(c, u)| (*c, u.as_str()))
        .collect();

    // 总系数
    let total_coef = numer_coef / denom_coef;

    // 构建分子字符串
    // 系数合并到第一个单位因子中（如 1000 * 1px → 1000px）
    let numer_str = if numer_units.is_empty() {
        if (total_coef - 1.0).abs() < 1e-10 {
            "1".to_string()
        } else {
            format_coef(total_coef)
        }
    } else {
        let mut parts: Vec<String> = numer_units
            .iter()
            .map(|(c, u)| format_factor(*c, u))
            .collect();
        // 将总系数合并到第一个因子
        if (total_coef - 1.0).abs() > 1e-10 && !parts.is_empty() {
            // 第一个因子必然是 "1unit" 格式，合并系数
            if let Some(first) = parts.first_mut() {
                *first = format_factor(total_coef, &numer_units[0].1);
            }
        }
        parts.join(" * ")
    };

    if denom_units.is_empty() {
        // 没有分母
        numer_str
    } else {
        // 有分母
        let denom_str = denom_units
            .iter()
            .map(|(c, u)| format_factor(*c, u))
            .collect::<Vec<_>>()
            .join(" / ");
        if numer_units.is_empty() && (total_coef - 1.0).abs() < 1e-10 {
            format!("1 / {}", denom_str)
        } else {
            format!("{} / {}", numer_str, denom_str)
        }
    }
}

/// 格式化系数。
fn format_coef(c: f64) -> String {
    if c.fract().abs() < 1e-10 {
        format!("{}", c as i64)
    } else {
        format!("{}", c)
    }
}

/// 格式化因子。
fn format_factor(coef: f64, unit: &str) -> String {
    if (coef - 1.0).abs() < 1e-10 {
        format!("1{}", unit)
    } else if coef.fract().abs() < 1e-10 {
        format!("{}{}", coef as i64, unit)
    } else {
        format!("{}{}", coef, unit)
    }
}
