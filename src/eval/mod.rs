//! 求值器——fold 实现。
//!
//! 使用 `try_fold` 替代 for loop + 可变状态。

pub mod builtin;
pub mod env;

pub use env::Env;

use crate::css::node::CssNode;
use crate::error::{Result, SassError};
use crate::parse::ast::{Ast, Node, Value};

/// 求值器——纯函数风格。
pub struct Evaluator;

impl Evaluator {
    /// 求值 AST —— 入口函数。
    pub fn evaluate(ast: &Ast) -> Result<Vec<CssNode>> {
        ast.nodes
            .iter()
            .try_fold((Vec::new(), Env::new()), |(mut css, env), node| {
                let (mut out, new_env) = Self::eval_node(node, &env)?;
                css.append(&mut out);
                // 更新环境
                let env = if let Some(name) = Self::extract_var_name(node) {
                    let val = Self::eval_value(&Self::extract_var_value(node)?, &env)?;
                    env.bind(name, val)
                } else {
                    new_env
                };
                Ok((css, env))
            })
            .map(|(css, _)| css)
    }

    /// 提取变量名（如果是变量声明）。
    fn extract_var_name(node: &Node) -> Option<String> {
        match node {
            Node::Variable { name, .. } => Some(name.clone()),
            _ => None,
        }
    }

    /// 提取变量值（如果是变量声明）。
    fn extract_var_value(node: &Node) -> Result<Value> {
        match node {
            Node::Variable { value, .. } => Ok(value.clone()),
            _ => Err(SassError::EvalError("不是变量声明".to_string())),
        }
    }

    /// 求值单个节点。
    fn eval_node(node: &Node, env: &Env) -> Result<(Vec<CssNode>, Env)> {
        match node {
            Node::Rule { selector, body } => Self::eval_rule(selector, body, env),
            Node::Decl {
                property,
                value,
                important,
            } => {
                let val = Self::eval_value(value, env)?;
                Ok((
                    vec![CssNode::Declaration {
                        property: property.clone(),
                        value: val.to_string(),
                        important: *important,
                    }],
                    env.clone(),
                ))
            }
            Node::Variable { name, value } => {
                let val = Self::eval_value(value, env)?;
                Ok((vec![], env.bind(name.clone(), val)))
            }
            Node::AtRule { name, params, body } => Self::eval_at_rule(name, params, body, env),
            Node::Comment(text) => Ok((vec![CssNode::Comment(text.clone())], env.clone())),
        }
    }

    /// 求值规则。
    fn eval_rule(selector: &str, body: &[Node], env: &Env) -> Result<(Vec<CssNode>, Env)> {
        let (declarations, children, _): (Vec<CssNode>, Vec<CssNode>, Env) = body.iter().try_fold(
            (Vec::new(), Vec::new(), env.clone()),
            |(mut decls, mut children, env), node| -> Result<_> {
                let (css, new_env) = Self::eval_node(node, &env)?;
                // 分离声明和子规则
                for node in css {
                    match node {
                        CssNode::Declaration { .. } => decls.push(node),
                        other => children.push(other),
                    }
                }
                Ok((decls, children, new_env))
            },
        )?;
        Ok((
            vec![CssNode::Rule {
                selector: selector.to_string(),
                declarations,
                children,
            }],
            env.clone(),
        ))
    }

    /// 求值 @规则。
    fn eval_at_rule(
        name: &str,
        params: &Option<String>,
        body: &Option<Vec<Node>>,
        env: &Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let children = match body {
            Some(nodes) => {
                let (css, _) = Self::evaluate_nodes(nodes, env)?;
                css
            }
            None => Vec::new(),
        };
        Ok((
            vec![CssNode::AtRule {
                name: name.to_string(),
                params: params.clone(),
                children,
            }],
            env.clone(),
        ))
    }

    /// 求值节点列表。
    fn evaluate_nodes(nodes: &[Node], env: &Env) -> Result<(Vec<CssNode>, Env)> {
        nodes
            .iter()
            .try_fold((Vec::new(), env.clone()), |(mut css, env), node| {
                let (mut out, new_env) = Self::eval_node(node, &env)?;
                css.append(&mut out);
                Ok((css, new_env))
            })
    }

