//! 阶段 3: Parsed —— 抽象语法树。
//!
//! 语法分析器的输出，包含从 Token 序列构建的抽象语法树（AST）。

use super::evaluated::Evaluated;
use crate::error::Result;
use crate::parse::ast::Ast;

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
}

impl Parsed {
    /// 求值——Parsed → Evaluated。
    ///
    /// 消费自身，返回求值后的 CSS 树或错误。
    ///
    /// # 返回
    /// 成功返回 `Evaluated`（包含 CSS 节点树），失败返回求值错误。
    pub fn evaluate(self) -> Result<Evaluated> {
        use crate::eval::Evaluator;

        let nodes = Evaluator::evaluate(&self.ast)?;
        Ok(Evaluated { nodes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ast::Ast;

    #[test]
    fn test_parsed_evaluate() {
        let parsed = Parsed {
            ast: Ast::default(),
        };
        let evaluated = parsed.evaluate().unwrap();
        assert!(evaluated.nodes.is_empty());
    }
}
