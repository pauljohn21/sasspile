//! InputSyntax — 支持的输入语法类型

/// 输入语法枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSyntax {
    /// SCSS 语法（带 $ 变量、@ 指令、分号）
    Scss,
    /// 缩进语法（.sass）
    Sass,
    /// 纯 CSS（仅 pass-through）
    Css,
}
