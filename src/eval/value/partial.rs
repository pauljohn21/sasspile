//! @if/@supports 条件的部分求值——保留 CSS 不可求值部分。
//!
//! `partial_eval_condition` 对条件表达式做部分求值：
//! - `sass()` → 正常求值
//! - `css()` / `var()` / `calc()` → 保留为 CSS 透传
//! - `and` / `or` / `not` → 短路 + CSS 拼接

use super::*;
use crate::error::{Result, SassError};
use crate::parse::ast::BinOpKind;

/// 部分条件求值结果。
pub(crate) enum PartialCond {
    True,
    False,
    Css(String),
}

impl Evaluator {
    /// 部分条件求值——保留 CSS 不可求值部分。
    pub(crate) fn partial_eval_condition(condition: &Value, env: &Env) -> Result<PartialCond> {
        match condition {
            // 括号——递归求值内部表达式
            Value::Paren(inner) => {
                match Self::partial_eval_condition(inner, env)? {
                    PartialCond::True => Ok(PartialCond::True),
                    PartialCond::False => Ok(PartialCond::False),
                    PartialCond::Css(s) => {
                        // List 不加括号；其他加括号
                        if matches!(inner.as_ref(), Value::List(_, Separator::Space, _)) {
                            Ok(PartialCond::Css(s))
                        } else {
                            Ok(PartialCond::Css(format!("({s})")))
                        }
                    }
                }
            }
            Value::UnaryOp(UnaryOp::Not, inner) => {
                match Self::partial_eval_condition(inner, env)? {
                    PartialCond::True => Ok(PartialCond::False),
                    PartialCond::False => Ok(PartialCond::True),
                    PartialCond::Css(s) => Ok(PartialCond::Css(format!("not {s}"))),
                }
            }
            Value::BinOp(b) => match b.op {
                BinOpKind::And => match Self::partial_eval_condition(&b.left, env)? {
                    PartialCond::False => Ok(PartialCond::False),
                    PartialCond::True => Self::partial_eval_condition(&b.right, env),
                    PartialCond::Css(left_css) => {
                        match Self::partial_eval_condition(&b.right, env)? {
                            PartialCond::False => Ok(PartialCond::False),
                            PartialCond::True => Ok(PartialCond::Css(left_css)),
                            PartialCond::Css(right_css) => {
                                Ok(PartialCond::Css(format!("{left_css} and {right_css}")))
                            }
                        }
                    }
                },
                BinOpKind::Or => match Self::partial_eval_condition(&b.left, env)? {
                    PartialCond::True => Ok(PartialCond::True),
                    PartialCond::False => Self::partial_eval_condition(&b.right, env),
                    PartialCond::Css(left_css) => {
                        match Self::partial_eval_condition(&b.right, env)? {
                            PartialCond::True => Ok(PartialCond::True),
                            PartialCond::False => Ok(PartialCond::Css(left_css)),
                            PartialCond::Css(right_css) => {
                                Ok(PartialCond::Css(format!("{left_css} or {right_css}")))
                            }
                        }
                    }
                },
                _ => {
                    let val = Self::eval_value(condition, env)?;
                    if Self::is_truthy(&val) {
                        Ok(PartialCond::True)
                    } else {
                        Ok(PartialCond::False)
                    }
                }
            },
            // CSS 原生函数——不可求值
            Value::Calc(_) => Ok(PartialCond::Css(format!("{condition}"))),
            // css() 函数——CSS 透传
            Value::Call(name, _args) if name == "css" => {
                Ok(PartialCond::Css(format!("{condition}")))
            }
            // 嵌套 if() 调用——如果返回 CSS 值则保留原始形式
            Value::Call(name, _args) if name == "if" => {
                let val = Self::eval_value(condition, env)?;
                if let Value::Calc(_) = val {
                    Ok(PartialCond::Css(format!("{condition}")))
                } else if Self::is_truthy(&val) {
                    Ok(PartialCond::True)
                } else {
                    Ok(PartialCond::False)
                }
            }
            // 空格分隔列表作为条件（如 var(--not) css()）
            Value::List(items, Separator::Space, _) => {
                let mut has_css = false;
                let mut css_parts: Vec<String> = Vec::new();
                for item in items {
                    match Self::partial_eval_condition(item, env)? {
                        PartialCond::False => return Ok(PartialCond::False),
                        PartialCond::True => {}
                        PartialCond::Css(s) => {
                            has_css = true;
                            css_parts.push(s);
                        }
                    }
                }
                if has_css {
                    Ok(PartialCond::Css(css_parts.join(" ")))
                } else {
                    Ok(PartialCond::True)
                }
            }
            // sass() 函数——求值参数
            Value::Call(name, _args) if name == "sass" => {
                if env.is_plain_css() {
                    return Err(SassError::Eval(
                        "sass() conditions aren't allowed in plain CSS".into(),
                    ));
                }
                let val = Self::eval_value(condition, env)?;
                if Self::is_truthy(&val) {
                    Ok(PartialCond::True)
                } else {
                    Ok(PartialCond::False)
                }
            }
            // 插值——plain CSS 中不允许
            Value::Interp(segments) => {
                if env.is_plain_css() {
                    return Err(SassError::Eval(
                        "Interpolation isn't allowed in plain CSS.".into(),
                    ));
                }
                let val_str = eval_interp_segments(segments, env);
                if val_str == "and"
                    || val_str == "or"
                    || val_str == "not"
                    || val_str.starts_with("css(")
                    || val_str.starts_with("var(")
                    || val_str.starts_with("attr(")
                    || val_str.starts_with("calc(")
                    || val_str.starts_with("env(")
                    || val_str.starts_with("clamp(")
                {
                    Ok(PartialCond::Css(val_str))
                } else {
                    let val = Value::String(val_str, false);
                    if Self::is_truthy(&val) {
                        Ok(PartialCond::True)
                    } else {
                        Ok(PartialCond::False)
                    }
                }
            }
            // 其他值——正常求值
            _ => {
                let val = Self::eval_value(condition, env)?;
                if let Value::Calc(_) = val {
                    Ok(PartialCond::Css(val.to_string()))
                } else if Self::is_truthy(&val) {
                    Ok(PartialCond::True)
                } else {
                    Ok(PartialCond::False)
                }
            }
        }
    }
}
