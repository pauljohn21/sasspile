//! —— CSS Evaluator ——
//!
//! 原生 CSS 求值器——消费 `CssAst`，产出 `Vec<CssNode>`。
//! 核心原则：**保留 CSS 原生嵌套结构**，不组合选择器，不提升 at-rule。
//!
//! 与 `ScssEvaluator` 的关键差异：
//! - ❌ 不组合选择器（保留 CSS 原生嵌套）
//! - ❌ 不处理变量 / @if / @for / mixin / 函数
//! - ❌ 不处理 @extend / @at-root
//! - ✅ @import / @media / @keyframes 透传
//! - ✅ CSS 值表达式格式化输出

use crate::css::node::CssNode;
use crate::error::Result;
use crate::parse::ast::Separator;
use crate::parse::css_ast::{CssArg, CssAst, CssNode as CssAstNode, CssValue};

/// CSS 求值器——消费原生 CSS + Nesting。
pub struct CssEvaluator;

impl CssEvaluator {
    /// 求值 CSS AST 为 CSS 节点树。
    pub fn evaluate(ast: &CssAst) -> Result<Vec<CssNode>> {
        Self::eval_nodes(&ast.nodes)
    }

    fn eval_nodes(nodes: &[CssAstNode]) -> Result<Vec<CssNode>> {
        nodes.iter().try_fold(Vec::new(), |mut acc, node| {
            acc.extend(Self::eval_node(node)?);
            Ok(acc)
        })
    }

    fn eval_node(node: &CssAstNode) -> Result<Vec<CssNode>> {
        match node {
            CssAstNode::Rule { selector, body } => Self::eval_rule(selector, body),
            CssAstNode::Decl {
                property,
                value,
                important,
            } => Self::eval_decl(property, *important, value),
            CssAstNode::Comment(text) => Ok(vec![CssNode::Comment(text.clone())]),
            CssAstNode::AtRule {
                name,
                params,
                body,
            } => Self::eval_at_rule(name, params, body),
            CssAstNode::Import { url, modifier } => Self::eval_import(url, modifier),
        }
    }

    fn eval_rule(selector: &str, body: &[CssAstNode]) -> Result<Vec<CssNode>> {
        let mut declarations = Vec::new();
        let mut children = Vec::new();

        for node in body {
            match node {
                CssAstNode::Decl {
                    property,
                    value,
                    important,
                } => {
                    if let Some(decl) = Self::format_decl(property, *important, value)? {
                        declarations.push(decl);
                    }
                }
                _ => children.extend(Self::eval_node(node)?),
            }
        }

        Ok(vec![CssNode::Rule {
            selector: selector.to_string(),
            declarations,
            children,
        }])
    }

    fn eval_decl(property: &str, important: bool, value: &CssValue) -> Result<Vec<CssNode>> {
        Self::format_decl(property, important, value).map(|opt| match opt {
            Some(decl) => vec![decl],
            None => vec![],
        })
    }

    fn format_decl(property: &str, important: bool, value: &CssValue) -> Result<Option<CssNode>> {
        let value_str = Self::format_value(value)?;
        match value_str.is_empty() {
            true => Ok(None),
            false => Ok(Some(CssNode::Declaration {
                property: property.to_string(),
                value: value_str,
                important,
            })),
        }
    }

    fn eval_at_rule(
        name: &str,
        params: &Option<String>,
        body: &Option<Vec<CssAstNode>>,
    ) -> Result<Vec<CssNode>> {
        let children = match body {
            Some(nodes) => Self::eval_nodes(nodes)?,
            None => vec![],
        };

        let params_str = params.clone().unwrap_or_default();
        let has_body = body.is_some();

        Ok(vec![CssNode::AtRule {
            name: name.to_string(),
            params: Some(params_str),
            children,
            has_body,
        }])
    }

    fn eval_import(url: &str, modifier: &Option<String>) -> Result<Vec<CssNode>> {
        let full_params = match modifier {
            Some(modifier) if !modifier.is_empty() => format!("{url} {modifier}"),
            _ => url.to_string(),
        };

        Ok(vec![CssNode::AtRule {
            name: "import".to_string(),
            params: Some(full_params),
            children: vec![],
            has_body: false,
        }])
    }

    // —— CSS 值格式化 ——

    fn format_value(value: &CssValue) -> Result<String> {
        match value {
            CssValue::Number(n, unit) => Self::format_number(*n, unit.as_deref()),
            CssValue::String(s, quoted) => Self::format_string(s, *quoted),
            CssValue::Color(c) => Ok(Self::format_color(c)),
            CssValue::List(items, sep, bracketed) => Self::format_list(items, sep.clone(), *bracketed),
            CssValue::Call(name, args) => Self::format_call(name, args),
            CssValue::Calc(s) => Ok(s.clone()),
            CssValue::Var(name) => Ok(format!("var({name})")),
        }
    }

    fn format_number(n: f64, unit: Option<&str>) -> Result<String> {
        let num_str = Self::format_float(n);
        match unit {
            Some(u) if !u.is_empty() => Ok(format!("{num_str}{u}")),
            _ => Ok(num_str),
        }
    }

    fn format_string(s: &str, quoted: bool) -> Result<String> {
        match quoted {
            true => Ok(format!("\"{s}\"")),
            false => Ok(s.to_string()),
        }
    }

    fn format_color(c: &crate::parse::ast::Color) -> String {
        use crate::parse::ast::ColorSpace;

        if c.space == ColorSpace::Rgb || c.space == ColorSpace::Srgb {
            let r = c.channels[0].round() as u8;
            let g = c.channels[1].round() as u8;
            let b = c.channels[2].round() as u8;
            let a = c.a;
            match a < 1.0 && a > 0.0 {
                true => format!("rgba({r}, {g}, {b}, {a})"),
                false => format!("#{r:02x}{g:02x}{b:02x}"),
            }
        } else {
            format!("color({c:?})")
        }
    }

    fn format_list(items: &[CssValue], sep: Separator, bracketed: bool) -> Result<String> {
        let parts: Result<Vec<String>> = items.iter().map(Self::format_value).collect();
        let parts = parts?;
        let separator = match sep {
            Separator::Comma => ", ",
            Separator::Slash => " / ",
            _ => " ",
        };
        let joined = parts.join(separator);
        match bracketed {
            true => Ok(format!("[{joined}]")),
            false => Ok(joined),
        }
    }

    fn format_call(name: &str, args: &[CssArg]) -> Result<String> {
        let parts: Result<Vec<String>> = args.iter().map(|a| Self::format_value(&a.value)).collect();
        let parts = parts?;
        Ok(format!("{name}({})", parts.join(", ")))
    }

    fn format_float(n: f64) -> String {
        match n.fract() == 0.0 && n.abs() < 1e15 {
            true => format!("{n:.0}"),
            false => {
                let s = format!("{n:.6}");
                let s = s.trim_end_matches('0').trim_end_matches('.');
                s.to_string()
            }
        }
    }
}
