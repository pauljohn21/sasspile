//! Value — SCSS 值类型（AST 级别，延迟求值）。

use crate::lex::token::QuoteStyle;
use crate::parse::ast::Arg;

mod display;
mod ops;

pub use display::ColorFormat;

/// 列表分隔符。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Separator {
    Comma,
    Space,
    Slash,
    /// 字面斜杠——声明值中直接写的 `1/2`，输出无空格。
    SlashLiteral,
    Undecided,
}

/// 函数引用。
#[derive(Debug, Clone)]
pub struct FunctionRef {
    pub name: String,
    pub is_builtin: bool,
}

/// 二元运算符类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

/// 一元运算符类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// 二元运算表达式。
#[derive(Debug, Clone)]
pub struct BinOp {
    pub op: BinOpKind,
    pub left: Value,
    pub right: Value,
}

/// SCSS 值——AST 级别，支持延迟求值。
#[derive(Debug, Clone)]
pub enum Value {
    // —— 字面量 ——
    Number(f64, Option<String>),
    String(String, QuoteStyle),
    Ident(String),
    Color(Box<Color>),
    Bool(bool),
    Null,

    // —— 容器 ——
    List(Vec<Value>, Separator, bool),  // items, separator, brackets
    Map(Vec<(Value, Value)>),

    // —— AST 级别（延迟求值）——
    Variable(String),
    /// 函数调用——`name(args)`。解析期保留，求值期分派。
    Call(String, Vec<Arg>),
    /// 插值——`#{...}`。
    Interp(String),
    /// 二元运算——`left op right`。
    BinOp(Box<BinOp>),
    /// 一元运算——`op operand`。
    UnaryOp(UnaryOp, Box<Value>),
    /// calc() 等原生 CSS 函数——原样保留。
    Calc(String),
    /// 括号表达式——保留括号用于 CSS 透传。
    Paren(Box<Value>),

    // —— 求值后类型 ——
    Function(FunctionRef),
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
            (Value::String(a, _), Value::Ident(b)) => a == b,
            (Value::Ident(a), Value::String(b, _)) => a == b,
            (Value::Ident(a), Value::Ident(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Color(a), Value::Color(b)) => {
                a.r as u8 == b.r as u8
                    && a.g as u8 == b.g as u8
                    && a.b as u8 == b.b as u8
                    && (a.a - b.a).abs() < 1e-10
            }
            (Value::List(a, _, _), Value::List(b, _, _)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.equals(y))
            }
            (Value::Map(a), Value::Map(b)) => {
                a.len() == b.len()
                    && a.iter().all(|(k, v)| {
                        b.iter().any(|(k2, v2)| k.equals(k2) && v.equals(v2))
                    })
            }
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
