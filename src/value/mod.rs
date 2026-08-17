//! Value types for Sass runtime.

use std::fmt;

pub mod color;
pub mod number;

pub use color::{Color, ColorSpace};
pub use number::Number;

/// Runtime value produced by evaluation.
#[derive(Debug, Clone)]
pub enum Value {
    Number(Number),
    String(SassString),
    Color(Color),
    List(SassList),
    Map(SassMap),
    Bool(bool),
    Null,
    Calculation(Calculation),
    FunctionRef(String),
    MixinRef(String),
}

/// A Sass string, either quoted or unquoted.
#[derive(Debug, Clone, PartialEq)]
pub struct SassString {
    pub value: String,
    pub quoted: bool,
}

impl SassString {
    pub fn quoted(value: impl Into<String>) -> Self {
        Self { value: value.into(), quoted: true }
    }

    pub fn unquoted(value: impl Into<String>) -> Self {
        Self { value: value.into(), quoted: false }
    }
}

impl fmt::Display for SassString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// A Sass list with separator and bracketed flag.
#[derive(Debug, Clone, PartialEq)]
pub struct SassList {
    pub items: Vec<Value>,
    pub separator: super::ast::ListSeparator,
    pub bracketed: bool,
}

impl SassList {
    pub fn new(items: Vec<Value>, separator: super::ast::ListSeparator, bracketed: bool) -> Self {
        Self { items, separator, bracketed }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.items.get(index)
    }
}

/// A Sass map (ordered key-value pairs).
#[derive(Debug, Clone, PartialEq)]
pub struct SassMap {
    pub entries: Vec<(Value, Value)>,
}

impl SassMap {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn get(&self, key: &Value) -> Option<&Value> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn has_key(&self, key: &Value) -> bool {
        self.entries.iter().any(|(k, _)| k == key)
    }

    pub fn keys(&self) -> Vec<&Value> {
        self.entries.iter().map(|(k, _)| k).collect()
    }

    pub fn values(&self) -> Vec<&Value> {
        self.entries.iter().map(|(_, v)| v).collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn insert(&mut self, key: Value, value: Value) {
        if let Some(idx) = self.entries.iter().position(|(k, _)| k == &key) {
            self.entries[idx].1 = value;
        } else {
            self.entries.push((key, value));
        }
    }

    pub fn remove(&mut self, key: &Value) -> Option<Value> {
        if let Some(idx) = self.entries.iter().position(|(k, _)| k == key) {
            Some(self.entries.remove(idx).1)
        } else {
            None
        }
    }
}

impl Default for SassMap {
    fn default() -> Self {
        Self::new()
    }
}

/// A CSS calculation expression (calc/min/max/clamp).
#[derive(Debug, Clone)]
pub struct Calculation {
    pub name: String,
    pub args: Vec<CalcArg>,
}

#[derive(Debug, Clone)]
pub enum CalcArg {
    Number(Number),
    Op(String, Box<CalcArg>, Box<CalcArg>),
    Other(String),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Color(_) => "color",
            Value::List(_) => "list",
            Value::Map(_) => "map",
            Value::Bool(_) => "bool",
            Value::Null => "null",
            Value::Calculation(_) => "calculation",
            Value::FunctionRef(_) => "function",
            Value::MixinRef(_) => "mixin",
        }
    }

    pub fn equals(&self, other: &Self) -> bool {
        self == other
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => {
                if s.quoted {
                    write!(f, "\"{}\"", s.value)
                } else {
                    write!(f, "{}", s.value)
                }
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Color(c) => write!(f, "{}", c),
            Value::List(l) => {
                let sep = match l.separator {
                    super::ast::ListSeparator::Comma => ", ",
                    super::ast::ListSeparator::Slash => " / ",
                    _ => " ",
                };
                let parts: Vec<String> = l.items.iter().map(|v| v.to_string()).collect();
                if l.bracketed {
                    write!(f, "[{}]", parts.join(sep))
                } else {
                    write!(f, "{}", parts.join(sep))
                }
            }
            Value::Map(m) => {
                let parts: Vec<String> = m.entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "({})", parts.join(", "))
            }
            Value::Calculation(c) => write!(f, "{}(...)", c.name),
            Value::FunctionRef(name) => write!(f, "get-function(\"{}\")", name),
            Value::MixinRef(name) => write!(f, "get-mixin(\"{}\")", name),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            _ => false,
        }
    }
}
