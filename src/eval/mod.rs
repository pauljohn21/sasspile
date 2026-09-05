//! 求值器——纯函数式管线 + move 语义（零 clone）。

use crate::__tracing::warn;
use crate::css::node::CssNode;
use crate::error::{Result, SassError};
use crate::parse::ast::*;

use std::path::PathBuf;

pub(crate) use env::{Env, FunctionDef, MixinDef, ModuleExports};
// 子模块通过 `use super::*` 获取这些类型
pub(crate) use std::collections::HashMap;
pub(crate) use std::rc::Rc;

/// 求值器。
pub struct Evaluator;
const MAX_DEPTH: usize = 100_000;

impl Evaluator {
    /// 求值 AST 为 CSS 节点树。
    ///
    /// # Errors
    ///
    /// 返回 [`SassError`] 如果求值遇到错误（如未定义变量、类型错误等）。
    pub fn evaluate(ast: &Ast) -> Result<Vec<CssNode>> {
        let (css, final_env) = Self::eval_nodes(&ast.nodes, Env::default())?;
        let extends = final_env.get_extends().to_vec();
        let css = if extends.is_empty() {
            css
        } else {
            let module_selectors = Self::build_module_selectors(final_env.get_module_cache());
            let css = Self::apply_extends(css, &extends, &module_selectors);
            Self::check_extend_targets(&css, &extends)?;
            css
        };
        Ok(hoist::hoist_css_imports(css))
    }

    /// 求值 AST 为 CSS 节点树（带初始 Env）。
    ///
    /// # Errors
    ///
    /// 返回 [`SassError`] 如果求值遇到错误。
    pub(crate) fn evaluate_with_env(ast: &Ast, env: Env) -> Result<Vec<CssNode>> {
        let (css, final_env) = Self::eval_nodes(&ast.nodes, env)?;
        let extends = final_env.get_extends().to_vec();
        let css = if extends.is_empty() {
            css
        } else {
            let module_selectors = Self::build_module_selectors(final_env.get_module_cache());
            let css = Self::apply_extends(css, &extends, &module_selectors);
            Self::check_extend_targets(&css, &extends)?;
            css
        };
        Ok(hoist::hoist_css_imports(css))
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip(nodes, env), fields(depth = env.get_depth(), n = nodes.len())))]
    fn eval_nodes(nodes: &[Node], env: Env) -> Result<(Vec<CssNode>, Env)> {
        match env.get_depth() > MAX_DEPTH {
            true => {
                warn!(depth = env.get_depth(), "recursion limit exceeded");
                return Err(SassError::Eval(
                    "Recursion depth limit exceeded (possible infinite loop)".into(),
                ));
            }
            false => {}
        }
        let (css, env) = nodes.iter().try_fold(
            (Vec::new(), env),
            |(mut css, env), node| -> Result<(Vec<CssNode>, Env)> {
                let (out, new_env) = Self::eval_node(node, env).map_err(|e| {
                    crate::__tracing::error!(error = %e, node_type = ?std::mem::discriminant(node), "eval_node failed");
                    e
                })?;
                css.extend(out);
                Ok((css, new_env))
            },
        )?;
        Ok((css, env))
    }

    /// 求值单个节点——纯函数分发，每个 arm 委托独立函数。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(node, env), fields(depth = env.get_depth())))]
    fn eval_node(node: &Node, env: Env) -> Result<(Vec<CssNode>, Env)> {
        match env.is_plain_css()
            && !matches!(
                node,
                Node::Use { .. } | Node::Forward { .. } | Node::Import { .. }
            )
        {
            true => Self::check_plain_css_node(node)?,
            false => {}
        }
        match node {
            Node::Rule { selector, body } => {
                match env.is_plain_css() {
                    true => Self::check_plain_css_selector(selector)?,
                    false => {}
                }
                Self::eval_rule(selector, body, env)
            }
            Node::Decl {
                property,
                value,
                important,
            } => eval_decl(property, value, *important, env),
            Node::Variable { name, value, flags } => Self::eval_variable(name, value, flags, env),
            Node::Comment(text, silent) => eval_comment(text, *silent, env),
            Node::If {
                branches,
                else_body,
            } => Self::eval_if(branches, else_body, env),
            Node::For {
                var,
                from,
                to,
                inclusive,
                body,
            } => Self::eval_for(var, from, to, *inclusive, body, env),
            Node::Each { vars, list, body } => Self::eval_each(vars, list, body, env),
            Node::While { cond, body } => Self::eval_while(cond, body, env),
            Node::MixinDef { name, params, body } => eval_mixin_def(name, params, body, env),
            Node::Include {
                name,
                args,
                content,
            } => Self::eval_include(name, args, content, env),
            Node::Content => eval_content(env),
            Node::FunctionDef { name, params, body } => eval_func_def(name, params, body, env),
            Node::Return(v) => eval_return(v, env),
            Node::Use {
                url,
                namespace,
                star,
                config,
            } => Self::eval_use(url, namespace, *star, config, env),
            Node::Forward {
                url,
                show,
                hide,
                prefix,
                config,
            } => Self::eval_forward(url, prefix, config, env, show, hide),
            Node::Import { url, modifier } => Self::eval_import(url, modifier, env),
            Node::Extend { selector, optional } => eval_extend_node(selector, *optional, env),
            Node::AtRoot { query, body } => Self::eval_at_root(query, body, env),
            Node::AtRule { name, params, body } => Self::eval_at_rule(name, params, body, env),
            Node::Warn(v) => eval_warn(v, env),
            Node::Debug(v) => eval_debug(v, env),
            Node::Error(v) => eval_error_node(v, env),
        }
    }
}

