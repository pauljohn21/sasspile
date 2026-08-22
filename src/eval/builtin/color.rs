//! color 内建函数——骨架。

use crate::error::{Result, SassError};
use crate::eval::value::Value;
use crate::eval::env::Env;
use crate::parse::ast::Arg;
use crate::eval::eval_value;

pub fn dispatch(field: &str, _args: &[Arg], _env: &Env) -> Result<Value> {
    let _args: Vec<Value> = _args.iter().map(|a| eval_value(&a.value, _env)).collect();
    match field {
        "mix" => match &_args[..] {
            [Value::Color(c1), Value::Color(c2)] => {
                let r = (c1.r + c2.r) / 2.0;
                let g = (c1.g + c2.g) / 2.0;
                let b = (c1.b + c2.b) / 2.0;
                let a = (c1.a + c2.a) / 2.0;
                Ok(Value::Color(Box::new(crate::eval::value::Color {
                    r, g, b, a,
                    format: crate::eval::value::ColorFormat::Auto,
                })))
            }
            [Value::Color(c1), Value::Color(c2), Value::Number(weight, _)] => {
                let w = (*weight / 100.0).clamp(0.0, 1.0);
                let r = c1.r * w + c2.r * (1.0 - w);
                let g = c1.g * w + c2.g * (1.0 - w);
                let b = c1.b * w + c2.b * (1.0 - w);
                let a = c1.a * w + c2.a * (1.0 - w);
                Ok(Value::Color(Box::new(crate::eval::value::Color {
                    r, g, b, a,
                    format: crate::eval::value::ColorFormat::Auto,
                })))
            }
            _ => Err(SassError::eval("mix() expects two colors")),
        },
        "adjust" | "change" | "scale" | "ie_hex_str" | "channel" => {
            // TODO: 实现颜色调整
            match &_args[..] {
                [Value::Color(c)] => Ok(Value::Color(c.clone())),
                _ => Err(SassError::eval(format!("color.{field}() expects a color"))),
            }
        }
        _ => Err(SassError::eval(format!("Unknown color function: {field}"))),
    }
}
