//! —— SCSS Evaluator ——
//!
//! SCSS 专用求值器，消费 `ScssAst`，产出 `Vec<CssNode>`。
//! 内部委托给 [`super::Evaluator`]（现有实现），
//! 类型系统保证 SCSS AST 不混入 CSS 路径。

use super::Evaluator;
use crate::css::node::CssNode;
use crate::error::Result;
use crate::eval::Env;
use crate::parse::scss_ast::{ScssAst, ScssNode};

/// SCSS 求值器——消费完整 Sass 语义。
pub struct ScssEvaluator;

impl ScssEvaluator {
    /// 求值 SCSS AST 为 CSS 节点树。
    pub fn evaluate(ast: &ScssAst) -> Result<Vec<CssNode>> {
        Evaluator::evaluate(ast)
    }

    /// 求值 SCSS AST 为 CSS 节点树（带初始 Env）。
    pub fn evaluate_with_env(ast: &ScssAst, env: Env) -> Result<Vec<CssNode>> {
        Evaluator::evaluate_with_env(ast, env)
    }

    /// 求值单个 SCSS 节点。
    pub fn eval_node(node: &ScssNode, env: Env) -> Result<(Vec<CssNode>, Env)> {
        Evaluator::eval_node(node, env)
    }

    /// 求值 SCSS 节点列表。
    pub fn eval_nodes(nodes: &[ScssNode], env: Env) -> Result<(Vec<CssNode>, Env)> {
        Evaluator::eval_nodes(nodes, env)
    }
}
