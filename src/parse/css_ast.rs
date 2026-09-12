//! —— CSS AST ——
//!
//! 仅包含原生 CSS + CSS Nesting。
//! 无变量、无控制流、无 mixin、无函数、无 @extend、无 @at-root。
//! .css 文件解析产出此类型。

use super::ast::{Color, Separator};

/// CSS 语法树——.css 文件解析产出。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CssAst {
    pub nodes: Vec<CssNode>,
}

/// CSS 语法树节点——仅原生 CSS + Nesting。
#[derive(Debug, Clone, PartialEq)]
pub enum CssNode {
    Rule {
        selector: String,
        body: Vec<CssNode>,
    },
    Decl {
        property: String,
        value: CssValue,
        important: bool,
    },
    Comment(String),
    AtRule {
        name: String,
        params: Option<String>,
        body: Option<Vec<CssNode>>,
    },
    Import {
        url: String,
        modifier: Option<String>,
    },
}

/// CSS 值表达式——CSS 原生表达式子集。
#[derive(Debug, Clone, PartialEq)]
pub enum CssValue {
    Number(f64, Option<String>),
    String(String, bool),
    Color(Color),
    List(Vec<CssValue>, Separator, bool),
    Call(String, Vec<CssArg>),
    Calc(String),
    Var(String),
}

/// CSS 函数调用参数。
#[derive(Debug, Clone, PartialEq)]
pub struct CssArg {
    pub name: Option<String>,
    pub value: CssValue,
}
