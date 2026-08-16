//! AST node definitions for SCSS.

use crate::source::SourceSpan;

/// Top-level stylesheet.
#[derive(Debug, Clone)]
pub struct Stylesheet {
    /// Top-level nodes.
    pub nodes: Vec<Node>,
}

/// Any node in the stylesheet.
#[derive(Debug, Clone)]
pub enum Node {
    /// Style rule (selector + body).
    Rule(Rule),
    /// Property declaration.
    Declaration(Declaration),
    /// @-rule (use, import, mixin, etc.).
    AtRule(AtRule),
    /// Comment.
    Comment(Comment),
}

/// Style rule: selector + nested body.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Selector expression.
    pub selector: Selector,
    /// Nested nodes (rules, declarations, at-rules).
    pub nodes: Vec<Node>,
}

/// Property declaration: name: value [!important].
#[derive(Debug, Clone)]
pub struct Declaration {
    /// Property name (for variables, without the `$` prefix).
    pub name: String,
    /// Value expression.
    pub value: Expr,
    /// `!important` flag.
    pub important: bool,
    /// Source span.
    pub span: SourceSpan,
    /// `true` if this is a variable declaration (parsed from `$name: value`).
    pub is_variable: bool,
}

/// Selector expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    /// Type selector (e.g., `div`).
    Type(String),
    /// Class selector (e.g., `.foo`).
    Class(String),
    /// ID selector (e.g., `#bar`).
    Id(String),
    /// Attribute selector (e.g., `[type="text"]`).
    Attribute(String),
    /// Pseudo class/element.
    Pseudo(String),
    /// Parent reference (e.g., `&:hover`).
    ParentRef(Box<Selector>),
    /// Compound selector (multiple parts).
    Compound(Vec<Selector>),
    /// Descendant combinator.
    Descendant(Box<Selector>, Box<Selector>),
    /// Child combinator (`>`).
    Child(Box<Selector>, Box<Selector>),
    /// Adjacent sibling (`+`).
    Adjacent(Box<Selector>, Box<Selector>),
    /// General sibling (`~`).
    Sibling(Box<Selector>, Box<Selector>),
    /// Interpolation placeholder in selector.
    Interpolation(String),
    /// Universal selector (`*`).
    Universal,
    /// Literal text selector.
    Literal(String),
}

/// @-rule variants.
#[derive(Debug, Clone)]
pub enum AtRule {
    /// `@use "module" as ns with (...)`.
    Use(UseRule),
    /// `@import "url"`.
    Import(ImportRule),
    /// `@forward "module"`.
    Forward(ForwardRule),
    /// `@mixin name(...) { ... }`.
    Mixin(MixinDef),
    /// `@include name(...)`.
    Include(IncludeRule),
    /// `@function name(...) { ... }`.
    Function(FunctionDef),
    /// `@return expr`.
    Return(Expr),
    /// `@if expr { ... }`.
    If(IfStmt),
    /// `@else` or `@else if expr { ... }`.
    Else(Vec<Node>),
    /// `@for $var from start to/through end { ... }`.
    For(ForStmt),
    /// `@each $var in list { ... }`.
    Each(EachStmt),
    /// `@while expr { ... }`.
    While(WhileStmt),
    /// `@extend selector`.
    Extend(Selector),
    /// `@at-root { ... }`.
    AtRoot(Vec<Node>),
    /// `@media query { ... }`.
    Media(MediaRule),
    /// `@supports query { ... }`.
    Supports(SupportsRule),
    /// `@content`.
    Content,
    /// `@debug expr`.
    Debug(Expr),
    /// `@warn expr`.
    Warn(Expr),
    /// `@error expr`.
    Error(Expr),
}

/// @use rule.
#[derive(Debug, Clone)]
pub struct UseRule {
    /// Module URL or name.
    pub url: String,
    /// Optional namespace (`as ns`).
    pub namespace: Option<String>,
    /// Optional configuration (`with ($var: default)`).
    pub config: Vec<(String, Expr)>,
}

/// @import rule.
#[derive(Debug, Clone)]
pub struct ImportRule {
    /// Import URL(s).
    pub urls: Vec<String>,
}

/// @forward rule.
#[derive(Debug, Clone)]
pub struct ForwardRule {
    /// Module URL.
    pub url: String,
}

