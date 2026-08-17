//! CSS value serialization — converts Sass Values to CSS strings.

use crate::ast::ListSeparator;
use crate::value::{CalcArg, Value};

/// Convert a Value to its CSS string representation.
pub fn value_to_css(val: &Value) -> String {
    match val {
        Value::Number(n) => n.to_css_string(),
        Value::String(s) => {
            if s.quoted {
                format!("\"{}\"", s.value)
            } else {
                s.value.clone()
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        Value::Color(c) => {
            if c.legacy {
                let alpha = c.alpha();
                if (alpha - 1.0).abs() < 1e-10 {
                    c.to_hex()
                } else {
                    let rgb = c.to_rgb();
                    format!("rgba({}, {}, {}, {})",
                        rgb.red() as u8,
                        rgb.green() as u8,
                        rgb.blue() as u8,
                        alpha,
                    )
                }
            } else {
                c.to_string()
            }
        }
        Value::List(l) => {
            let sep = match l.separator {
                ListSeparator::Comma => ", ",
                ListSeparator::Slash => " / ",
                _ => " ",
            };
            let parts: Vec<String> = l.items.iter().map(value_to_css).collect();
            if l.bracketed {
                format!("[{}]", parts.join(sep))
            } else {
                parts.join(sep)
            }
        }
        Value::Map(m) => {
            let parts: Vec<String> = m.entries.iter()
                .map(|(k, v)| format!("{}: {}", value_to_css(k), value_to_css(v)))
                .collect();
            format!("({})", parts.join(", "))
        }
        Value::Calculation(c) => {
            let args: Vec<String> = c.args.iter().map(calc_arg_to_css).collect();
            if c.name == "calc" && c.args.len() == 1 {
                if let CalcArg::Number(n) = &c.args[0] {
                    return n.to_css_string();
                }
            }
            format!("{}({})", c.name, args.join(", "))
        }
        Value::FunctionRef(name) => format!("get-function(\"{}\")", name),
        Value::MixinRef(name) => format!("get-mixin(\"{}\")", name),
    }
}

/// Serialize a calculation argument to CSS.
pub fn calc_arg_to_css(arg: &CalcArg) -> String {
    match arg {
        CalcArg::Number(n) => n.to_css_string(),
        CalcArg::Op(op, l, r) => {
            format!("{} {} {}", calc_arg_to_css(l), op, calc_arg_to_css(r))
        }
        CalcArg::Other(s) => s.clone(),
    }
}
