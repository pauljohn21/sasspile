//! 阶段 3: Parsed —— 抽象语法树。
//!
//! 语法分析器的输出，包含从 Token 序列构建的抽象语法树（AST）。
//! 携带文件路径和加载路径，用于求值阶段的 @use/@import 解析。

use super::evaluated::Evaluated;
use crate::error::Result;
use crate::parse::ast::Ast;
use std::path::PathBuf;

/// 语法分析产物——AST。
///
/// 由 `Lexed::parse()` 产生，作为求值阶段的输入。
///
/// # 示例
///
/// ```
/// use sasspile::stage::source::Source;
///
/// let parsed = Source::new("a { color: red; }".to_string())
///     .lex().unwrap()
///     .parse().unwrap();
/// let evaluated = parsed.evaluate().unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Parsed {
    /// 抽象语法树。
    pub ast: Ast,
    /// 源文件路径（用于 @use/@import 解析和 plain CSS 检测）。
    pub base_path: Option<PathBuf>,
    /// 加载路径（用于 @use/@import 模块搜索）。
    pub load_paths: Vec<PathBuf>,
}

impl Parsed {
    /// 求值——Parsed → Evaluated。
    ///
    /// 消费自身，构建 Env 并求值 AST，返回求值后的 CSS 树或错误。
    /// 当携带 `base_path` 时，自动设置文件路径和 plain CSS 模式。
    /// 当携带 `load_paths` 时，自动设置模块搜索路径。
    ///
    /// # 返回
    /// 成功返回 `Evaluated`（包含 CSS 节点树），失败返回求值错误。
    pub fn evaluate(self) -> Result<Evaluated> {
        use crate::eval::{Env, Evaluator};

        let mut env = Env::default();

        if let Some(ref path) = self.base_path {
            let is_plain_css = path.extension().and_then(|e| e.to_str()) == Some("css");
            env = env
                .with_base_path(path.clone())
                .with_plain_css(is_plain_css);
        }

        if !self.load_paths.is_empty() {
            env = env.with_load_paths(self.load_paths.clone());
        }

        let nodes = Evaluator::evaluate_with_env(&self.ast, env)?;
        Ok(Evaluated { nodes })
    }
}
