//! Operators — arithmetic, string, boolean, comparison, equality operators.

use crate::ast::BinOp;
use crate::error::{SassError, SourcePos};
use crate::value::{SassString, Value};

/// Apply a binary operator to two values.
pub fn apply_binop(op: &BinOp, left: &Value, right: &Value, pos: &SourcePos) -> Result<Value, SassError> {
    match op {
        BinOp::Add => add(left, right, pos),
        BinOp::Sub => sub(left, right, pos),
        BinOp::Mul => mul(left, right, pos),
        BinOp::Div => div(left, right, pos),
        BinOp::Mod => modulo(left, right, pos),
        BinOp::Eq => Ok(Value::Bool(left == right)),
        BinOp::NotEq => Ok(Value::Bool(left != right)),
        BinOp::Lt => lt(left, right, pos),
        BinOp::LtEq => lte(left, right, pos),
        BinOp::Gt => gt(left, right, pos),
        BinOp::GtEq => gte(left, right, pos),
        BinOp::And => {
            if left.is_truthy() {
                Ok(right.clone())
            } else {
                Ok(left.clone())
            }
        }
        BinOp::Or => {
            if left.is_truthy() {
                Ok(left.clone())
            } else {
                Ok(right.clone())
            }
        }
    }
}

fn add(left: &Value, right: &Value, pos: &SourcePos) -> Result<Value, SassError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            Ok(Value::Number(a.add(b)?))
        }
        (Value::String(a), Value::String(b)) => {
            if a.quoted || b.quoted {
                Ok(Value::String(SassString {
                    value: format!("{}{}", a.value, b.value),
                    quoted: true,
                }))
            } else {
                Ok(Value::String(SassString {
                    value: format!("{}{}", a.value, b.value),
                    quoted: false,
                }))
            }
        }
        (Value::String(a), Value::Number(b)) => {
            Ok(Value::String(SassString {
                value: format!("{}{}", a.value, b),
                quoted: a.quoted,
            }))
        }
        (Value::Number(a), Value::String(b)) => {
            Ok(Value::String(SassString {
                value: format!("{}{}", a, b.value),
                quoted: b.quoted,
            }))
        }
        _ => Err(SassError::type_err(
            format!("undefined operation {} + {}", left.type_name(), right.type_name()),
            pos.clone(),
        )),
    }
}

fn sub(left: &Value, right: &Value, pos: &SourcePos) -> Result<Value, SassError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.sub(b)?)),
        // String - number → CSS string (e.g. "calc(100% - 10px)" parts)
        (Value::String(a), Value::Number(b)) => {
            Ok(Value::String(SassString {
                value: format!("{} - {}", a.value, b),
                quoted: a.quoted,
            }))
        }
        (Value::Number(a), Value::String(b)) => {
            Ok(Value::String(SassString {
                value: format!("{} - {}", a, b.value),
                quoted: b.quoted,
            }))
        }
        (Value::String(a), Value::String(b)) => {
            Ok(Value::String(SassString {
                value: format!("{} - {}", a.value, b.value),
                quoted: a.quoted || b.quoted,
            }))
        }
        _ => Err(SassError::type_err(
            format!("undefined operation {} - {}", left.type_name(), right.type_name()),
            pos.clone(),
        )),
    }
}

fn mul(left: &Value, right: &Value, pos: &SourcePos) -> Result<Value, SassError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.mul(b))),
        // String * number → CSS string
        (Value::String(a), Value::Number(b)) => {
            Ok(Value::String(SassString {
                value: format!("{} * {}", a.value, b),
                quoted: a.quoted,
            }))
        }
        (Value::Number(a), Value::String(b)) => {
            Ok(Value::String(SassString {
                value: format!("{} * {}", a, b.value),
                quoted: b.quoted,
            }))
        }
        _ => Err(SassError::type_err(
            format!("undefined operation {} * {}", left.type_name(), right.type_name()),
            pos.clone(),
        )),
    }
}

fn div(left: &Value, right: &Value, pos: &SourcePos) -> Result<Value, SassError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            if b.value == 0.0 {
                return Err(SassError::eval("division by zero", pos.clone()));
            }
            Ok(Value::Number(a.div(b)))
        }
        // String / number → CSS string (e.g. "var(--x) / 2")
        (Value::String(a), Value::Number(b)) => {
            Ok(Value::String(SassString {
                value: format!("{} / {}", a.value, b),
                quoted: a.quoted,
            }))
        }
        (Value::Number(a), Value::String(b)) => {
            Ok(Value::String(SassString {
                value: format!("{} / {}", a, b.value),
                quoted: b.quoted,
            }))
        }
        (Value::String(a), Value::String(b)) => {
            Ok(Value::String(SassString {
                value: format!("{} / {}", a.value, b.value),
                quoted: a.quoted || b.quoted,
            }))
        }
        _ => Err(SassError::type_err(
            format!("undefined operation {} / {}", left.type_name(), right.type_name()),
            pos.clone(),
        )),
    }
}

fn modulo(left: &Value, right: &Value, pos: &SourcePos) -> Result<Value, SassError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.modulo(b)?)),
        _ => Err(SassError::type_err(
            format!("undefined operation {} % {}", left.type_name(), right.type_name()),
            pos.clone(),
        )),
    }
}

fn lt(left: &Value, right: &Value, _pos: &SourcePos) -> Result<Value, SassError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a.cmp(b) == std::cmp::Ordering::Less)),
        // Non-number comparison in boolean context → false (prevents @while infinite loop)
        _ => Ok(Value::Bool(false)),
    }
}

fn lte(left: &Value, right: &Value, _pos: &SourcePos) -> Result<Value, SassError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            Ok(Value::Bool(a.cmp(b) != std::cmp::Ordering::Greater))
        }
        _ => Ok(Value::Bool(false)),
    }
}

fn gt(left: &Value, right: &Value, _pos: &SourcePos) -> Result<Value, SassError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a.cmp(b) == std::cmp::Ordering::Greater)),
        _ => Ok(Value::Bool(false)),
    }
}

fn gte(left: &Value, right: &Value, _pos: &SourcePos) -> Result<Value, SassError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            Ok(Value::Bool(a.cmp(b) != std::cmp::Ordering::Less))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// Apply a unary operator.
pub fn apply_unaryop(op: &crate::ast::UnaryOp, operand: &Value, pos: &SourcePos) -> Result<Value, SassError> {
    match op {
        crate::ast::UnaryOp::Neg => match operand {
            Value::Number(n) => Ok(Value::Number(n.negate())),
            _ => Err(SassError::type_err(
                format!("cannot negate {}", operand.type_name()),
                pos.clone(),
            )),
        },
        crate::ast::UnaryOp::Not => Ok(Value::Bool(!operand.is_truthy())),
    }
}
