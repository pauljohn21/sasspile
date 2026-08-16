//! Operator implementations for expression evaluation.
//!
//! Covers arithmetic, string concatenation, comparison, and logical
//! operators with proper unit-aware semantics following Sass spec.

use crate::eval::error::EvalError;
use crate::parser::{BinaryOp, UnaryOp};
use crate::value::{Number, Value};

/// Apply a binary operator to two values.
pub fn binary(op: &BinaryOp, lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    match op {
        BinaryOp::Add => add(lhs, rhs),
        BinaryOp::Sub => subtract(lhs, rhs),
        BinaryOp::Mul => multiply(lhs, rhs),
        BinaryOp::Div => divide(lhs, rhs),
        BinaryOp::Mod => modulo(lhs, rhs),
        BinaryOp::Eq => equal(lhs, rhs),
        BinaryOp::NotEq => not_equal(lhs, rhs),
        BinaryOp::Greater => greater(lhs, rhs),
        BinaryOp::Less => less(lhs, rhs),
        BinaryOp::GreaterEq => greater_eq(lhs, rhs),
        BinaryOp::LessEq => less_eq(lhs, rhs),
        BinaryOp::And => logical_and(lhs, rhs),
        BinaryOp::Or => logical_or(lhs, rhs),
    }
}

/// Apply a unary operator to a value.
pub fn unary(op: &UnaryOp, val: &Value) -> Result<Value, EvalError> {
    match op {
        UnaryOp::Neg => negate(val),
        UnaryOp::Not => logical_not(val),
    }
}

/// Negate a value (unary minus).
fn negate(val: &Value) -> Result<Value, EvalError> {
    match val {
        Value::Number(n) => Ok(Value::Number(Number::new(-n.value, n.unit.clone()))),
        _ => Err(EvalError::type_error("number", val.type_name())),
    }
}

/// Logical not.
fn logical_not(val: &Value) -> Result<Value, EvalError> {
    Ok(Value::Boolean(!val.to_bool()))
}

/// Addition: number + number (with units), or string concat.
fn add(lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Number(a), Value::Number(b)) => {
            if !a.unit.is_compatible(&b.unit) {
                return Err(EvalError::TypeError(format!(
                    "incompatible units: {:?} and {:?}",
                    a.unit, b.unit
                )));
            }
            Ok(Value::Number(Number::new(a.value + b.value, b.unit.clone())))
        }
        // String concatenation for quoted + quoted.
        (Value::String(a, _), Value::String(b, _)) => {
            Ok(Value::String(format!("{a}{b}"), crate::value::Quoted::Quoted))
        }
        // Color + color (blend).
        (Value::Color(a), Value::Color(b)) => Ok(Value::Color(a.mix(b, 0.5))),
        _ => Err(EvalError::TypeError(format!(
            "cannot add {} and {}",
            lhs.type_name(),
            rhs.type_name()
        ))),
    }
}

/// Subtraction: number - number (with unit checking).
fn subtract(lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Number(a), Value::Number(b)) => {
            if !a.unit.is_compatible(&b.unit) {
                return Err(EvalError::TypeError(format!(
                    "incompatible units: {:?} and {:?}",
                    a.unit, b.unit
                )));
            }
            Ok(Value::Number(Number::new(a.value - b.value, b.unit.clone())))
        }
        _ => Err(EvalError::TypeError(format!(
            "cannot subtract {} from {}",
            rhs.type_name(),
            lhs.type_name()
        ))),
    }
}

/// Multiplication: number * number.
fn multiply(lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Number(a), Value::Number(b)) => {
            Ok(Value::Number(Number::new(a.value * b.value, b.unit.clone())))
        }
        _ => Err(EvalError::TypeError(format!(
            "cannot multiply {} and {}",
            lhs.type_name(),
            rhs.type_name()
        ))),
    }
}

/// Division: number / number (Sass-style, always unitless result).
fn divide(lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Number(a), Value::Number(b)) => {
            if b.value.abs() < f64::EPSILON {
                return Err(EvalError::DivisionByZero);
            }
            Ok(Value::Number(Number::new(a.value / b.value, crate::value::Unit::None)))
        }
        _ => Err(EvalError::TypeError(format!(
            "cannot divide {} by {}",
            lhs.type_name(),
            rhs.type_name()
        ))),
    }
}

/// Modulo: number % number.
fn modulo(lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Number(a), Value::Number(b)) => {
            if b.value.abs() < f64::EPSILON {
                return Err(EvalError::DivisionByZero);
            }
            Ok(Value::Number(Number::new(a.value % b.value, b.unit.clone())))
        }
        _ => Err(EvalError::TypeError(format!(
            "cannot modulo {} by {}",
            lhs.type_name(),
            rhs.type_name()
        ))),
    }
}

/// Equality comparison.
fn equal(lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    Ok(Value::Boolean(lhs == rhs))
}

/// Inequality comparison.
fn not_equal(lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    Ok(Value::Boolean(lhs != rhs))
}

/// Greater than comparison (numbers only).
fn greater(lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a.value > b.value)),
        _ => Err(EvalError::TypeError(format!(
            "cannot compare {} > {}",
            lhs.type_name(),
            rhs.type_name()
        ))),
    }
}

/// Less than comparison (numbers only).
fn less(lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a.value < b.value)),
        _ => Err(EvalError::TypeError(format!(
            "cannot compare {} < {}",
            lhs.type_name(),
            rhs.type_name()
        ))),
    }
}

/// Greater than or equal (numbers only).
fn greater_eq(lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a.value >= b.value)),
        _ => Err(EvalError::TypeError(format!(
            "cannot compare {} >= {}",
            lhs.type_name(),
            rhs.type_name()
        ))),
    }
}

/// Less than or equal (numbers only).
fn less_eq(lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a.value <= b.value)),
        _ => Err(EvalError::TypeError(format!(
            "cannot compare {} <= {}",
            lhs.type_name(),
            rhs.type_name()
        ))),
    }
}

/// Logical and (Sass semantics: returns rhs if lhs is truthy).
fn logical_and(lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    if lhs.to_bool() {
        Ok(rhs.clone())
    } else {
        Ok(lhs.clone())
    }
}

/// Logical or (Sass semantics: returns lhs if truthy, else rhs).
fn logical_or(lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    if lhs.to_bool() {
        Ok(lhs.clone())
    } else {
        Ok(rhs.clone())
    }
}
