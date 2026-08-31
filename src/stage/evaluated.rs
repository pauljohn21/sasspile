//! 阶段 4: Evaluated —— CssNode 序列。
//!
//! 求值器的输出，包含求值后的 CSS 节点树。

use super::serialized::Serialized;
use crate::OutputStyle;
use crate::css::node::CssNode;

/// 求值产物——CssNode 中间表示。
///
/// 由 `Parsed::evaluate()` 产生，包含求值后的 CSS 节点树。
///
/// # 示例
///
/// ```
/// use sasspile::stage::source::Source;
/// use sasspile::OutputStyle;
///
/// let evaluated = Source::new("a { color: red; }".to_string())
///     .lex().unwrap()
///     .parse().unwrap()
///     .evaluate().unwrap();
/// let serialized = evaluated.serialize(OutputStyle::Expanded);
/// ```
#[derive(Debug, Clone)]
pub struct Evaluated {
    /// CssNode 列表。
    pub nodes: Vec<CssNode>,
}

impl Evaluated {
    /// 序列化——Evaluated → Serialized。
    ///
    /// 将 CssNode 树序列化为 CSS 字符串。
    ///
    /// # 参数
    /// - `style`: 输出风格（展开式或压缩式）。
    ///
    /// # 返回
    /// 返回包含 CSS 字符串的 `Serialized` 实例。
    pub fn serialize(self, style: OutputStyle) -> Serialized {
        use crate::css::Serializer;

        let css = Serializer::serialize(&self.nodes, style);
        Serialized { css }
    }
}

