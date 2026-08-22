//! Evaluator — AST 求值，产出 CssNode。

use crate::error::{Result, SassError};
use crate::parse::{Parsed, Node};
use crate::css::CssNode;
use crate::eval::value::Value;
use crate::eval::env::Env;

pub mod value;
pub mod env;
pub mod rule;
pub mod mixin;
pub mod function;
pub mod module;
pub mod control;
pub mod builtin;
pub mod file_resolver;
pub mod extend;
pub mod plain_css;

/// 递归深度上限。
const MAX_DEPTH: usize = 200;

/// 求值完成。
pub(crate) struct Evaluated {
    pub nodes: Vec<CssNode>,
}

impl TryFrom<Parsed> for Evaluated {
    type Error = SassError;

    fn try_from(parsed: Parsed) -> Result<Self> {
        let env = Env::root(parsed.base_path, parsed.load_paths);
        let (mut nodes, final_env) = eval_nodes(&parsed.ast, env)?;

        // @extend 后处理
        let extends: Vec<(String, String, bool)> = final_env.get_extends().to_vec();
        if !extends.is_empty() {
            extend::apply_extends(&mut nodes, &extends);
            extend::check_extend_targets(&nodes, &extends)?;
        }

        // CSS @import 提升到顶部
        plain_css::hoist_css_imports(&mut nodes);

        Ok(Self { nodes })
    }
}

impl Evaluated {
    /// 序列化——Evaluated → Serialized。
    pub fn serialize(self, style: crate::css::OutputStyle) -> crate::css::Serialized {
        crate::css::Serialized::from_nodes(self.nodes, style)
    }
}

/// 求值节点序列——返回 (CSS 输出, 最终 Env)。
pub(crate) fn eval_nodes(nodes: &[Node], env: Env) -> Result<(Vec<CssNode>, Env)> {
    if env.depth > MAX_DEPTH {
        return Err(SassError::eval("Recursion depth limit exceeded"));
    }
    let mut output = Vec::new();
    let mut env = env;
    for node in nodes {
        let (result, new_env) = eval_node(node, env)?;
        env = new_env;
        if let Some(css) = result {
            output.extend(css);
        }
    }
    Ok((output, env))
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
        Node::Variable { name, value, flags } => {
            let v = eval_value(value, &env);
            // !default — 仅未定义时赋值
            if flags.default && env.get_var(name).is_some() {
                return Ok((None, env));
            }
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
                let css = eval_nodes(body, child_env)?.0;
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
        Node::Use { url, namespace, star, config } => {
            module::eval_use(url, namespace, *star, config, env)
        }
        Node::Forward { url, show, hide, prefix, config } => {
            module::eval_forward(url, prefix, config, env, show, hide)
        }
        Node::Import { url, modifier } => {
            module::eval_import(url, modifier, env)
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
            let css = eval_nodes(body, child_env)?.0;
            // AtRoot 输出为 AtRoot 节点——序列化器提升到顶层
            Ok((Some(vec![CssNode::AtRoot(css)]), env))
        }
        Node::AtRule { name, params, body } => {
            let params_str = params.clone().unwrap_or_default();
            let children = if let Some(b) = body {
            let child_env = env.enter_scope();
            eval_nodes(b, child_env)?.0
            } else {
                Vec::new()
            };

            // @media/@supports/@container 在规则内部时，提升声明到父选择器
            if matches!(name.as_str(), "media" | "supports" | "container")
                && env.current_selector.is_some()
            {
                let sel = env.current_selector.as_ref().unwrap().clone();
                let mut new_children = Vec::new();
                let mut current_decls = Vec::new();
                for child in children {
                    match &child {
                        CssNode::Declaration { .. } | CssNode::Comment(_) => {
                            current_decls.push(child);
                        }
                        _ => {
                            if !current_decls.is_empty() {
                                new_children.push(CssNode::Rule {
                                    selector: sel.clone(),
                                    declarations: std::mem::take(&mut current_decls),
                                    children: vec![],
                                });
                            }
                            new_children.push(child);
                        }
                    }
                }
                if !current_decls.is_empty() {
                    new_children.push(CssNode::Rule {
                        selector: sel,
                        declarations: current_decls,
                        children: vec![],
                    });
                }
                Ok((Some(vec![CssNode::AtRule {
                    name: name.clone(),
                    params: params_str,
                    children: new_children,
                    has_body: body.is_some(),
                }]), env))
            } else {
                Ok((Some(vec![CssNode::AtRule {
                    name: name.clone(),
                    params: params_str,
                    children,
                    has_body: body.is_some(),
                }]), env))
            }
        }
        Node::Warn(v) => {
            let msg = eval_value(v, &env).to_css_string();
            tracing::warn!(message = %msg, "@warn");
            Ok((None, env))
        }
        Node::Debug(v) => {
            let msg = eval_value(v, &env).to_css_string();
            tracing::debug!(message = %msg, "@debug");
            Ok((None, env))
        }
        Node::Error(v) => {
            let msg = eval_value(v, &env).to_css_string();
            Err(SassError::eval(msg))
        }
    }
}

