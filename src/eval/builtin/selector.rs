//! selector 内建函数——骨架。

use crate::error::{Result, SassError};
use crate::eval::value::Value;
use crate::eval::env::Env;
use crate::parse::ast::Arg;

pub fn dispatch(field: &str, _args: &[Arg], _env: &Env) -> Result<Value> {
    match field {
        "append" | "extend" | "nest" | "parse" | "replace" | "unify"
        | "is_super_selector" | "simple" => {
            Err(SassError::eval(format!("selector.{field}() not yet implemented")))
        }
        _ => Err(SassError::eval(format!("Unknown selector function: {field}"))),
    }
}
