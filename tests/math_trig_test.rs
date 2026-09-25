//! 三角函数 + 高级数学函数测试
//! 纯函数求值验证 (响应式风格, 无 GC, 编译期决定)

use sasspile::eval::{try_eval_builtin, eval_all_calls};

#[test]
fn sin_45_deg() {
    let result = try_eval_builtin("sin(45deg)").expect("sin should evaluate");
    // sin(45deg) = √2/2 ≈ 0.7071...
    let val: f64 = result.parse().expect("should be numeric");
    assert!((val - 0.7071067811865476).abs() < 1e-10, "got {}", val);
}

#[test]
fn cos_60_deg() {
    let result = try_eval_builtin("cos(60deg)").expect("cos should evaluate");
    let val: f64 = result.parse().expect("should be numeric");
    assert!((val - 0.5).abs() < 1e-10, "got {}", val);
}

#[test]
fn tan_45_deg() {
    let result = try_eval_builtin("tan(45deg)").expect("tan should evaluate");
    let val: f64 = result.parse().expect("should be numeric");
    assert!((val - 1.0).abs() < 1e-10, "got {}", val);
}

#[test]
fn asin_half_returns_deg() {
    let result = try_eval_builtin("asin(0.5)").expect("asin should evaluate");
    assert!(result.ends_with("deg"), "should return deg unit, got {}", result);
    let num_part = result.trim_end_matches("deg").parse::<f64>().unwrap();
    assert!((num_part - 30.0).abs() < 1e-10, "asin(0.5) should be 30deg, got {}", result);
}

#[test]
fn sqrt_144() {
    assert_eq!(try_eval_builtin("sqrt(144)"), Some("12".to_string()));
}

#[test]
fn pow_2_10() {
    assert_eq!(try_eval_builtin("pow(2, 10)"), Some("1024".to_string()));
}

#[test]
fn clamp_middle() {
    let result = eval_all_calls("width: clamp(10px, 50px, 100px);");
    assert!(result.contains("50px"), "got: {}", result);
}

#[test]
fn clamp_below_min() {
    let result = eval_all_calls("width: clamp(10px, 5px, 100px);");
    assert!(result.contains("10px"), "clamp should floor to min, got: {}", result);
}

#[test]
fn clamp_above_max() {
    let result = eval_all_calls("width: clamp(10px, 200px, 100px);");
    assert!(result.contains("100px"), "clamp should cap to max, got: {}", result);
}

#[test]
fn sign_positive() {
    assert_eq!(try_eval_builtin("sign(5)"), Some("1".to_string()));
}

#[test]
fn sign_negative() {
    assert_eq!(try_eval_builtin("sign(-3)"), Some("-1".to_string()));
}

#[test]
fn sign_zero() {
    // CSS spec: sign(0) = 0
    assert_eq!(try_eval_builtin("sign(0)"), Some("0".to_string()));
}

#[test]
fn log_natural() {
    let result = try_eval_builtin("log(1)").expect("log should evaluate");
    let val: f64 = result.parse().unwrap();
    assert!(val.abs() < 1e-10, "log(1) should be 0, got {}", val);
}

#[test]
fn exp_zero() {
    let result = try_eval_builtin("exp(0)").expect("exp should evaluate");
    let val: f64 = result.parse().unwrap();
    assert!((val - 1.0).abs() < 1e-10, "exp(0) should be 1, got {}", val);
}

// ═══ 字符串函数测试 ═══

#[test]
fn str_length_quoted() {
    assert_eq!(try_eval_builtin("str-length(\"hello\")"), Some("5".to_string()));
}

#[test]
fn str_length_unquoted() {
    assert_eq!(try_eval_builtin("str-length(abc)"), Some("3".to_string()));
}

#[test]
fn str_index_found() {
    assert_eq!(try_eval_builtin("str-index(hello, ell)"), Some("2".to_string()));
}

#[test]
fn str_index_not_found() {
    assert_eq!(try_eval_builtin("str-index(hello, xyz)"), None);
}

#[test]
fn str_slice_basic() {
    assert_eq!(try_eval_builtin("str-slice(hello, 2, 4)"), Some("ell".to_string()));
}

#[test]
fn str_slice_negative_end() {
    assert_eq!(try_eval_builtin("str-slice(hello, 2, -1)"), Some("ell".to_string()));
}

#[test]
fn to_upper_case() {
    let result = try_eval_builtin("to-upper-case(hello)").expect("should evaluate");
    assert_eq!(result, "HELLO");
}

#[test]
fn to_lower_case() {
    let result = try_eval_builtin("to-lower-case(HELLO)").expect("should evaluate");
    assert_eq!(result, "hello");
}
