//! Evaluator — AST 求值，产出 CssNode。

use crate::error::{Result, SassError};
use crate::parse::{Parsed, Node};
use crate::css::CssNode;
use crate::eval::value::Value;
use crate::eval::env::Env;
use std::path::PathBuf;

pub mod value;
pub mod env;
pub mod rule;
pub mod mixin;
pub mod function;
pub mod module;
pub mod control;
pub mod builtin;

/// 求值完成。
pub(crate) struct Evaluated {
    pub nodes: Vec<CssNode>,
}

impl TryFrom<Parsed> for Evaluated {
    type Error = SassError;

    fn try_from(parsed: Parsed) -> Result<Self> {
        let env = Env::root(parsed.base_path, parsed.load_paths);
        let nodes = eval_nodes(&parsed.ast, env)?;
        Ok(Self { nodes })
    }
}

impl Evaluated {
    /// 序列化——Evaluated → Serialized。
    pub fn serialize(self, style: crate::css::OutputStyle) -> crate::css::Serialized {
        crate::css::Serialized::from_nodes(self.nodes, style)
    }
}

/// 求值节点序列。
fn eval_nodes(nodes: &[Node], env: Env) -> Result<Vec<CssNode>> {
    let mut output = Vec::new();
    let mut env = env;
    for node in nodes {
        let (result, new_env) = eval_node(node, env)?;
        env = new_env;
        if let Some(css) = result {
            output.extend(css);
        }
    }
    Ok(output)
}

/// 求值单个节点，返回 (CSS 输出, 新 Env)。
fn eval_node(node: &Node, env: Env) -> Result<(Option<Vec<CssNode>>, Env)> {
    match node {
        Node::Rule { selector, body } => rule::eval_rule(selector, body, env),
        Node::Decl { property, value, important } => {
            let v = eval_value(value, &env);
            let css = CssNode::Declaration {
                property: property.clone(),
                value: v.to_css_string(),
                important: *important,
            };
            Ok((Some(vec![css]), env))
        }
        Node::Comment(s) => {
            Ok((Some(vec![CssNode::Comment(s.clone())]), env))
        }
        Node::Variable { name, value, .. } => {
            let v = eval_value(value, &env);
            Ok((None, env.define_var(name, v)))
        }
        Node::If { branches, else_body } => control::eval_if(branches, else_body.as_deref(), env),
        Node::For { var, from, to, inclusive, body } => control::eval_for(var, from, to, *inclusive, body, env),
        Node::Each { vars, list, body } => control::eval_each(vars, list, body, env),
        Node::While { cond, body } => control::eval_while(cond, body, env),
        Node::MixinDef { name, params, body } => {
            let mixin = env::MixinDef {
                name: name.clone(),
                params: params.clone(),
                body: body.clone(),
            };
            Ok((None, env.define_mixin(mixin)))
        }
        Node::FunctionDef { name, params, body } => {
            let func = env::FunctionDef {
                name: name.clone(),
                params: params.clone(),
                body: body.clone(),
            };
            Ok((None, env.define_function(func)))
        }
        Node::Include { name, args, content } => {
            mixin::exec_include(name, args, content.as_deref(), env)
        }
        Node::Content => {
            // 插入 @content 块
            if let Some(body) = env.get_content() {
                let child_env = env.enter_scope();
                let css = eval_nodes(body, child_env)?;
                Ok((Some(css), env))
            } else {
                Ok((None, env))
            }
        }
        Node::Return(v) => {
            // 函数返回——在 function.rs 中处理
            let val = eval_value(v, &env);
            Ok((Some(vec![CssNode::Return(val)]), env))
        }
        Node::Use { .. } | Node::Forward { .. } | Node::Import { .. } => {
            // 模块系统——骨架阶段跳过
            Ok((None, env))
        }
        Node::Extend { selector, optional } => {
            let current_sel = env.current_selector.clone().unwrap_or_default();
            let env = env.add_extend(
                current_sel,
                selector.clone(),
                *optional,
            );
            Ok((None, env))
        }
        Node::AtRoot { body, .. } => {
            let child_env = env.enter_scope();
            let css = eval_nodes(body, child_env)?;
            Ok((Some(css), env))
        }
        Node::AtRule { name, params, body } => {
            let children = if let Some(b) = body {
                let child_env = env.enter_scope();
                let css = eval_nodes(b, child_env)?;
                css
            } else { Vec::new() };
            Ok((Some(vec![CssNode::AtRule {
                name: name.clone(),
                params: params.clone(),
                children,
                has_body: body.is_some(),
            }]), env))
        }
        Node::Warn(_) | Node::Debug(_) => Ok((None, env)),
        Node::Error(v) => {
            let msg = eval_value(v, &env).to_css_string();
            Err(SassError::eval(msg))
        }
    }
}

/// 求值值表达式——解析变量引用。
pub fn eval_value(value: &Value, env: &Env) -> Value {
    match value {
        Value::Variable(name) => {
            env.get_var(name).cloned().unwrap_or(Value::Null)
        }
        Value::List(items, sep, brackets) => {
            let evaluated: Vec<Value> = items.iter()
                .map(|v| eval_value(v, env))
                .collect();
            Value::List(evaluated, *sep, *brackets)
        }
        Value::Map(pairs) => {
            let evaluated: Vec<(Value, Value)> = pairs.iter()
                .map(|(k, v)| (eval_value(k, env), eval_value(v, env)))
                .collect();
            Value::Map(evaluated)
        }
        _ => value.clone(),
    }
}