/// @mixin definition.
#[derive(Debug, Clone)]
pub struct MixinDef {
    /// Mixin name.
    pub name: String,
    /// Parameters.
    pub params: Vec<Param>,
    /// Body.
    pub body: Vec<Node>,
}

/// @include invocation.
#[derive(Debug, Clone)]
pub struct IncludeRule {
    /// Mixin name.
    pub name: String,
    /// Arguments.
    pub args: Vec<Expr>,
    /// Body nodes (used when @include has { ... } block).
    pub body: Vec<Node>,
}

/// @function definition.
#[derive(Debug, Clone)]
pub struct FunctionDef {
    /// Function name.
    pub name: String,
    /// Parameters.
    pub params: Vec<Param>,
    /// Body.
    pub body: Vec<Node>,
}

/// Function/mixin parameter.
#[derive(Debug, Clone)]
pub struct Param {
    /// Parameter name (including $).
    pub name: String,
    /// Optional default value.
    pub default: Option<Expr>,
}

/// @if statement.
#[derive(Debug, Clone)]
pub struct IfStmt {
    /// Condition expression.
    pub condition: Expr,
    /// Body for true branch.
    pub body: Vec<Node>,
    /// Optional else clause.
    pub else_body: Option<Vec<Node>>,
}

/// @for statement.
#[derive(Debug, Clone)]
pub struct ForStmt {
    /// Loop variable name.
    pub var: String,
    /// Start expression.
    pub start: Expr,
    /// End expression.
    pub end: Expr,
    /// Inclusive (`through`) or exclusive (`to`).
    pub inclusive: bool,
    /// Loop body.
    pub body: Vec<Node>,
}

/// @each statement.
#[derive(Debug, Clone)]
pub struct EachStmt {
    /// Loop variable(s).
    pub vars: Vec<String>,
    /// List expression.
    pub list: Expr,
    /// Loop body.
    pub body: Vec<Node>,
}

/// @while statement.
#[derive(Debug, Clone)]
pub struct WhileStmt {
    /// Condition expression.
    pub condition: Expr,
    /// Loop body.
    pub body: Vec<Node>,
}

/// @media rule.
#[derive(Debug, Clone)]
pub struct MediaRule {
    /// Media query string.
    pub query: String,
    /// Nested body.
    pub body: Vec<Node>,
}

/// @supports rule.
#[derive(Debug, Clone)]
pub struct SupportsRule {
    /// Supports condition.
    pub condition: String,
    /// Nested body.
    pub body: Vec<Node>,
}

/// Common comment.
#[derive(Debug, Clone)]
pub struct Comment {
    /// Comment text.
    pub text: String,
    /// `/**/` (silent) vs `//` (loud).
    pub silent: bool,
}

/// Expression AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Variable reference (`$var`). Name does NOT include the `$` prefix.
    Variable(String),
    /// Bare identifier (CSS value like `red`, `bold`, `button`). Treated as a string literal in evaluation.
    Identifier(String),
    /// Numeric literal.
    Number(f64, Option<String>),
    /// String literal.
    String(String),
    /// Boolean literal.
    Boolean(bool),
    /// Null literal.
    Null,
    /// Color literal.
    Color(u32),
    /// URL literal (`url(...)`).
    Url(String),
    /// Interpolation `#{...}`.
    Interpolation(Box<Expr>),
    /// Binary operation.
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    /// Unary operation.
    Unary(UnaryOp, Box<Expr>),
    /// Function call.
    Call(String, Vec<Expr>),
    /// List literal.
    List(Vec<Expr>),
    /// Map literal (key-value pairs).
    Map(Vec<(Expr, Expr)>),
    /// Parenthesized expression.
    Parens(Box<Expr>),
    /// Slash-separated values.
    SlashList(Vec<Expr>),
    /// Space-separated values (e.g., `1px sans-serif`, `1px 2px 3px`).
    SpaceList(Vec<Expr>),
    /// Spread in arg list: `$args...`
    Spread(Box<Expr>),
    /// Named argument (`$arg: value`) in function calls.
    NamedArg(String, Box<Expr>),
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Greater,
    Less,
    GreaterEq,
    LessEq,
    And,
    Or,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}
