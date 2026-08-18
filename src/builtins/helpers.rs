//! Shared utilities for builtin function implementations.

use crate::ast::Arg;
use crate::env::Env;
use crate::error::{SassError, SourcePos};
use crate::eval::expr::eval_expr;
use crate::value::{SassString, Value};
use crate::value::Number;

/// Evaluate all args and return their values.
pub fn eval_args(args: &[Arg], env: &mut Env, parent_sel: &[String]) -> Result<Vec<Value>, SassError> {
    let span = tracing::trace_span!("eval_args", stage = "builtin", arg_count = args.len());
    let _enter = span.enter();
    let mut vals = Vec::with_capacity(args.len());
    for arg in args {
        vals.push(eval_expr(&arg.value, env, parent_sel)?);
    }
    Ok(vals)
}

/// Check that at least `min` args are present, returning an error otherwise.
pub fn check_min_args(args: &[Value], min: usize, name: &str) -> Result<(), SassError> {
    if args.len() < min {
        return Err(SassError::eval(
            format!("{}() requires at least {} argument{}, got {}", name, min, if min == 1 { "" } else { "s" }, args.len()),
            SourcePos::default(),
        ));
    }
    Ok(())
}

/// Get a positional arg by index, returning an error if missing.
pub fn get_positional<'a>(args: &'a [Value], index: usize, name: &str) -> Result<&'a Value, SassError> {
    args.get(index).ok_or_else(|| {
        SassError::eval(format!("Missing argument #{} for ${}", index + 1, name), SourcePos::default())
    })
}

/// Get a named arg, checking by name first, then falling back to positional index.
pub fn get_named_or_positional(
    args: &[Arg],
    vals: &[Value],
    name: &str,
    index: usize,
    func_name: &str,
) -> Result<Value, SassError> {
    for (i, arg) in args.iter().enumerate() {
        if let Some(n) = &arg.name {
            if n == name {
                return Ok(vals[i].clone());
            }
        }
    }
    vals.get(index).cloned().ok_or_else(|| {
        SassError::eval(
            format!("Missing argument ${} for {}", name, func_name),
            SourcePos::default(),
        )
    })
}

/// Expect a Number value.
pub fn expect_number<'a>(val: &'a Value, name: &str) -> Result<&'a Number, SassError> {
    match val {
        Value::Number(n) => Ok(n),
        _ => Err(SassError::type_err(
            format!("{}: expected number, got {}", name, val.type_name()),
            SourcePos::default(),
        )),
    }
}

/// Expect a String value.
pub fn expect_string<'a>(val: &'a Value, name: &str) -> Result<&'a SassString, SassError> {
    match val {
        Value::String(s) => Ok(s),
        _ => Err(SassError::type_err(
            format!("{}: expected string, got {}", name, val.type_name()),
            SourcePos::default(),
        )),
    }
}

/// Expect a Color value.
pub fn expect_color<'a>(val: &'a Value, name: &str) -> Result<&'a crate::value::Color, SassError> {
    match val {
        Value::Color(c) => Ok(c),
        _ => Err(SassError::type_err(
            format!("{}: expected color, got {}", name, val.type_name()),
            SourcePos::default(),
        )),
    }
}

/// Expect a List value (converts single values to singleton lists).
pub fn expect_list(val: &Value) -> crate::value::SassList {
    match val {
        Value::List(l) => l.clone(),
        _ => crate::value::SassList::new(vec![val.clone()], crate::ast::ListSeparator::Space, false),
    }
}

/// Expect a Map value.
pub fn expect_map<'a>(val: &'a Value, name: &str) -> Result<&'a crate::value::SassMap, SassError> {
    match val {
        Value::Map(m) => Ok(m),
        _ => Err(SassError::type_err(
            format!("{}: expected map, got {}", name, val.type_name()),
            SourcePos::default(),
        )),
    }
}

/// Expect a Color value, or convert a color name string to Color.
pub fn expect_color_or_name<'a>(val: &'a Value, name: &str) -> Result<crate::value::Color, SassError> {
    match val {
        Value::Color(c) => Ok(c.clone()),
        Value::String(s) if !s.quoted => {
            color_from_name(&s.value).ok_or_else(|| {
                SassError::type_err(
                    format!("{}: expected color, got string \"{}\"", name, s.value),
                    SourcePos::default(),
                )
            })
        }
        _ => Err(SassError::type_err(
            format!("{}: expected color, got {}", name, val.type_name()),
            SourcePos::default(),
        )),
    }
}

/// Check if a string is a known CSS color name.
pub fn is_color_name(name: &str) -> bool {
    color_from_name(name).is_some()
}

/// Convert a CSS color name to a Color value.
fn color_from_name(name: &str) -> Option<crate::value::Color> {
    use crate::value::Color;
    let (r, g, b) = match name.to_lowercase().as_str() {
        "red" => (255.0, 0.0, 0.0),
        "green" => (0.0, 128.0, 0.0),
        "blue" => (0.0, 0.0, 255.0),
        "white" => (255.0, 255.0, 255.0),
        "black" => (0.0, 0.0, 0.0),
        "yellow" => (255.0, 255.0, 0.0),
        "orange" => (255.0, 165.0, 0.0),
        "purple" => (128.0, 0.0, 128.0),
        "pink" => (255.0, 192.0, 203.0),
        "cyan" => (0.0, 255.0, 255.0),
        "magenta" => (255.0, 0.0, 255.0),
        "gray" | "grey" => (128.0, 128.0, 128.0),
        "brown" => (165.0, 42.0, 42.0),
        "lime" => (0.0, 255.0, 0.0),
        "navy" => (0.0, 0.0, 128.0),
        "teal" => (0.0, 128.0, 128.0),
        "aqua" => (0.0, 255.0, 255.0),
        "fuchsia" => (255.0, 0.0, 255.0),
        "silver" => (192.0, 192.0, 192.0),
        "maroon" => (128.0, 0.0, 0.0),
        "olive" => (128.0, 128.0, 0.0),
        "transparent" => return Some(Color::rgb(0.0, 0.0, 0.0, 0.0)),
        _ => return None,
    };
    Some(Color::rgb(r, g, b, 1.0))
}

/// Convert a number to a Sass Number value.
pub fn num(value: f64) -> Value {
    Value::Number(Number::unitless(value))
}

/// Convert a number with unit to a Sass Number value.
pub fn num_unit(value: f64, unit: &str) -> Value {
    Value::Number(Number::new(value, Some(unit.to_string())))
}

/// Convert a quoted string to a Sass String value.
pub fn quoted_str(s: &str) -> Value {
    Value::String(SassString::quoted(s))
}

/// Convert an unquoted string to a Sass String value.
pub fn unquoted_str(s: &str) -> Value {
    Value::String(SassString::unquoted(s))
}