/// 求值声明节点。
fn eval_decl(
    property: &str,
    value: &Value,
    important: bool,
    env: Env,
) -> Result<(Vec<CssNode>, Env)> {
    use crate::eval::Evaluator;
    match env.is_plain_css() {
        true => {
            Evaluator::check_plain_css_value(value)?;
            match property.contains("#{") {
                true => return Err(SassError::Eval(
                    "Interpolation isn't allowed in plain CSS.".into(),
                )),
                false => {}
            }
        }
        false => {}
    }
    // 顶层声明检测：不在样式规则内的裸声明是非法的
    match env.get_selector().is_none() {
        true => return Err(SassError::Eval(
            "Declarations may only be used within style rules.".into(),
        )),
        false => {}
    }
    let val = Evaluator::eval_value(value, &env)?;
    // plain CSS 模式保留 null 值（如 `x: null`）
    match matches!(val, Value::Null) && !env.is_plain_css() {
        true => return Ok((vec![], env)),
        false => {}
    }
    let property = crate::eval::value::eval_property_name(property, &env);
    Ok((
        vec![CssNode::Declaration {
            property,
            value: val.to_string(),
            important,
        }],
        env,
    ))
}

/// 求值注释节点。
fn eval_comment(text: &str, silent: bool, env: Env) -> Result<(Vec<CssNode>, Env)> {
    match silent {
        true => Ok((vec![], env)),
        false => Ok((vec![CssNode::Comment(text.to_string())], env)),
    }
}

/// 求值 mixin 定义。
fn eval_mixin_def(
    name: &str,
    params: &[Param],
    body: &[Node],
    mut env: Env,
) -> Result<(Vec<CssNode>, Env)> {
    let captured = std::mem::take(&mut env.namespaces);
    Ok((
        vec![],
        env.define_mixin(
            name.to_string(),
            MixinDef {
                params: params.to_vec(),
                body: body.to_vec(),
                captured_namespaces: captured,
            },
        ),
    ))
}

