//! Type coercion following Sass semantics.

use super::Value;

impl Value {
    /// Convert to boolean following Sass truthiness rules.
    /// In Sass: false, null, 0, empty list, empty string (quoted) are falsy.
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Number(n) => n.value.abs() >= f64::EPSILON,
            Value::String(s, _) => !s.is_empty(),  // empty string is truthy in Sass!
            Value::List(items, _) => !items.is_empty(),
            _ => true,
        }
    }

    /// Convert to Sass string representation.
    pub fn to_string_value(&self) -> String {
        match self {
            Value::String(s, _) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
            Value::Null => String::new(),
            Value::Color(c) => c.to_string(),
            Value::List(items, sep) => {
                let parts: Vec<String> = items.iter().map(|v| v.to_string_value()).collect();
                let delimiter = match sep {
                    super::Separator::Comma => ", ",
                    super::Separator::Space => " ",
                    super::Separator::Slash => " / ",
                    super::Separator::Undecided => " ",
                };
                parts.join(delimiter)
            }
            Value::Map(entries) => {
                let parts: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.to_string_value(), v.to_string_value()))
                    .collect();
                format!("({})", parts.join(", "))
            }
            _ => self.to_css_string(),
        }
    }

    /// Convert to CSS string output.
    pub fn to_css_string(&self) -> String {
        match self {
            Value::Number(n) => format!("{}{}", n, unit_suffix(&n.unit)),
            Value::String(s, q) => match q {
                super::Quoted::Quoted => format!("\"{}\"", s),
                super::Quoted::Unquoted => s.clone(),
            },
            Value::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
            Value::Null => String::new(),
            Value::Color(c) => c.to_string(),
            Value::List(items, sep) => {
                let parts: Vec<String> = items.iter().map(|v| v.to_css_string()).collect();
                let delimiter = match sep {
                    super::Separator::Comma => ", ",
                    super::Separator::Space => " ",
                    super::Separator::Slash => " / ",
                    super::Separator::Undecided => " ",
                };
                parts.join(delimiter)
            }
            other => format!("{:?}", other),
        }
    }

    /// Get type name for error messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(..) => "number",
            Value::String(..) => "string",
            Value::Boolean(..) => "bool",
            Value::Null => "null",
            Value::Color(..) => "color",
            Value::List(..) => "list",
            Value::Map(..) => "map",
            Value::ArgList(..) => "arglist",
            Value::Function(..) => "function",
            Value::Calculation(..) => "calculation",
            Value::Error(..) => "error",
        }
    }
}

fn unit_suffix(unit: &super::Unit) -> &'static str {
    match unit {
        super::Unit::None => "",
        super::Unit::Em => "em",
        super::Unit::Rem => "rem",
        super::Unit::Px => "px",
        super::Unit::Pt => "pt",
        super::Unit::Pc => "pc",
        super::Unit::In => "in",
        super::Unit::Cm => "cm",
        super::Unit::Mm => "mm",
        super::Unit::Q => "q",
        super::Unit::Deg => "deg",
        super::Unit::Rad => "rad",
        super::Unit::Grad => "grad",
        super::Unit::Turn => "turn",
        super::Unit::S => "s",
        super::Unit::Ms => "ms",
        super::Unit::Hz => "hz",
        super::Unit::Khz => "khz",
        super::Unit::Dpi => "dpi",
        super::Unit::Dpcm => "dpcm",
        super::Unit::Dppx => "dppx",
        super::Unit::Percent => "%",
        super::Unit::Compound(units) => {
            if let Some(first) = units.first() {
                unit_suffix(first)
            } else {
                ""
            }
        }
    }
}
