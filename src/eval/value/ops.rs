//! Value 算术运算。

use crate::error::{Result, SassError};
use super::Value;

impl Value {
    pub fn add(a: Value, b: Value) -> Result<Value> {
        match (&a, &b) {
            (Value::Number(x, ux), Value::Number(y, uy)) => {
                let unit = ux.clone().or(uy.clone());
                Ok(Value::Number(x + y, unit))
            }
            (Value::String(s, style), other) => {
                Ok(Value::String(format!("{}{}", s, other.to_css_string()), *style))
            }
            (other, Value::String(s, style)) => {
                Ok(Value::String(format!("{}{}", other.to_css_string(), s), *style))
            }
            (Value::Ident(s), other) => {
                Ok(Value::Ident(format!("{}{}", s, other.to_css_string())))
            }
            (other, Value::Ident(s)) => {
                Ok(Value::Ident(format!("{}{}", other.to_css_string(), s)))
            }
            _ => Err(SassError::eval(format!("Cannot add {:?} and {:?}", a, b))),
        }
    }

    pub fn sub(a: Value, b: Value) -> Result<Value> {
        match (&a, &b) {
            (Value::Number(x, ux), Value::Number(y, uy)) => {
                let unit = ux.clone().or(uy.clone());
                Ok(Value::Number(x - y, unit))
            }
            _ => Err(SassError::eval(format!("Cannot subtract {:?} from {:?}", b, a))),
        }
    }

    pub fn mul(a: Value, b: Value) -> Result<Value> {
        match (&a, &b) {
            (Value::Number(x, ux), Value::Number(y, uy)) => {
                let unit = ux.clone().or(uy.clone());
                Ok(Value::Number(x * y, unit))
            }
            _ => Err(SassError::eval(format!("Cannot multiply {:?} and {:?}", a, b))),
        }
    }

    pub fn div(a: Value, b: Value) -> Result<Value> {
        match (&a, &b) {
            (Value::Number(x, ux), Value::Number(y, uy)) => {
                if *y == 0.0 {
                    // SCSS 中除以零返回字符串
                    return Ok(Value::String(format!("{}/{}", a.to_css_string(), b.to_css_string()), crate::lex::token::QuoteStyle::None));
                }
                let unit = match (ux, uy) {
                    (Some(u1), Some(u2)) if u1 == u2 => None,
                    (Some(u), None) | (None, Some(u)) => Some(u.clone()),
                    _ => None,
                };
                Ok(Value::Number(x / y, unit))
            }
            // 非 Number 除法——返回字符串（CSS fallback）
            _ => Ok(Value::String(
                format!("{}/{}", a.to_css_string(), b.to_css_string()),
                crate::lex::token::QuoteStyle::None,
            )),
        }
    }

    pub fn rem(a: Value, b: Value) -> Result<Value> {
        match (&a, &b) {
            (Value::Number(x, ux), Value::Number(y, uy)) => {
                let unit = ux.clone().or(uy.clone());
                Ok(Value::Number(x % y, unit))
            }
            _ => Err(SassError::eval(format!("Cannot modulo {:?} and {:?}", a, b))),
        }
    }

    pub fn neg(v: Value) -> Result<Value> {
        match v {
            Value::Number(n, u) => Ok(Value::Number(-n, u)),
            _ => Err(SassError::eval("Cannot negate non-number")),
        }
    }
}
