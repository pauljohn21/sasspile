//! Operator tests — tests arithmetic, string, boolean, comparison, equality operators.

use sasspile::operators::{apply_binop, apply_unaryop};
use sasspile::ast::{BinOp, UnaryOp};
use sasspile::value::{Number, SassString, Value};
use sasspile::error::SourcePos;

fn pos() -> SourcePos { SourcePos::default() }

fn num(v: f64) -> Value { Value::Number(Number::unitless(v)) }
fn num_u(v: f64, u: &str) -> Value { Value::Number(Number::new(v, Some(u.to_string()))) }

#[test]
fn test_add_numbers() {
    let result = apply_binop(&BinOp::Add, &num(1.0), &num(2.0), &pos()).unwrap();
    assert_eq!(result, num(3.0));
}

#[test]
fn test_add_numbers_with_unit() {
    let result = apply_binop(&BinOp::Add, &num_u(10.0, "px"), &num_u(20.0, "px"), &pos()).unwrap();
    assert_eq!(result, num_u(30.0, "px"));
}

#[test]
fn test_sub_numbers() {
    let result = apply_binop(&BinOp::Sub, &num(10.0), &num(3.0), &pos()).unwrap();
    assert_eq!(result, num(7.0));
}

#[test]
fn test_mul_numbers() {
    let result = apply_binop(&BinOp::Mul, &num(3.0), &num(4.0), &pos()).unwrap();
    assert_eq!(result, num(12.0));
}

#[test]
fn test_div_numbers() {
    let result = apply_binop(&BinOp::Div, &num(10.0), &num(2.0), &pos()).unwrap();
    assert_eq!(result, num(5.0));
}

#[test]
fn test_mod_numbers() {
    let result = apply_binop(&BinOp::Mod, &num(10.0), &num(3.0), &pos()).unwrap();
    assert_eq!(result, num(1.0));
}

#[test]
fn test_string_concat() {
    let a = Value::String(SassString::quoted("hello"));
    let b = Value::String(SassString::quoted("world"));
    let result = apply_binop(&BinOp::Add, &a, &b, &pos()).unwrap();
    assert_eq!(result, Value::String(SassString { value: "helloworld".to_string(), quoted: true }));
}

#[test]
fn test_bool_and() {
    let result = apply_binop(&BinOp::And, &Value::Bool(true), &Value::Bool(false), &pos()).unwrap();
    assert_eq!(result, Value::Bool(false));
    let result = apply_binop(&BinOp::And, &Value::Bool(false), &Value::Bool(true), &pos()).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_bool_or() {
    let result = apply_binop(&BinOp::Or, &Value::Bool(true), &Value::Bool(false), &pos()).unwrap();
    assert_eq!(result, Value::Bool(true));
    let result = apply_binop(&BinOp::Or, &Value::Bool(false), &Value::Bool(true), &pos()).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_equality() {
    let result = apply_binop(&BinOp::Eq, &num(1.0), &num(1.0), &pos()).unwrap();
    assert_eq!(result, Value::Bool(true));
    let result = apply_binop(&BinOp::NotEq, &num(1.0), &num(2.0), &pos()).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_comparison() {
    let result = apply_binop(&BinOp::Lt, &num(1.0), &num(2.0), &pos()).unwrap();
    assert_eq!(result, Value::Bool(true));
    let result = apply_binop(&BinOp::Gt, &num(3.0), &num(2.0), &pos()).unwrap();
    assert_eq!(result, Value::Bool(true));
    let result = apply_binop(&BinOp::LtEq, &num(2.0), &num(2.0), &pos()).unwrap();
    assert_eq!(result, Value::Bool(true));
    let result = apply_binop(&BinOp::GtEq, &num(2.0), &num(2.0), &pos()).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_unary_neg() {
    let result = apply_unaryop(&UnaryOp::Neg, &num(5.0), &pos()).unwrap();
    assert_eq!(result, num(-5.0));
}

#[test]
fn test_unary_not() {
    let result = apply_unaryop(&UnaryOp::Not, &Value::Bool(true), &pos()).unwrap();
    assert_eq!(result, Value::Bool(false));
    let result = apply_unaryop(&UnaryOp::Not, &Value::Null, &pos()).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_add_incompatible_units() {
    let result = apply_binop(&BinOp::Add, &num_u(10.0, "px"), &num_u(20.0, "em"), &pos());
    assert!(result.is_err());
}
