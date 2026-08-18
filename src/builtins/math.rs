//! sass:math built-in module.
//!
//! Implements all math functions: abs, ceil, clamp, floor, max, min, round,
//! div, percentage, unit, unitless, comparable, random, pow, sqrt, log, exp,
//! sin, cos, tan, asin, acos, atan, atan2, hypot, and math.$pi / math.$e.

use crate::ast::Arg;
use crate::env::Env;
use crate::error::{SassError, SourcePos};
use crate::value::{Number, SassString, Value};
use super::helpers::*;

/// Register all math builtins into the environment.
pub fn register(env: &mut Env) {
    let span = tracing::debug_span!("register_math", stage = "init", module = "math");
    let _enter = span.enter();

    env.register_builtin("abs".into(), math_abs);
    env.register_builtin("ceil".into(), math_ceil);
    env.register_builtin("floor".into(), math_floor);
    env.register_builtin("round".into(), math_round);
    env.register_builtin("min".into(), math_min);
    env.register_builtin("max".into(), math_max);
    env.register_builtin("clamp".into(), math_clamp);
    env.register_builtin("math-abs".into(), math_abs);
    env.register_builtin("math-ceil".into(), math_ceil);
    env.register_builtin("math-floor".into(), math_floor);
    env.register_builtin("math-round".into(), math_round);
    env.register_builtin("math-min".into(), math_min);
    env.register_builtin("math-max".into(), math_max);
    env.register_builtin("math-clamp".into(), math_clamp);
    env.register_builtin("math-div".into(), math_div);
    env.register_builtin("math-percentage".into(), math_percentage);
    env.register_builtin("percentage".into(), math_percentage);
    env.register_builtin("math-unit".into(), math_unit);
    env.register_builtin("unit".into(), math_unit);
    env.register_builtin("math-unitless".into(), math_unitless);
    env.register_builtin("unitless".into(), math_unitless);
    env.register_builtin("math-comparable".into(), math_comparable);
    env.register_builtin("comparable".into(), math_comparable);
    env.register_builtin("math-random".into(), math_random);
    env.register_builtin("math-pow".into(), math_pow);
    env.register_builtin("math-sqrt".into(), math_sqrt);
    env.register_builtin("math-log".into(), math_log);
    env.register_builtin("math-exp".into(), math_exp);
    env.register_builtin("math-sin".into(), math_sin);
    env.register_builtin("math-cos".into(), math_cos);
    env.register_builtin("math-tan".into(), math_tan);
    env.register_builtin("math-asin".into(), math_asin);
    env.register_builtin("math-acos".into(), math_acos);
    env.register_builtin("math-atan".into(), math_atan);
    env.register_builtin("math-atan2".into(), math_atan2);
    env.register_builtin("math-hypot".into(), math_hypot);

    // Variables: $pi and $e
    env.set_var("pi".into(), num(std::f64::consts::PI), false, false);
    env.set_var("e".into(), num(std::f64::consts::E), false, false);
}

fn get_arg_values(args: &[Arg], env: &mut Env) -> Result<Vec<Value>, SassError> {
    let parent_sel: Vec<String> = Vec::new();
    eval_args(args, env, &parent_sel)
}

fn math_abs(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let v = get_positional(&vals, 0, "")?;
    if !matches!(v, Value::Number(_)) {
        return Ok(Value::String(SassString::unquoted(format!("abs({})", v))));
    }
    let n = expect_number(v, "abs")?;
    Ok(Value::Number(Number::new(n.value.abs(), n.unit.clone())))
}

fn math_ceil(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let v = get_positional(&vals, 0, "")?;
    if !matches!(v, Value::Number(_)) {
        return Ok(Value::String(SassString::unquoted(format!("ceil({})", v))));
    }
    let n = expect_number(v, "ceil")?;
    Ok(Value::Number(Number::new(n.value.ceil(), n.unit.clone())))
}

fn math_floor(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let v = get_positional(&vals, 0, "")?;
    if !matches!(v, Value::Number(_)) {
        return Ok(Value::String(SassString::unquoted(format!("floor({})", v))));
    }
    let n = expect_number(v, "floor")?;
    Ok(Value::Number(Number::new(n.value.floor(), n.unit.clone())))
}

fn math_round(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let v = get_positional(&vals, 0, "")?;
    if !matches!(v, Value::Number(_)) {
        return Ok(Value::String(SassString::unquoted(format!("round({})", v))));
    }
    let n = expect_number(v, "round")?;
    Ok(Value::Number(Number::new(n.value.round(), n.unit.clone())))
}

fn math_min(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let items = list_args_to_items(&vals);
    if items.is_empty() {
        return Err(SassError::eval("min: at least one argument required", SourcePos::default()));
    }
    let mut result = expect_number(&items[0], "min")?.clone();
    for item in &items[1..] {
        let n = expect_number(item, "min")?;
        if !n.is_compatible_with(&result) {
            // Incompatible units — fall back to CSS min() function
            let parts: Vec<String> = items.iter().map(|v| v.to_string()).collect();
            return Ok(Value::String(crate::value::SassString::unquoted(format!(
                "min({})", parts.join(", ")
            ))));
        }
        if n.value < result.value {
            result = n.clone();
        }
    }
    Ok(Value::Number(result))
}

