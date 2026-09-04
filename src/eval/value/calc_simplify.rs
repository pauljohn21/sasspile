//! calc 表达式简化算法。
//!
//! 设计原则：
//! - `simplify_calc_node` 消费 `CalcNode`（move 语义），返回 `Result<CalcNode, CalcError>`
//! - 子节点简化用 `map` + `collect::<Result<Vec<_>, _>>`（函数式错误传播）
//! - 无 `&mut` 参数，无 `clone()` 满天飞

use super::calc_ast::{CalcError, CalcNode, CalcOp};
use super::calc_units;

/// 消费 calc AST 节点，递归简化后返回新节点。
///
/// - 常量折叠（同单位加减法）
/// - 兼容单位转换（deg+rad 等）
/// - 乘除法规则
/// - 常量替换（pi/e → 数字）
/// - var()/Func 保留
#[tracing::instrument(level = "debug", fields(node = %node))]
pub fn simplify_calc_node(node: CalcNode) -> Result<CalcNode, CalcError> {
    simplify_recursive(node)
}

/// 递归简化——消费 node，返回新节点。
fn simplify_recursive(node: CalcNode) -> Result<CalcNode, CalcError> {
    match node {
        CalcNode::Number(n, unit) => Ok(CalcNode::Number(n, unit)),
        CalcNode::Op { op, left, right } => {
            let left = simplify_recursive(*left)?;
            let right = simplify_recursive(*right)?;
            simplify_op(op, left, right)
        }
        CalcNode::Func { name, args } => {
            let args: Vec<CalcNode> = args
                .into_iter()
                .map(simplify_recursive)
                .collect::<Result<Vec<_>, _>>()?;
            simplify_func(&name, args)
        }
        CalcNode::Var { name, fallback } => {
            let fallback = fallback
                .map(|fb| simplify_recursive(*fb).map(Box::new))
                .transpose()?;
            Ok(CalcNode::Var { name, fallback })
        }
    }
}

/// 简化运算节点——消费 left 和 right（move 语义）。
fn simplify_op(op: CalcOp, left: CalcNode, right: CalcNode) -> Result<CalcNode, CalcError> {
    // 两个都是纯数字
    if let (CalcNode::Number(a, ua), CalcNode::Number(b, ub)) = (&left, &right) {
        return simplify_number_op(op, *a, ua, *b, ub);
    }
    // 无法简化——保留原样
    Ok(CalcNode::Op {
        op,
        left: Box::new(left),
        right: Box::new(right),
    })
}

/// 简化两个数字的运算。
fn simplify_number_op(
    op: CalcOp,
    a: f64,
    ua: &Option<String>,
    b: f64,
    ub: &Option<String>,
) -> Result<CalcNode, CalcError> {
    match op {
        CalcOp::Add | CalcOp::Sub => simplify_add_sub(op, a, ua, b, ub),
        CalcOp::Mul => {
            let unit = ua.clone().or(ub.clone());
            Ok(CalcNode::Number(a * b, unit))
        }
        CalcOp::Div => {
            if b == 0.0 {
                return Err(CalcError::DivisionByZero);
            }
            Ok(CalcNode::Number(a / b, ua.clone()))
        }
    }
}

/// 简化加减法——同单位或兼容单位转换。
fn simplify_add_sub(
    op: CalcOp,
    a: f64,
    ua: &Option<String>,
    b: f64,
    ub: &Option<String>,
) -> Result<CalcNode, CalcError> {
    // 同单位或都无单位
    if ua == ub {
        let result = match op {
            CalcOp::Add => a + b,
            CalcOp::Sub => a - b,
            _ => unreachable!(),
        };
        return Ok(CalcNode::Number(result, ua.clone()));
    }
    // 兼容单位转换
    if let (Some(u1), Some(u2)) = (ua, ub) {
        if calc_units::units_compatible(u1, u2) {
            let converted = calc_units::convert_unit(b, u2, u1)
                .ok_or(CalcError::CannotSimplify)?;
            let result = match op {
                CalcOp::Add => a + converted,
                CalcOp::Sub => a - converted,
                _ => unreachable!(),
            };
            return Ok(CalcNode::Number(result, ua.clone()));
        }
    }
    Err(CalcError::IncompatibleUnits(
        ua.clone().unwrap_or_default(),
        ub.clone().unwrap_or_default(),
    ))
}

