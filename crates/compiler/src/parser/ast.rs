//! SassAstNode — 完整 Sass 语法解析树

/// 解析树节点 — 完整表达 Sass SCSS/SASS/CSS 语法
#[derive(Debug, Clone, PartialEq)]
pub enum SassAstNode {
    /// 变量声明: `$name: value;` 或 `$name: value !default;`
    VariableDecl {
        /// 变量名（含 $）
        name: String,
        /// 值节点
        value: Box<SassAstNode>,
        /// 是否带 !default
        has_default: bool,
    },
    /// 嵌套规则: `selector { inner... }`
    Rule {
        /// 选择器文本
        selector: String,
        /// 子节点
        inner: Vec<SassAstNode>,
    },
    /// 样式声明: `property: value;`
    StyleDecl {
        /// 属性名
        prop: String,
        /// 属性值
        value: String,
    },
    /// 插值表达式: `#{expr}`
    Interpolated(String),
    /// 父选择器 `&`
    ParentSelector,
    /// Map 字面量: `(key: value, ...)`
    MapLiteral(Vec<(SassAstNode, SassAstNode)>),
    /// List 字面量: `(item, item, ...)`
    ListLiteral(Vec<SassAstNode>),
    /// 注释
    Comment(String),
    /// 未知 / 透传
    Raw(String),
}