fn math_max(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let items = list_args_to_items(&vals);
    if items.is_empty() {
        return Err(SassError::eval("max: at least one argument required", SourcePos::default()));
    }
    let mut result = expect_number(&items[0], "max")?.clone();
    for item in &items[1..] {
        let n = expect_number(item, "max")?;
        if !n.is_compatible_with(&result) {
            // Incompatible units — fall back to CSS max() function
            let parts: Vec<String> = items.iter().map(|v| v.to_string()).collect();
            return Ok(Value::String(crate::value::SassString::unquoted(format!(
                "max({})", parts.join(", ")
            ))));
        }
        if n.value > result.value {
            result = n.clone();
        }
    }
    Ok(Value::Number(result))
}

fn math_clamp(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    if vals.len() < 3 {
        return Err(SassError::eval("clamp: expected 3 arguments", SourcePos::default()));
    }
    let lo = expect_number(get_positional(&vals, 0, "")?, "clamp")?;
    let mid = expect_number(&vals[1], "clamp")?;
    let hi = expect_number(&vals[2], "clamp")?;
    if !lo.is_compatible_with(mid) || !mid.is_compatible_with(hi) {
        return Err(SassError::eval(
            "clamp: incompatible units", SourcePos::default(),
        ));
    }
    let v = mid.value.min(hi.value).max(lo.value);
    Ok(Value::Number(Number::new(v, mid.unit.clone())))
}

fn math_div(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("div: expected 2 arguments", SourcePos::default()));
    }
    let a = expect_number(get_positional(&vals, 0, "")?, "div")?;
    let b = expect_number(&vals[1], "div")?;
    if b.value == 0.0 {
        return Err(SassError::eval("division by zero", SourcePos::default()));
    }
    Ok(Value::Number(a.div(b)))
}

fn math_percentage(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let n = expect_number(get_positional(&vals, 0, "")?, "percentage")?;
    if !n.is_unitless() {
        return Err(SassError::eval(
            format!("percentage: expected unitless number, got {}", n.unit_str()),
            SourcePos::default(),
        ));
    }
    Ok(Value::Number(Number::new(n.value * 100.0, Some("%".to_string()))))
}

fn math_unit(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let span = tracing::debug_span!(
        "math_unit",
        stage = "eval",
        module = "math",
        arg0_type = tracing::field::Empty,
    );
    let _enter = span.enter();

    let vals = get_arg_values(args, env)?;
    if !vals.is_empty() {
        tracing::Span::current().record("arg0_type", vals[0].type_name());
        tracing::trace!(stage = "eval", module = "math", arg0 = %vals[0], "unit called");
    }
    let n = expect_number(get_positional(&vals, 0, "")?, "unit")?;
    Ok(quoted_str(n.unit_str()))
}

fn math_unitless(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let n = expect_number(get_positional(&vals, 0, "")?, "unitless")?;
    Ok(Value::Bool(n.is_unitless()))
}

fn math_comparable(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("comparable: expected 2 arguments", SourcePos::default()));
    }
    let a = expect_number(get_positional(&vals, 0, "")?, "comparable")?;
    let b = expect_number(&vals[1], "comparable")?;
    Ok(Value::Bool(a.is_compatible_with(b)))
}

fn math_random(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    if vals.is_empty() {
        let v = simple_random();
        return Ok(num(v));
    }
    let limit = expect_number(get_positional(&vals, 0, "")?, "random")?;
    if limit.value < 1.0 {
        return Err(SassError::eval("random: $limit must be at least 1", SourcePos::default()));
    }
    if !limit.is_unitless() {
        return Err(SassError::eval("random: $limit must be unitless", SourcePos::default()));
    }
    let max = limit.value as u64;
    let r = (simple_random() * max as f64).floor() + 1.0;
    Ok(num(r))
}

fn math_pow(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("pow: expected 2 arguments", SourcePos::default()));
    }
    let base = expect_number(get_positional(&vals, 0, "")?, "pow")?;
    let exp = expect_number(&vals[1], "pow")?;
    if !base.is_unitless() || !exp.is_unitless() {
        return Err(SassError::eval("pow: both arguments must be unitless", SourcePos::default()));
    }
    Ok(num(base.value.powf(exp.value)))
}

fn math_sqrt(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let n = expect_number(get_positional(&vals, 0, "")?, "sqrt")?;
    if !n.is_unitless() {
        return Err(SassError::eval("sqrt: $number must be unitless", SourcePos::default()));
    }
    Ok(num(n.value.sqrt()))
}

