//! AST 节点定义 — Token / Node / CssNode 类型
//!
//! 三个核心 Token 流通过 scan 累积在不同阶段间传递

/// Token 单元：从 char 流通过 scan 聚合得到
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Whitespace,
    Newline,
    Indent(u32),
    /// 标识符（属性名、选择器片段等）
    Ident(String),
    /// 字符串字面量（保留引号）
    String(String),
    /// 数字
    Number(String),
    /// SCSS 变量前缀
    Dollar,
    /// 插值前缀
    Hash,
    /// 插值表达式占位 — 形态如 "#{$a}-suffix",用于选择器/属性名/值
    Interpolation(String),
    /// At 指令前缀
    At,
    Dot,
    Colon,
    Semicolon,
    Comma,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    /// 通用运算符
    Op(String),
    /// 注释
    Comment(String),
    /// 字面 char（fallback）
    Char(char),
}

/// AST Node：从 Token 流通过 scan(Parser) 构建得到
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// CSS 规则：selector { ... }
    Rule {
        selector: String,
        body: Vec<Node>,
    },
    /// 声明：property: value;
    Declaration {
        prop: String,
        value: String,
    },
    /// @import / @forward
    Import { path: String },
    /// @use (module system)
    Use { path: String },
    /// @forward (module system)
    Forward { path: String },
    /// @mixin 定义
    MixinDef {
        name: String,
        params: Vec<String>,
        body: Vec<Node>,
    },
    /// @include 调用
    MixinCall {
        name: String,
        args: Vec<String>,
    },
    /// @extend 指令
    Extend {
        selector: String,
    },
    /// @if / @else 分支
    If {
        condition: String,
        then_branch: Vec<Node>,
        else_branch: Option<Vec<Node>>,
    },
    /// @for 循环
    For {
        var: String,
        from: String,
        to: String,
        body: Vec<Node>,
    },
    /// SCSS 变量声明
    Variable {
        name: String,
        value: String,
    },
    /// @include / @debug / @warn 等内置指令
    Directive {
        name: String,
        args: String,
    },
    /// 原始文本片段（verbatim）
    Text(String),
}

/// 最终渲染节点：从 Node 流通过 flat_map(evaluate) 展开得到
#[derive(Debug, Clone, PartialEq)]
pub enum CssNode {
    Rule {
        selector: String,
        body: Vec<CssNode>,
    },
    Declaration {
        prop: String,
        value: String,
    },
    Comment(String),
    Text(String),
}

impl CssNode {
    /// 将 CssNode 渲染为 CSS 字符串片段
    pub fn render(&self) -> String {
        match self {
            Self::Rule { selector, body } => {
                let inner = body.iter().map(Self::render).collect::<Vec<_>>().join("");
                format!("{selector} {{{inner}}}")
            }
            Self::Declaration { prop, value } => format!("{prop}: {value};"),
            Self::Comment(c) => format!("/* {c} */"),
            Self::Text(t) => t.clone(),
        }
    }
}