/// 简化 CSS 数学函数——消费 args（move 语义）。
fn simplify_func(name: &str, args: Vec<CalcNode>) -> Result<CalcNode, CalcError> {
    match name.to_lowercase().as_str() {
        "min" | "max" => simplify_min_max(name, args),
        "clamp" => simplify_clamp(args),
        "round" | "mod" | "rem" => simplify_round_mod_rem(name, args),
        "abs" => simplify_abs(args),
        "sign" => simplify_sign(args),
        "sqrt" => simplify_unary_math(name, args, f64::sqrt),
        "sin" => simplify_unary_math(name, args, |x: f64| x.to_radians().sin()),
        "cos" => simplify_unary_math(name, args, |x: f64| x.to_radians().cos()),
        "tan" => simplify_unary_math(name, args, |x: f64| x.to_radians().tan()),
        "exp" => simplify_unary_math(name, args, f64::exp),
        "pow" => simplify_pow(args),
        "log" => simplify_log(args),
        _ => Ok(CalcNode::Func { name: name.to_string(), args }),
    }
}

/// 简化 min()/max()——所有参数都是同单位纯数字时计算。
fn simplify_min_max(name: &str, args: Vec<CalcNode>) -> Result<CalcNode, CalcError> {
    let numbers = extract_numbers(&args);
    if numbers.is_empty() || numbers.len() != args.len() {
        return preserve_func(name, args);
    }
    let first_unit = &numbers[0].1;
    let all_same_unit = numbers.iter().all(|(_, u)| u == first_unit);
    if !all_same_unit {
        return preserve_func(name, args);
    }
    let result = match name {
        "min" => numbers.iter().fold(f64::INFINITY, |acc, (v, _)| acc.min(*v)),
        "max" => numbers.iter().fold(f64::NEG_INFINITY, |acc, (v, _)| acc.max(*v)),
        _ => unreachable!(),
    };
    Ok(CalcNode::Number(result, first_unit.clone()))
}

/// 简化 clamp(min, val, max)——同单位时计算。
fn simplify_clamp(args: Vec<CalcNode>) -> Result<CalcNode, CalcError> {
    if args.len() != 3 {
        return preserve_func("clamp", args);
    }
    let nums = extract_numbers(&args);
    if nums.len() != 3 {
        return preserve_func("clamp", args);
    }
    let all_same_unit = nums.windows(2).all(|w| w[0].1 == w[1].1);
    if !all_same_unit {
        return preserve_func("clamp", args);
    }
    let result = nums[1].0.clamp(nums[0].0, nums[2].0);
    Ok(CalcNode::Number(result, nums[0].1.clone()))
}

/// 简化 round/mod/rem。
fn simplify_round_mod_rem(name: &str, args: Vec<CalcNode>) -> Result<CalcNode, CalcError> {
    let nums = extract_numbers(&args);
    if nums.is_empty() || nums.len() != args.len() {
        return preserve_func(name, args);
    }
    match name {
        "round" => simplify_round(&nums, &args),
        "mod" => simplify_mod(&nums, &args),
        "rem" => simplify_rem(&nums, &args),
        _ => unreachable!(),
    }
}

