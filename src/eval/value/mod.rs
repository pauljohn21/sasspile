//! Value — SCSS 值类型。

use crate::lex::token::QuoteStyle;

mod display;
mod ops;

pub use display::ColorFormat;

/// 列表分隔符。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Separator {
    Comma,
    Space,
    Slash,
    Undecided,
}

/// 函数引用。
#[derive(Debug, Clone)]
pub struct FunctionRef {
    pub name: String,
    pub is_builtin: bool,
}

/// SCSS 值。
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64, Option<String>),
    String(String, QuoteStyle),
    Ident(String),
    Color(Box<Color>),
    Bool(bool),
    Null,
    List(Vec<Value>, Separator, bool),  // items, separator, brackets
    Map(Vec<(Value, Value)>),
    Function(FunctionRef),
    Variable(String),  // 解析期变量引用
    ArgList(Vec<Value>),
}

/// 颜色值。
#[derive(Debug, Clone)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
    pub format: ColorFormat,
}

impl Value {
    pub fn or(a: Value, b: Value) -> Value {
        if a.is_truthy() { a } else { b }
    }
    pub fn and(a: Value, b: Value) -> Value {
        if !a.is_truthy() { a } else { b }
    }
    pub fn not(v: Value) -> Value {
        Value::Bool(!v.is_truthy())
    }

    pub fn eq(a: Value, b: Value) -> Value {
        Value::Bool(a.equals(&b))
    }
    pub fn ne(a: Value, b: Value) -> Value {
        Value::Bool(!a.equals(&b))
    }

    pub fn gt(a: Value, b: Value) -> Value {
        match (&a, &b) {
            (Value::Number(x, _), Value::Number(y, _)) => Value::Bool(x > y),
            _ => Value::Bool(false),
        }
    }
    pub fn gte(a: Value, b: Value) -> Value {
        match (&a, &b) {
            (Value::Number(x, _), Value::Number(y, _)) => Value::Bool(x >= y),
            _ => Value::Bool(false),
        }
    }
    pub fn lt(a: Value, b: Value) -> Value {
        match (&a, &b) {
            (Value::Number(x, _), Value::Number(y, _)) => Value::Bool(x < y),
            _ => Value::Bool(false),
        }
    }
    pub fn lte(a: Value, b: Value) -> Value {
        match (&a, &b) {
            (Value::Number(x, _), Value::Number(y, _)) => Value::Bool(x <= y),
            _ => Value::Bool(false),
        }
    }

    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Null | Value::Bool(false))
    }

    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(a, _), Value::Number(b, _)) => a == b,
            (Value::String(a, _), Value::String(b, _)) => a == b,
            (Value::Ident(a), Value::Ident(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    pub fn parse_hex_color(hex: &str) -> Value {
        let (r, g, b, a) = match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
                (r, g, b, 1.0)
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                (r, g, b, 1.0)
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
                (r, g, b, a as f64 / 255.0)
            }
            _ => (0, 0, 0, 1.0),
        };
        Value::Color(Box::new(Color {
            r: r as f64, g: g as f64, b: b as f64, a, format: ColorFormat::Auto,
        }))
    }
}