/// 求值值表达式——递归求值 AST 级别表达式。
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
        Value::Paren(v) => {
            eval_value(v, env)
        }
        Value::BinOp(b) => {
            // 短路求值
            match b.op {
                crate::eval::value::BinOpKind::And => {
                    let left = eval_value(&b.left, env);
                    if !left.is_truthy() { return left; }
                    return eval_value(&b.right, env);
                }
                crate::eval::value::BinOpKind::Or => {
                    let left = eval_value(&b.left, env);
                    if left.is_truthy() { return left; }
                    return eval_value(&b.right, env);
                }
                _ => {}
            }
            let left = eval_value(&b.left, env);
            let right = eval_value(&b.right, env);
            eval_binop(&b.op, left, right)
        }
        Value::UnaryOp(op, v) => {
            let val = eval_value(v, env);
            match op {
                crate::eval::value::UnaryOp::Neg => {
                    match Value::neg(val) {
                        Ok(v) => v,
                        Err(e) => { tracing::warn!(error = %e, "neg failed"); Value::Null }
                    }
                }
                crate::eval::value::UnaryOp::Not => Value::not(val),
            }
        }
        Value::Call(name, args) => {
            let evaluated_args: Vec<crate::parse::ast::Arg> = args.iter()
                .map(|a| crate::parse::ast::Arg {
                    name: a.name.clone(),
                    value: eval_value(&a.value, env),
                    spread: a.spread,
                })
                .collect();
            // 展开 spread 参数
            let spread_args: Vec<crate::parse::ast::Arg> = Vec::new();
            let mut final_args = spread_args;
            for a in &evaluated_args {
                if a.spread {
                    // 展开列表/arglist
                    match &a.value {
                        Value::List(items, _, _) => {
                            for item in items {
                                final_args.push(crate::parse::ast::Arg {
                                    name: None,
                                    value: item.clone(),
                                    spread: false,
                                });
                            }
                        }
                        Value::ArgList(items) => {
                            for item in items {
                                final_args.push(crate::parse::ast::Arg {
                                    name: None,
                                    value: item.clone(),
                                    spread: false,
                                });
                            }
                        }
                        _ => final_args.push(a.clone()),
                    }
                } else {
                    final_args.push(a.clone());
                }
            }
            match crate::eval::function::call_function(name, &final_args, env) {
                Ok(v) => v,
                Err(e) => {
                    tracing::debug!(error = %e, fn = %name, "function call failed");
                    // 未知函数——返回字符串形式
                    let inner: Vec<String> = evaluated_args.iter()
                        .map(|a| a.value.to_css_string())
                        .collect();
                    Value::String(
                        format!("{name}({})", inner.join(", ")),
                        crate::lex::token::QuoteStyle::None,
                    )
                }
            }
        }
        Value::Interp(s) => {
            // 插值——求值表达式内容
            eval_interp(s, env)
        }
        Value::Calc(s) => {
            Value::String(s.clone(), crate::lex::token::QuoteStyle::None)
        }
        _ => value.clone(),
    }
}

/// 求值插值字符串——处理 `#{expr}` 中的变量引用和简单表达式。
fn eval_interp(s: &str, env: &Env) -> Value {
    // 简化：如果是纯变量名，直接查
    if let Some(v) = env.get_var(s) {
        return v.clone();
    }
    // 尝试解析为表达式
    // 简化：直接返回字符串
    Value::String(s.to_string(), crate::lex::token::QuoteStyle::None)
}

/// 求值二元运算。
fn eval_binop(op: &crate::eval::value::BinOpKind, left: Value, right: Value) -> Value {
    use crate::eval::value::BinOpKind;
    match op {
        BinOpKind::Add => Value::add(left, right).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "add failed"); Value::Null
        }),
        BinOpKind::Sub => Value::sub(left, right).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "sub failed"); Value::Null
        }),
        BinOpKind::Mul => Value::mul(left, right).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "mul failed"); Value::Null
        }),
        BinOpKind::Div => Value::div(left, right).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "div failed"); Value::Null
        }),
        BinOpKind::Mod => Value::rem(left, right).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "mod failed"); Value::Null
        }),
        BinOpKind::Eq => Value::eq(left, right),
        BinOpKind::NotEq => Value::ne(left, right),
        BinOpKind::Lt => Value::lt(left, right),
        BinOpKind::Gt => Value::gt(left, right),
        BinOpKind::LtEq => Value::lte(left, right),
        BinOpKind::GtEq => Value::gte(left, right),
        BinOpKind::And | BinOpKind::Or => unreachable!(),
    }
}