/// 简化 round(step, x) 或 round(x)。
fn simplify_round(nums: &[(f64, Option<String>)], args: &[CalcNode]) -> Result<CalcNode, CalcError> {
    let val = match nums.len() {
        1 => nums[0].0.round(),
        2 if nums[0].0 != 0.0 => (nums[1].0 / nums[0].0).round() * nums[0].0,
        2 => nums[1].0.round(),
        _ => return preserve_func("round", args.to_vec()),
    };
    Ok(CalcNode::Number(val, nums.last().unwrap().1.clone()))
}

/// 简化 mod(a, b)。
fn simplify_mod(nums: &[(f64, Option<String>)], args: &[CalcNode]) -> Result<CalcNode, CalcError> {
    if nums.len() == 2 && nums[1].0 != 0.0 {
        Ok(CalcNode::Number(nums[0].0 % nums[1].0, nums[0].1.clone()))
    } else {
        preserve_func("mod", args.to_vec())
    }
}

/// 简化 rem(a, b)。
fn simplify_rem(nums: &[(f64, Option<String>)], args: &[CalcNode]) -> Result<CalcNode, CalcError> {
    if nums.len() == 2 && nums[1].0 != 0.0 {
        Ok(CalcNode::Number(nums[0].0.rem_euclid(nums[1].0), nums[0].1.clone()))
    } else {
        preserve_func("rem", args.to_vec())
    }
}

/// 简化 abs(x)。
fn simplify_abs(args: Vec<CalcNode>) -> Result<CalcNode, CalcError> {
    match args.as_slice() {
        [CalcNode::Number(v, u)] => Ok(CalcNode::Number(v.abs(), u.clone())),
        _ => preserve_func("abs", args),
    }
}

/// 简化 sign(x)。
fn simplify_sign(args: Vec<CalcNode>) -> Result<CalcNode, CalcError> {
    match args.as_slice() {
        [CalcNode::Number(v, _)] => Ok(CalcNode::Number(v.signum(), None)),
        _ => preserve_func("sign", args),
    }
}

/// 简化一元数学函数（sqrt/exp/sin/cos/tan）。
fn simplify_unary_math(
    name: &str,
    args: Vec<CalcNode>,
    f: impl Fn(f64) -> f64,
) -> Result<CalcNode, CalcError> {
    match args.as_slice() {
        [CalcNode::Number(v, u)] => {
            let result = f(*v);
            let unit = if matches!(name, "sqrt" | "exp") {
                u.clone()
            } else {
                None
            };
            Ok(CalcNode::Number(result, unit))
        }
        _ => preserve_func(name, args),
    }
}

/// 简化 pow(base, exp)。
fn simplify_pow(args: Vec<CalcNode>) -> Result<CalcNode, CalcError> {
    match args.as_slice() {
        [CalcNode::Number(b, _), CalcNode::Number(e, _)] => Ok(CalcNode::Number(b.powf(*e), None)),
        _ => preserve_func("pow", args),
    }
}

/// 简化 log(x) 或 log(x, base)。
fn simplify_log(args: Vec<CalcNode>) -> Result<CalcNode, CalcError> {
    match args.as_slice() {
        [CalcNode::Number(x, _)] => Ok(CalcNode::Number(x.ln(), None)),
        [CalcNode::Number(x, _), CalcNode::Number(base, _)] => {
            Ok(CalcNode::Number(x.log(*base), None))
        }
        _ => preserve_func("log", args),
    }
}

// ─── 辅助函数 ───────────────────────────────────────────────────

/// 从节点切片提取数字——用 `filter_map`（函数式变换）。
fn extract_numbers(args: &[CalcNode]) -> Vec<(f64, Option<String>)> {
    args.iter()
        .filter_map(|n| match n {
            CalcNode::Number(v, u) => Some((*v, u.clone())),
            _ => None,
        })
        .collect()
}

/// 保留函数原样——消费 args（move 语义）。
fn preserve_func(name: &str, args: Vec<CalcNode>) -> Result<CalcNode, CalcError> {
    Ok(CalcNode::Func {
        name: name.to_string(),
        args,
    })
}