fn math_log(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    if vals.len() < 2 {
        let n = expect_number(get_positional(&vals, 0, "")?, "log")?;
        if !n.is_unitless() {
            return Err(SassError::eval("log: $number must be unitless", SourcePos::default()));
        }
        return Ok(num(n.value.ln()));
    }
    let n = expect_number(get_positional(&vals, 0, "")?, "log")?;
    let base = expect_number(&vals[1], "log")?;
    if !n.is_unitless() || !base.is_unitless() {
        return Err(SassError::eval("log: both arguments must be unitless", SourcePos::default()));
    }
    Ok(num(n.value.log(base.value)))
}

fn math_exp(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let n = expect_number(get_positional(&vals, 0, "")?, "exp")?;
    if !n.is_unitless() {
        return Err(SassError::eval("exp: $number must be unitless", SourcePos::default()));
    }
    Ok(num(n.value.exp()))
}

fn math_sin(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let n = expect_number(get_positional(&vals, 0, "")?, "sin")?;
    if !n.is_unitless() && n.unit_str() != "deg" && n.unit_str() != "rad" && n.unit_str() != "turn" && n.unit_str() != "grad" {
        return Err(SassError::eval("sin: $number must have angle unit", SourcePos::default()));
    }
    let radians = to_radians(n);
    Ok(num(radians.sin()))
}

fn math_cos(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let n = expect_number(get_positional(&vals, 0, "")?, "cos")?;
    let radians = to_radians(n);
    Ok(num(radians.cos()))
}

fn math_tan(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let n = expect_number(get_positional(&vals, 0, "")?, "tan")?;
    let radians = to_radians(n);
    Ok(num(radians.tan()))
}

fn math_asin(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let n = expect_number(get_positional(&vals, 0, "")?, "asin")?;
    if !n.is_unitless() {
        return Err(SassError::eval("asin: $number must be unitless", SourcePos::default()));
    }
    Ok(num_unit(n.value.asin() * 180.0 / std::f64::consts::PI, "deg"))
}

fn math_acos(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let n = expect_number(get_positional(&vals, 0, "")?, "acos")?;
    if !n.is_unitless() {
        return Err(SassError::eval("acos: $number must be unitless", SourcePos::default()));
    }
    Ok(num_unit(n.value.acos() * 180.0 / std::f64::consts::PI, "deg"))
}

fn math_atan(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let n = expect_number(get_positional(&vals, 0, "")?, "atan")?;
    if !n.is_unitless() {
        return Err(SassError::eval("atan: $number must be unitless", SourcePos::default()));
    }
    Ok(num_unit(n.value.atan() * 180.0 / std::f64::consts::PI, "deg"))
}

fn math_atan2(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("atan2: expected 2 arguments", SourcePos::default()));
    }
    let y = expect_number(get_positional(&vals, 0, "")?, "atan2")?;
    let x = expect_number(&vals[1], "atan2")?;
    if !y.is_compatible_with(x) {
        return Err(SassError::eval("atan2: incompatible units", SourcePos::default()));
    }
    Ok(num_unit(y.value.atan2(x.value) * 180.0 / std::f64::consts::PI, "deg"))
}

fn math_hypot(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_arg_values(args, env)?;
    let items = list_args_to_items(&vals);
    if items.is_empty() {
        return Ok(num(0.0));
    }
    let first = expect_number(&items[0], "hypot")?;
    let mut sum_sq = first.value * first.value;
    let unit = first.unit.clone();
    for item in &items[1..] {
        let n = expect_number(item, "hypot")?;
        if !n.is_compatible_with(first) {
            return Err(SassError::eval("hypot: incompatible units", SourcePos::default()));
        }
        sum_sq += n.value * n.value;
    }
    Ok(Value::Number(Number::new(sum_sq.sqrt(), unit)))
}

/// Convert a number with possible angle unit to radians.
fn to_radians(n: &Number) -> f64 {
    match n.unit_str() {
        "deg" => n.value.to_radians(),
        "rad" => n.value,
        "turn" => n.value * std::f64::consts::TAU,
        "grad" => n.value * std::f64::consts::PI / 200.0,
        _ => n.value.to_radians(), // treat unitless as degrees (Sass behavior)
    }
}

// Simple pseudo-random number generator (deterministic for testing).
thread_local! {
    static RNG_STATE: std::cell::Cell<u64> = std::cell::Cell::new(0x9E3779B97F4A7C15);
}

fn simple_random() -> f64 {
    RNG_STATE.with(|state| {
        let mut s = state.get();
        // xorshift64
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        state.set(s);
        // Convert to [0, 1) range
        (s >> 11) as f64 / (1u64 << 53) as f64
    })
}

/// Flatten list arguments into individual items.
/// If an arg is a List, expand it; otherwise keep as-is.
fn list_args_to_items(vals: &[Value]) -> Vec<Value> {
    let mut items = Vec::new();
    for v in vals {
        match v {
            Value::List(l) => items.extend(l.items.clone()),
            _ => items.push(v.clone()),
        }
    }
    items
}
