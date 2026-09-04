//! calc 简化规则测试——对照 sass-spec 预期值。

use sasspile::eval::value::calc_ast::{parse_calc_expr, CalcError, CalcNode};
use sasspile::eval::value::calc_simplify::simplify_calc_node;

fn simplify(input: &str) -> Result<CalcNode, CalcError> {
    let node = parse_calc_expr(input).ok_or(CalcError::CannotSimplify)?;
    simplify_calc_node(node)
}

#[test]
fn test_simplify_add_same_unit() {
    let result = simplify("1px + 2px");
    assert_eq!(result, Ok(CalcNode::Number(3.0, Some("px".to_string()))));
}

#[test]
fn test_simplify_sub_same_unit() {
    let result = simplify("10px - 3px");
    assert_eq!(result, Ok(CalcNode::Number(7.0, Some("px".to_string()))));
}

#[test]
fn test_simplify_mul_no_unit() {
    let result = simplify("2 * 3");
    assert_eq!(result, Ok(CalcNode::Number(6.0, None)));
}

#[test]
fn test_simplify_mul_with_unit() {
    let result = simplify("2 * 3px");
    assert_eq!(result, Ok(CalcNode::Number(6.0, Some("px".to_string()))));
}

#[test]
fn test_simplify_div_no_unit() {
    let result = simplify("6 / 2");
    assert_eq!(result, Ok(CalcNode::Number(3.0, None)));
}

#[test]
fn test_simplify_div_with_unit() {
    let result = simplify("6px / 2");
    assert_eq!(result, Ok(CalcNode::Number(3.0, Some("px".to_string()))));
}

#[test]
fn test_simplify_angle_conversion() {
    // 180deg + pi rad = 360deg
    let result = simplify("180deg + 3.141592653589793rad");
    assert!(result.is_ok());
    if let Ok(CalcNode::Number(v, u)) = result {
        assert_eq!(u, Some("deg".to_string()));
        assert!((v - 360.0).abs() < 1e-6);
    }
}

#[test]
fn test_simplify_min() {
    let result = simplify("min(1px, 2px, 3px)");
    assert_eq!(result, Ok(CalcNode::Number(1.0, Some("px".to_string()))));
}

#[test]
fn test_simplify_max() {
    let result = simplify("max(1px, 2px, 3px)");
    assert_eq!(result, Ok(CalcNode::Number(3.0, Some("px".to_string()))));
}

#[test]
fn test_simplify_clamp() {
    let result = simplify("clamp(1px, 5px, 10px)");
    assert_eq!(result, Ok(CalcNode::Number(5.0, Some("px".to_string()))));
}

#[test]
fn test_simplify_clamp_min() {
    let result = simplify("clamp(5px, 1px, 10px)");
    assert_eq!(result, Ok(CalcNode::Number(5.0, Some("px".to_string()))));
}

#[test]
fn test_simplify_abs() {
    let result = simplify("abs(-5px)");
    assert_eq!(result, Ok(CalcNode::Number(5.0, Some("px".to_string()))));
}

#[test]
fn test_simplify_sqrt() {
    let result = simplify("sqrt(9)");
    assert!(result.is_ok());
    if let Ok(CalcNode::Number(v, _)) = result {
        assert!((v - 3.0).abs() < 1e-6);
    }
}

#[test]
fn test_simplify_round() {
    let result = simplify("round(3.7)");
    assert_eq!(result, Ok(CalcNode::Number(4.0, None)));
}

#[test]
fn test_simplify_nested() {
    let result = simplify("2 * (3px + 4px)");
    assert_eq!(result, Ok(CalcNode::Number(14.0, Some("px".to_string()))));
}

#[test]
fn test_simplify_incompatible_units() {
    let result = simplify("1px + 1deg");
    assert!(result.is_err());
}

#[test]
fn test_simplify_var_preserved() {
    let result = simplify("var(--x) + 1px");
    assert!(result.is_ok());
    // var 保留——不是纯数字
    assert!(!matches!(result, Ok(CalcNode::Number(..))));
}