    /// 求值表达式。
    fn eval_value(value: &Value, env: &Env) -> Result<Value> {
        match value {
            Value::Number(..) | Value::String(..) | Value::Color(..) | Value::Bool(..) => {
                Ok(value.clone())
            }
            Value::Variable(name) => env
                .lookup(name)
                .cloned()
                .ok_or_else(|| SassError::UndefinedVariable(name.clone())),
            Value::List(elements, sep) => {
                let evaluated = elements
                    .iter()
                    .map(|e| Self::eval_value(e, env))
                    .collect::<Result<Vec<_>>>()?;
                // 尝试计算数学表达式
                if evaluated.len() == 3 {
                    if let (Some(left), Some(op), Some(right)) =
                        (evaluated.first(), evaluated.get(1), evaluated.get(2))
                    {
                        if let (Value::Number(l, u1), Value::String(op, false), Value::Number(r, u2)) =
                            (left, op, right)
                        {
                            return Self::eval_math(*l, op, *r, u1.as_ref().or(u2.as_ref()).cloned());
                        }
                    }
                }
                Ok(Value::List(evaluated, sep.clone()))
            }
            Value::Call(name, args) => {
                let evaluated_args = args
                    .iter()
                    .map(|a| Self::eval_value(a, env))
                    .collect::<Result<Vec<_>>>()?;
                crate::eval::builtin::call(name, &evaluated_args)
            }
        }
    }

    /// 计算简单的二元数学运算。
    fn eval_math(left: f64, op: &str, right: f64, unit: Option<String>) -> Result<Value> {
        let result = match op {
            "*" => left * right,
            "/" => {
                if right == 0.0 {
                    return Err(SassError::EvalError("除零错误".to_string()));
                }
                left / right
            }
            "+" => left + right,
            "-" => left - right,
            "%" => left % right,
            _ => return Err(SassError::EvalError(format!("未知运算符: {op}"))),
        };
        Ok(Value::Number(result, unit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ast::{Ast, Node, Value};

    #[test]
    fn test_eval_empty_ast() {
        let ast = Ast::default();
        let result = Evaluator::evaluate(&ast).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_eval_single_decl() {
        let ast = Ast {
            nodes: vec![Node::Decl {
                property: "color".to_string(),
                value: Value::String("red".to_string(), false),
                important: false,
            }],
        };
        let result = Evaluator::evaluate(&ast).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            CssNode::Declaration {
                property, value, ..
            } => {
                assert_eq!(property, "color");
                assert_eq!(value, "red");
            }
            _ => panic!("期望 Declaration"),
        }
    }

    #[test]
    fn test_eval_variable() {
        let ast = Ast {
            nodes: vec![
                Node::Variable {
                    name: "x".to_string(),
                    value: Value::Number(10.0, Some("px".to_string())),
                },
                Node::Decl {
                    property: "width".to_string(),
                    value: Value::Variable("x".to_string()),
                    important: false,
                },
            ],
        };
        let result = Evaluator::evaluate(&ast).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            CssNode::Declaration {
                property, value, ..
            } => {
                assert_eq!(property, "width");
                assert_eq!(value, "10px");
            }
            _ => panic!("期望 Declaration"),
        }
    }

    #[test]
    fn test_eval_nested_rule() {
        let ast = Ast {
            nodes: vec![Node::Rule {
                selector: ".outer".to_string(),
                body: vec![Node::Rule {
                    selector: ".inner".to_string(),
                    body: vec![Node::Decl {
                        property: "color".to_string(),
                        value: Value::String("red".to_string(), false),
                        important: false,
                    }],
                }],
            }],
        };
        let result = Evaluator::evaluate(&ast).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            CssNode::Rule {
                selector, children, ..
            } => {
                assert_eq!(selector, ".outer");
                assert_eq!(children.len(), 1);
            }
            _ => panic!("期望 Rule"),
        }
    }

    #[test]
    fn test_eval_rgba_call() {
        let ast = Ast {
            nodes: vec![Node::Decl {
                property: "color".to_string(),
                value: Value::Call(
                    "rgba".to_string(),
                    vec![
                        Value::Number(0.0, None),
                        Value::Number(0.0, None),
                        Value::Number(0.0, None),
                        Value::Number(0.55, None),
                    ],
                ),
                important: false,
            }],
        };
        let result = Evaluator::evaluate(&ast).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            CssNode::Declaration { property, value, .. } => {
                assert_eq!(property, "color");
                // 求值后应转为 Color，显示为 rgba(r, g, b, a) 格式
                assert!(value.contains("rgba(0, 0, 0"));
            }
            _ => panic!("期望 Declaration"),
        }
    }

    #[test]
    fn test_eval_comment() {
        let ast = Ast {
            nodes: vec![Node::Comment("test".to_string())],
        };
        let result = Evaluator::evaluate(&ast).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            CssNode::Comment(text) => assert_eq!(text, "test"),
            _ => panic!("期望 Comment"),
        }
    }
}
