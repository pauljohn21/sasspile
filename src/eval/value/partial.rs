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
                // (else) 不允许作为条件
                if let Value::String(s, _) = inner.as_ref()
                    && s == "else" {
                        return Err(SassError::Parse {
                            expected: "(".into(),
                            found: "else".into(),
                        });
                    }
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
                // not not 不允许（不带括号）
                if let Value::UnaryOp(UnaryOp::Not, _) = inner.as_ref() {
                    return Err(SassError::Parse {
                        expected: "(".into(),
                        found: "not".into(),
                    });
                }
                // not else 不允许
                if let Value::String(s, _) = inner.as_ref()
                    && s == "else" {
                        return Err(SassError::Parse {
                            expected: "(".into(),
                            found: "else".into(),
                        });
                    }
                // not 后面不能为空
                if matches!(&**inner, Value::Null) {
                    return Err(SassError::Parse {
                        expected: "identifier".into(),
                        found: ":".into(),
                    });
                }
                match Self::partial_eval_condition(inner, env)? {
                    PartialCond::True => Ok(PartialCond::False),
                    PartialCond::False => Ok(PartialCond::True),
                    PartialCond::Css(s) => Ok(PartialCond::Css(format!("not {s}"))),
                }
            }
            Value::BinOp(b) => match b.op {
                BinOpKind::And => {
                    // and 的 LHS 不允许 or（不带括号的混用）
                    if let Value::BinOp(lb) = &b.left
                        && lb.op == BinOpKind::Or {
                            return Err(SassError::Parse {
                                expected: ":".into(),
                                found: "or".into(),
                            });
                        }
                    // and 后面不允许 or（不带括号的混用）
                    if let Value::BinOp(rb) = &b.right
                        && rb.op == BinOpKind::Or {
                            return Err(SassError::Parse {
                                expected: ":".into(),
                                found: "or".into(),
                            });
                        }
                    // and 后面不允许 not（不带括号）
                    if let Value::UnaryOp(UnaryOp::Not, _) = &b.right {
                        return Err(SassError::Parse {
                            expected: "(".into(),
                            found: "not".into(),
                        });
                    }
                    // and 后面不允许 else
                    if let Value::String(s, _) = &b.right
                        && s == "else" {
                            return Err(SassError::Parse {
                                expected: "(".into(),
                                found: "else".into(),
                            });
                        }
                    // and 后面不能为空
                    if matches!(b.right, Value::Null) {
                        return Err(SassError::Parse {
                            expected: "identifier".into(),
                            found: ":".into(),
                        });
                    }
                    match Self::partial_eval_condition(&b.left, env)? {
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
                    }
                }
                BinOpKind::Or => {
                    // or 的 LHS 不允许 and（不带括号的混用）
                    if let Value::BinOp(lb) = &b.left
                        && lb.op == BinOpKind::And {
                            return Err(SassError::Parse {
                                expected: ":".into(),
                                found: "and".into(),
                            });
                        }
                    // or 后面不允许 and（不带括号的混用）
                    if let Value::BinOp(rb) = &b.right
                        && rb.op == BinOpKind::And {
                            return Err(SassError::Parse {
                                expected: ":".into(),
                                found: "and".into(),
                            });
                        }
                    // or 后面不允许 not（不带括号）
                    if let Value::UnaryOp(UnaryOp::Not, _) = &b.right {
                        return Err(SassError::Parse {
                            expected: "(".into(),
                            found: "not".into(),
                        });
                    }
                    // or 后面不允许 else
                    if let Value::String(s, _) = &b.right
                        && s == "else" {
                            return Err(SassError::Parse {
                                expected: "(".into(),
                                found: "else".into(),
                            });
                        }
                    // or 后面不能为空
                    if matches!(b.right, Value::Null) {
                        return Err(SassError::Parse {
                            expected: "identifier".into(),
                            found: ":".into(),
                        });
                    }
                    match Self::partial_eval_condition(&b.left, env)? {
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
                    }
                }
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
            // 空列表——不允许作为条件
            Value::List(items, _, _) if items.is_empty() => {
                Err(SassError::Parse {
                    expected: "identifier".into(),
                    found: "()".into(),
                })
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

    /// 检查条件中是否有 sass()+CSS 混用。
    /// 只在同一空格列表中同时出现才报错。
    pub(crate) fn check_sass_css_mix(value: &Value) -> Result<()> {
        match value {
            Value::List(items, Separator::Space, _) => {
                let has_sass = items.iter().any(Self::contains_sass_call);
                let has_css = items.iter().any(Self::contains_css_value);
                if has_sass && has_css {
                    return Err(SassError::Eval(
                        "if() conditions with arbitrary substitutions may not contain sass() expressions.".into(),
                    ));
                }
                // 递归检查每个元素
                for item in items {
                    Self::check_sass_css_mix(item)?;
                }
                Ok(())
            }
            Value::BinOp(b) => {
                Self::check_sass_css_mix(&b.left)?;
                Self::check_sass_css_mix(&b.right)?;
                Ok(())
            }
            Value::UnaryOp(_, inner) => Self::check_sass_css_mix(inner),
            Value::Paren(inner) => Self::check_sass_css_mix(inner),
            _ => Ok(()),
        }
    }

    /// 检查条件中是否包含 `sass()` 调用。
    pub(crate) fn contains_sass_call(value: &Value) -> bool {
        match value {
            Value::Call(name, _) if name == "sass" => true,
            Value::Call(_, args) => args.iter().any(|a| {
                Self::contains_sass_call(&a.value)
                    || a.condition.as_ref().is_some_and(Self::contains_sass_call)
            }),
            Value::Paren(inner) => Self::contains_sass_call(inner),
            Value::UnaryOp(_, inner) => Self::contains_sass_call(inner),
            Value::BinOp(b) => Self::contains_sass_call(&b.left) || Self::contains_sass_call(&b.right),
            Value::List(items, _, _) => items.iter().any(Self::contains_sass_call),
            Value::Interp(segments) => segments.iter().any(|s| {
                if let crate::parse::ast::InterpSegment::Expr(e) = s {
                    e.contains("sass(")
                } else {
                    false
                }
            }),
            _ => false,
        }
    }

    /// 检查条件中是否包含 CSS `不可求值部分（var()/css()/calc()` 等）。
    pub(crate) fn contains_css_value(value: &Value) -> bool {
        match value {
            Value::Calc(_) => true,
            Value::Call(name, _) if name == "css" || name == "var" => true,
            Value::Call(name, _) if matches!(name.as_str(), "attr" | "env" | "clamp" | "min" | "max" | "round" | "mod" | "rem") => true,
            Value::Call(_, args) => args.iter().any(|a| {
                Self::contains_css_value(&a.value)
                    || a.condition.as_ref().is_some_and(Self::contains_css_value)
            }),
            Value::Paren(inner) => Self::contains_css_value(inner),
            Value::UnaryOp(_, inner) => Self::contains_css_value(inner),
            Value::BinOp(b) => Self::contains_css_value(&b.left) || Self::contains_css_value(&b.right),
            Value::List(items, _, _) => items.iter().any(Self::contains_css_value),
            Value::Interp(segments) => segments.iter().any(|s| {
                let text = match s {
                    crate::parse::ast::InterpSegment::Expr(e) => e,
                    crate::parse::ast::InterpSegment::Text(t) => t,
                };
                text.contains("var(") || text.contains("css(") || text.contains("calc(")
            }),
            _ => false,
        }
    }
}