/// 求值 @content 节点。
fn eval_content(env: Env) -> Result<(Vec<CssNode>, Env)> {
    use crate::eval::Evaluator;
    match env.get_content() {
        Some((content_nodes, content_env)) => {
            // @content 在 mixin body 内执行，继承当前 current_selector
            let content_env = content_env.clone().with_selector(
                env.get_selector()
                    .map(std::string::ToString::to_string)
                    .unwrap_or_default(),
            );
            let content_nodes = content_nodes.to_vec();
            Evaluator::eval_nodes(&content_nodes, content_env)
        }
        None => Ok((vec![], env)),
    }
}

/// 求值函数定义。
fn eval_func_def(
    name: &str,
    params: &[Param],
    body: &[Node],
    mut env: Env,
) -> Result<(Vec<CssNode>, Env)> {
    let captured = std::mem::take(&mut env.namespaces);
    Ok((
        vec![],
        env.define_function(
            name.to_string(),
            FunctionDef {
                params: params.to_vec(),
                body: body.to_vec(),
                captured_namespaces: captured,
            },
        ),
    ))
}

/// 求值 @return 节点。
fn eval_return(v: &Value, env: Env) -> Result<(Vec<CssNode>, Env)> {
    use crate::eval::Evaluator;
    let val = Evaluator::eval_value(v, &env)?;
    Ok((vec![CssNode::Return(val)], env))
}

/// 求值 @extend 节点。
///
/// 按 `,` 拆分 target，为每个目标生成独立 extend 条目。
/// 校验：复杂选择器（`a b`）、复合选择器（`a:hover`）、空选择器报错。
fn eval_extend_node(selector: &str, optional: bool, env: Env) -> Result<(Vec<CssNode>, Env)> {
    match env.get_selector().map(std::string::ToString::to_string) {
        Some(extender) => {
            let module = env.get_base_path().cloned();
            let targets: Vec<&str> = selector.split(',').map(str::trim).collect();
            // 空选择器校验
            if targets.iter().all(|s| s.is_empty()) {
                return Err(SassError::Eval("expected selector.".into()));
            }
            let env = targets
                .into_iter()
                .filter(|s| !s.is_empty())
                .try_fold(env, |env, target| {
                    // 复杂选择器校验：包含空格（多 compound）→ 报错
                    if target.chars().any(|c| c.is_whitespace()) {
                        return Err(SassError::Eval(
                            "complex selectors may not be extended.".into(),
                        ));
                    }
                    // 复合选择器校验：包含 `:` 伪类/伪元素且非纯伪类 → 报错
                    if target.contains(':') && !target.starts_with(':') {
                        return Err(SassError::Eval(
                            "compound selectors may no longer be extended.\nConsider `@extend a, :hover` instead.\nSee https://sass-lang.com/d/extend-compound for details.".into(),
                        ));
                    }
                    Ok(env.add_extend(extender.clone(), target.to_string(), optional, module.clone()))
                })?;
            Ok((vec![], env))
        }
        None => Ok((vec![], env)),
    }
}

/// 求值 @warn 节点。
fn eval_warn(_v: &Value, env: Env) -> Result<(Vec<CssNode>, Env)> {
    Ok((vec![], env))
}

/// 求值 @debug 节点。
fn eval_debug(_v: &Value, env: Env) -> Result<(Vec<CssNode>, Env)> {
    Ok((vec![], env))
}

/// 求值 @error 节点。
fn eval_error_node(v: &Value, env: Env) -> Result<(Vec<CssNode>, Env)> {
    use crate::eval::Evaluator;
    let msg = Evaluator::eval_value(v, &env)?;
    Err(SassError::Eval(msg.to_string()))
}

mod at_params;
mod builtin;
mod color;
mod color_names;
mod control_flow;
mod env;
mod env_impl;
pub(crate) mod error_msgs;
mod extend;
mod file_resolver;
mod forward;
mod hoist;
mod import;
mod meta_ops;
mod mixin;
mod module;
mod module_helpers;
mod plain_css;
mod rule;
mod scope;
pub mod value;
