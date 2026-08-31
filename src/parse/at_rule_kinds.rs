//! Sass @规则和 CSS @规则的枚举定义。
//!
//! 替代散布在 `at_rules.rs`、`plain_css.rs`、`rule.rs` 中的 &str 字面量比较。

/// Sass 内建 @规则种类。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtRuleKind {
    /// `@if`
    If,
    /// `@for`
    For,
    /// `@each`
    Each,
    /// `@while`
    While,
    /// `@mixin`
    Mixin,
    /// `@include`
    Include,
    /// `@content`
    Content,
    /// `@function`
    Function,
    /// `@return`
    Return,
    /// `@use`
    Use,
    /// `@forward`
    Forward,
    /// `@import`
    Import,
    /// `@extend`
    Extend,
    /// `@at-root`
    AtRoot,
    /// `@warn`
    Warn,
    /// `@debug`
    Debug,
    /// `@error`
    Error,
    /// 未知 @规则——透传到 CSS 输出。
    Other(String),
}

impl AtRuleKind {
    /// 从 @规则名解析为枚举变体。
    pub fn from_str(name: &str) -> Self {
        match name {
            "if" => Self::If,
            "for" => Self::For,
            "each" => Self::Each,
            "while" => Self::While,
            "mixin" => Self::Mixin,
            "include" => Self::Include,
            "content" => Self::Content,
            "function" => Self::Function,
            "return" => Self::Return,
            "use" => Self::Use,
            "forward" => Self::Forward,
            "import" => Self::Import,
            "extend" => Self::Extend,
            "at-root" => Self::AtRoot,
            "warn" => Self::Warn,
            "debug" => Self::Debug,
            "error" => Self::Error,
            other => Self::Other(other.to_string()),
        }
    }

    /// 是否为已知 Sass @规则（非 Other）。
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Other(_))
    }
}

/// 标准 CSS @规则种类——用于 plain CSS 模式验证。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssAtRule {
    /// `@media`
    Media,
    /// `@supports`
    Supports,
    /// `@container`
    Container,
    /// `@import`
    Import,
    /// `@charset`
    Charset,
    /// `@page`
    Page,
    /// `@font-face`
    FontFace,
    /// `@font-feature-values`
    FontFeatureValues,
    /// `@keyframes`
    Keyframes,
    /// `@layer`
    Layer,
    /// `@scope`
    Scope,
    /// `@starting-style`
    StartingStyle,
    /// `@position-try`
    PositionTry,
    /// `@property`
    Property,
    /// `@namespace`
    Namespace,
    /// `@document`
    Document,
    /// 非 CSS 标准 @规则。
    Other(String),
}

impl CssAtRule {
    /// 从 @规则名解析为枚举变体（大小写不敏感）。
    pub fn from_str(name: &str) -> Self {
        let lower = name.to_lowercase();
        match lower.as_str() {
            "media" => Self::Media,
            "supports" => Self::Supports,
            "container" => Self::Container,
            "import" => Self::Import,
            "charset" => Self::Charset,
            "page" => Self::Page,
            "font-face" => Self::FontFace,
            "font-feature-values" => Self::FontFeatureValues,
            "keyframes" => Self::Keyframes,
            "layer" => Self::Layer,
            "scope" => Self::Scope,
            "starting-style" => Self::StartingStyle,
            "position-try" => Self::PositionTry,
            "property" => Self::Property,
            "namespace" => Self::Namespace,
            "document" => Self::Document,
            other => Self::Other(other.to_string()),
        }
    }

    /// 是否为 CSS 标准 @规则（非 Other）。
    pub fn is_valid(&self) -> bool {
        !matches!(self, Self::Other(_))
    }

    /// 是否为 keyframes（含 vendor prefix 变体）。
    pub fn is_keyframes(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower == "keyframes"
            || lower == "-webkit-keyframes"
            || lower == "-moz-keyframes"
            || lower == "-o-keyframes"
            || lower == "-ms-keyframes"
    }
}
