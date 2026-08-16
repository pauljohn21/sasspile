//! Arithmetic, comparison, and logical operations for Value.

#![allow(dead_code)]

use super::{Number, Value, Unit};

/// Result of a binary operation.
pub type OpResult = std::result::Result<Value, super::error::ValueError>;

/// Negate a value.
pub fn negate(val: &Value) -> OpResult {
    match val {
        Value::Number(n) => Ok(Value::Number(Number::new(-n.value, n.unit.clone()))),
        _ => Err(super::error::ValueError::InvalidOperand("negate", val.type_name())),
    }
}

/// Add two values.
pub fn add(lhs: &Value, rhs: &Value) -> OpResult {
    match (lhs, rhs) {
        (Value::Number(a), Value::Number(b)) => {
            if !a.unit.is_compatible(&b.unit) {
                return Err(super::error::ValueError::IncompatibleUnits(
                    format!("{:?}", a.unit),
                    format!("{:?}", b.unit),
                ));
            }
            Ok(Value::Number(Number::new(a.value + b.value, b.unit.clone())))
        }
        (Value::String(a, _), Value::String(b, _)) => {
            Ok(Value::String(format!("{}{}", a, b), super::Quoted::Unquoted))
        }
        _ => Err(super::error::ValueError::InvalidOperands("+", lhs.type_name(), rhs.type_name())),
    }
}

/// Subtract two values.
pub fn subtract(lhs: &Value, rhs: &Value) -> OpResult {
    match (lhs, rhs) {
        (Value::Number(a), Value::Number(b)) => {
            if !a.unit.is_compatible(&b.unit) {
                return Err(super::error::ValueError::IncompatibleUnits(
                    format!("{:?}", a.unit),
                    format!("{:?}", b.unit),
                ));
            }
            Ok(Value::Number(Number::new(a.value - b.value, b.unit.clone())))
        }
        _ => Err(super::error::ValueError::InvalidOperands("-", lhs.type_name(), rhs.type_name())),
    }
}

/// Multiply two values.
pub fn multiply(lhs: &Value, rhs: &Value) -> OpResult {
    match (lhs, rhs) {
        (Value::Number(a), Value::Number(b)) => {
            Ok(Value::Number(Number::new(a.value * b.value, b.unit.clone())))
        }
        _ => Err(super::error::ValueError::InvalidOperands("*", lhs.type_name(), rhs.type_name())),
    }
}

/// Divide two values (Sass division).
pub fn divide(lhs: &Value, rhs: &Value) -> OpResult {
    match (lhs, rhs) {
        (Value::Number(a), Value::Number(b)) => {
            if b.value.abs() < f64::EPSILON {
                return Err(super::error::ValueError::DivisionByZero);
            }
            Ok(Value::Number(Number::new(a.value / b.value, Unit::None)))
        }
        _ => Err(super::error::ValueError::InvalidOperands("/", lhs.type_name(), rhs.type_name())),
    }
}

/// Modulo operation.
pub fn modulo(lhs: &Value, rhs: &Value) -> OpResult {
    match (lhs, rhs) {
        (Value::Number(a), Value::Number(b)) => {
            if b.value.abs() < f64::EPSILON {
                return Err(super::error::ValueError::DivisionByZero);
            }
            Ok(Value::Number(Number::new(a.value % b.value, b.unit.clone())))
        }
        _ => Err(super::error::ValueError::InvalidOperands("%", lhs.type_name(), rhs.type_name())),
    }
}
