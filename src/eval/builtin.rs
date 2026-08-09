//! 内建函数——纯函数实现。
//!
//! 签名统一为 `fn(&[Value]) -> Result<Value, SassError>`。

use crate::error::{Result, SassError};
use crate::parse::ast::Value;

// ── 模块声明 ──

mod color;
mod list;
mod map;
mod math;
mod string;

// ── 公共 API ──

/// 分派内建函数调用。
///
/// # 支持的模块
///
/// - `sass:math`: 数学运算（`abs`, `ceil`, `floor`, `sqrt` 等）
/// - `sass:string`: 字符串操作（`length`, `index`, `slice` 等）
/// - `sass:list`: 列表操作（`length`, `nth`, `append` 等）
/// - `sass:map`: Map 操作（`get`, `keys`, `merge` 等）
/// - `sass:color`: 颜色操作（`adjust`, `mix`, `invert` 等）
///
/// # 示例
///
/// ```ignore
/// let args = vec![Value::Number(3.14, None)];
/// let result = builtin::call("math.floor", &args)?; // 3.0
/// ```
pub fn call(name: &str, args: &[Value]) -> Result<Value> {
    match name {
        // sass:math 函数
        "math.abs" => math::abs(args),
        "math.ceil" => math::ceil(args),
        "math.floor" => math::floor(args),
        "math.round" => math::round(args),
        "math.clamp" => math::clamp(args),
        "math.min" => math::min(args),
        "math.max" => math::max(args),
        "math.percentage" => math::percentage(args),
        "math.compatible" => math::compatible(args),
        "math.is-unitless" => math::is_unitless(args),
        "math.sqrt" => math::sqrt(args),
        "math.sin" => math::sin(args),
        "math.cos" => math::cos(args),
        "math.tan" => math::tan(args),
        "math.asin" => math::asin(args),
        "math.acos" => math::acos(args),
        "math.atan" => math::atan(args),
        "math.atan2" => math::atan2(args),
        "math.pow" => math::pow(args),
        "math.log" => math::log(args),
        "math.hypot" => math::hypot(args),
        // sass:string 函数
        "string.length" => string::length(args),
        "string.index" => string::index(args),
        "string.slice" => string::slice(args),
        "string.to-upper-case" => string::to_upper_case(args),
        "string.to-lower-case" => string::to_lower_case(args),
        "string.insert" => string::insert(args),
        "string.unique-id" => string::unique_id(args),
        "string.quote" => string::quote(args),
        "string.unquote" => string::unquote(args),
        // sass:list 函数
        "list.length" => list::length(args),
        "list.nth" => list::nth(args),
        "list.append" => list::append(args),
        "list.join" => list::join(args),
        "list.index" => list::index(args),
        "list.separator" => list::separator(args),
        "list.set-nth" => list::set_nth(args),
        "list.sl-separator" => list::sl_separator(args),
        // sass:map 函数
        "map.get" => map::get(args),
        "map.keys" => map::keys(args),
        "map.values" => map::values(args),
        "map.has-key" => map::has_key(args),
        "map.merge" => map::merge(args),
        "map.remove" => map::remove(args),
        "map.deep-get" => map::deep_get(args),
        "map.deep-merge" => map::deep_merge(args),
        // sass:color 函数
        "color.adjust" => color::adjust(args),
        "color.change" => color::change(args),
        "color.scale" => color::scale(args),
        "color.opacity" => color::opacity(args),
        "color.mix" => color::mix(args),
        "color.invert" => color::invert(args),
        "color.grayscale" => color::grayscale(args),
        "color.lighten" => color::lighten(args),
        "color.darken" => color::darken(args),
        "color.rgba" => color::rgba(args),
        "rgba" => color::rgba(args),
        // 直接函数名（向后兼容）
        "abs" => math::abs(args),
        "ceil" => math::ceil(args),
        "floor" => math::floor(args),
        "round" => math::round(args),
        "percentage" => math::percentage(args),
        "str-length" => string::length(args),
        "str-index" => string::index(args),
        "str-slice" => string::slice(args),
        "to-upper-case" => string::to_upper_case(args),
        "to-lower-case" => string::to_lower_case(args),
        "list-length" => list::length(args),
        "nth" => list::nth(args),
        "append" => list::append(args),
        "join" => list::join(args),
        "map-get" => map::get(args),
        "map-keys" => map::keys(args),
        "map-values" => map::values(args),
        "map-has-key" => map::has_key(args),
        "map-merge" => map::merge(args),
        _ => Err(SassError::EvalError(format!("未知函数: {name}"))),
    }
}

/// 获取 math 常量。
pub fn math_constant(name: &str) -> Option<Value> {
    match name {
        "math.pi" | "pi" => Some(Value::Number(std::f64::consts::PI, None)),
        "math.e" | "e" => Some(Value::Number(std::f64::consts::E, None)),
        "math.epsilon" | "epsilon" => Some(Value::Number(f64::EPSILON, None)),
        "math.max-number" => Some(Value::Number(f64::MAX, None)),
        "math.min-number" => Some(Value::Number(f64::MIN, None)),
        "math.infinity" => Some(Value::Number(f64::INFINITY, None)),
        _ => None,
    }
}

// 辅助函数已移至各子模块

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_math_abs() {
        let result = call("math.abs", &[Value::Number(-10.0, None)]).unwrap();
        assert_eq!(result, Value::Number(10.0, None));
    }

    #[test]
    fn test_dispatch_string_length() {
        let result = call(
            "string.length",
            &[Value::String("hello".to_string(), false)],
        )
        .unwrap();
        assert_eq!(result, Value::Number(5.0, None));
    }

    #[test]
    fn test_dispatch_list_length() {
        let list = Value::List(
            vec![
                Value::Number(1.0, None),
                Value::Number(2.0, None),
                Value::Number(3.0, None),
            ],
            crate::parse::ast::Separator::Comma,
        );
        let result = call("list.length", &[list]).unwrap();
        assert_eq!(result, Value::Number(3.0, None));
    }

    #[test]
    fn test_unknown_function() {
        let result = call("unknown.func", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_math_constant_pi() {
        let val = math_constant("math.pi").unwrap();
        match val {
            Value::Number(n, None) => assert!((n - std::f64::consts::PI).abs() < 1e-10),
            _ => panic!("Expected Number"),
        }
    }
}
