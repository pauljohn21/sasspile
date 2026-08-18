//! AST types — statement, expression, and value definitions.

/// Top-level statement in a SCSS stylesheet.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// `.foo { ... }`
    StyleRule {
        selector: String,
        body: Vec<Stmt>,
    },
    /// `color: red`
    Declaration {
        property: String,
        value: Expr,
    },
    /// `$var: value;`
    VariableDecl {
        name: String,
        value: Expr,
        default: bool,
        global: bool,
    },
    /// `@mixin name($a) { ... }`
    MixinDef {
        name: String,
        params: Vec<crate::ast::Param>,
        body: Vec<Stmt>,
    },
    /// `@include name(args) { @content }`
    IncludeCall {
        name: String,
        args: Vec<crate::ast::Arg>,
        content: Option<Vec<Stmt>>,
    },
    /// `@function name($a) { @return ... }`
    FunctionDef {
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
    /// `@return expr;`
    ReturnStmt(Expr),
    /// `@if cond { ... } @else if cond { ... } @else { ... }`
    IfStmt {
        branches: Vec<(Expr, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
    },
    /// `@for $i from start through/to end { ... }`
    ForStmt {
        var: String,
        from: Expr,
        to: Expr,
        exclusive: bool,
        body: Vec<Stmt>,
    },
    /// `@each $key, $value in list { ... }`
    EachStmt {
        vars: Vec<String>,
        list: Expr,
        body: Vec<Stmt>,
    },
    /// `@while cond { ... }`
    WhileStmt {
        cond: Expr,
        body: Vec<Stmt>,
    },
    /// `@error "msg";`
    ErrorStmt(Expr),
    /// `@warn "msg";`
    WarnStmt(Expr),
    /// `@debug "msg";`
    DebugStmt(Expr),
    /// `@at-root { ... }`
    AtRootRule(Vec<Stmt>),
    /// `@media query { ... }`
    MediaRule {
        query: String,
        body: Vec<Stmt>,
    },
    /// `@supports condition { ... }`
    SupportsRule {
        condition: String,
        body: Vec<Stmt>,
    },
    /// `@use "url" as name with (...)`
    UseRule {
        url: String,
        namespace: Option<String>,
        config: Vec<(String, Expr)>,
    },
    /// `@forward "url" show/hide ...`
    ForwardRule {
        url: String,
        show: Option<Vec<String>>,
        hide: Option<Vec<String>>,
    },
    /// `@import "url"`
    ImportRule(String),
    /// `@extend .selector;` or `@extend %placeholder;`
    ExtendRule {
        selector: String,
        optional: bool,
    },
    /// `@content;`
    ContentRule,
    /// CSS pass-through at-rule
    CssAtRule {
        name: String,
        value: String,
        body: Option<Vec<Stmt>>,
    },
    /// Block comment preserved in output
    Comment(String),
}

/// Expression in SCSS.
#[derive(Debug, Clone)]
pub enum Expr {
    /// Literal value
    Literal(crate::value::Value),
    /// Variable reference $name
    Variable(String),
    /// Binary operation
    Operation {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// Unary operation
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    /// Function call name(args)
    FunctionCall {
        name: String,
        args: Vec<Arg>,
        namespace: Option<String>,
    },
    /// Interpolation #{...}
    Interpolation(Vec<InterpPart>),
    /// List literal
    ListExpr {
        items: Vec<Expr>,
        separator: ListSeparator,
        bracketed: bool,
    },
    /// Map literal
    MapExpr(Vec<(Expr, Expr)>),
    /// Parenthesized expression
    Paren(Box<Expr>),
    /// Parent selector &
    ParentSelector,
/// Namespace-qualified variable reference: ns.$name
NamespacedVariable {
namespace: String,
name: String,
},
}

/// Binary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

/// Unary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// List separator types.
#[derive(Debug, Clone, PartialEq)]
pub enum ListSeparator {
    Space,
    Comma,
    Slash,
    Undetermined,
}

/// Function/mixin argument.
#[derive(Debug, Clone)]
pub struct Arg {
    pub name: Option<String>,
    pub value: Expr,
    pub spread: bool,
}

/// Function/mixin parameter.
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub default: Option<Expr>,
    pub rest: bool,
}

/// Part of an interpolation string.
#[derive(Debug, Clone)]
pub enum InterpPart {
    Literal(String),
    Expr(Expr),
}

impl Stmt {
    /// Return a short name for the statement type (for tracing).
    pub fn node_name(&self) -> &'static str {
        match self {
            Stmt::StyleRule { .. } => "StyleRule",
            Stmt::Declaration { .. } => "Declaration",
            Stmt::VariableDecl { .. } => "VariableDecl",
            Stmt::MixinDef { .. } => "MixinDef",
            Stmt::IncludeCall { .. } => "IncludeCall",
            Stmt::FunctionDef { .. } => "FunctionDef",
            Stmt::ReturnStmt(_) => "ReturnStmt",
            Stmt::IfStmt { .. } => "IfStmt",
            Stmt::ForStmt { .. } => "ForStmt",
            Stmt::EachStmt { .. } => "EachStmt",
            Stmt::WhileStmt { .. } => "WhileStmt",
            Stmt::ErrorStmt(_) => "ErrorStmt",
            Stmt::WarnStmt(_) => "WarnStmt",
            Stmt::DebugStmt(_) => "DebugStmt",
            Stmt::AtRootRule(_) => "AtRootRule",
            Stmt::MediaRule { .. } => "MediaRule",
            Stmt::SupportsRule { .. } => "SupportsRule",
            Stmt::UseRule { .. } => "UseRule",
            Stmt::ForwardRule { .. } => "ForwardRule",
            Stmt::ImportRule(_) => "ImportRule",
            Stmt::ExtendRule { .. } => "ExtendRule",
            Stmt::ContentRule => "ContentRule",
            Stmt::CssAtRule { .. } => "CssAtRule",
            Stmt::Comment(_) => "Comment",
        }
    }
}
