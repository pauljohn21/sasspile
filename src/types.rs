use std::convert::Infallible;
use std::fmt;
use rxrust::prelude::*;

// ── Token ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Str(String),
    Number(f64, Option<String>),
    HashId(String),
    Ampersand,
    Dollar,
    Colon,
    Semicolon,
    Comma,
    Dot,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Eq,          // ==
    Ne,          // !=
    Le,          // <=
    Ge,          // >=
    Lt,          // <
    Gt,          // >
    Plus,
    Minus,
    Slash,
    Star,
    Percent,
    Bang,        // !
    Question,    // ?
    AtMedia,
    AtSupports,
    AtIf,
    AtElse,
    AtFor,
    AtEach,
    AtWhile,
    AtMixin,
    AtInclude,
    AtFunction,
    AtReturn,
    AtUse,
    AtForward,
    AtImport,
    AtExtend,
    AtAtRoot,
    AtContent,
    AtWarn,
    AtDebug,
    AtError,
    AtCharset,
    AtNamespace,
    AtKeyframes,
    AtFontFace,
    AtPage,
    AtCustomMedia,
    AtCustomSelector,
    InterpolationStart,
    InterpolationEnd,
    And,         // 'and' keyword
    Or,          // 'or' keyword
    AtName,
    IdentAt(String),
    Whitespace,
    Eof,
}

impl Token {
    pub fn pos(&self) -> u32 { 0 }
}

// ── Value ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64, Option<String>),
    String(String),
    Color(u8, u8, u8, u8),
    Calc(String),
    List(Vec<Value>),
    Map(Vec<(String, Value)>),
    Bool(bool),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n, unit) => {
                let num_str = if *n == n.trunc() {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                };
                match unit {
                    Some(u) => write!(f, "{}{}", num_str, u),
                    None => write!(f, "{}", num_str),
                }
            }
            Value::Calc(expr) => write!(f, "calc({})", expr),
            Value::String(s) => write!(f, "{}", s),
            Value::Color(r, g, b, a) => {
                if *a == 255 {
                    write!(f, "#{:02x}{:02x}{:02x}", r, g, b)
                } else {
                    write!(f, "#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
                }
            }
            Value::Bool(true) => write!(f, "true"),
            Value::Bool(false) => write!(f, "false"),
            Value::Null => write!(f, "null"),
            Value::List(items) => {
                let inner = items
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "{}", inner)
            }
            Value::Map(entries) => {
                let inner = entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "({})", inner)
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a, ua), Value::Number(b, ub)) => a == b && ua == ub,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Color(r1, g1, b1, a1), Value::Color(r2, g2, b2, a2)) => {
                r1 == r2 && g1 == g2 && b1 == b2 && a1 == a2
            }
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}

// ── CssStmt ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum CssStmt {
    Decl {
        property: String,
        value: String,
    },
    Rule {
        selector: String,
        inner: Vec<CssStmt>,
    },
    Media {
        query: String,
        inner: Vec<CssStmt>,
    },
    Supports {
        query: String,
        inner: Vec<CssStmt>,
    },
    Comment(String),
    Charset,
}

impl CssStmt {
    pub fn is_invisible(&self) -> bool {
        match self {
            CssStmt::Rule { inner, .. } => {
                inner.is_empty() || inner.iter().all(|s| s.is_invisible())
            }
            CssStmt::Media { inner, .. } => {
                inner.is_empty() || inner.iter().all(|s| s.is_invisible())
            }
            CssStmt::Supports { inner, .. } => {
                inner.is_empty() || inner.iter().all(|s| s.is_invisible())
            }
            _ => false,
        }
    }
}

// ── AstNode ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum AstNode {
    Literal(Value),
    VariableRef { name: String, scope_id: u64 },
    BinOp { op: BinOp, left: Box<AstNode>, right: Box<AstNode> },
    UnaryOp { op: UnaryOp, expr: Box<AstNode> },
    Interpolation(Vec<AstNode>),
    FunctionCall { name: String, args: Vec<AstNode> },
    ListLiteral(Vec<AstNode>),
    MapLiteral(Vec<(String, AstNode)>),
    VariableDecl { name: String, value: Box<AstNode>, scope_id: u64 },
    StyleDecl { property: String, value: Box<AstNode> },
    Rule { selector: String, inner: Vec<AstNode> },
    If { cond: Box<AstNode>, then_branch: Vec<AstNode>, else_branch: Option<Vec<AstNode>> },
    For { var: String, from: Box<AstNode>, to: Box<AstNode>, inclusive: bool, body: Vec<AstNode> },
    Each { vars: Vec<String>, list: Box<AstNode>, body: Vec<AstNode> },
    While { cond: Box<AstNode>, body: Vec<AstNode> },
    MixinDecl { name: String, params: Vec<Param>, body: Vec<AstNode> },
    MixinCall { name: String, args: Vec<AstNode>, content: Vec<AstNode> },
    FunctionDecl { name: String, params: Vec<Param>, body: Vec<AstNode> },
    Return(Box<AstNode>),
    Media { query: String, inner: Vec<AstNode> },
    Supports { query: String, inner: Vec<AstNode> },
    Warn(Box<AstNode>),
    Debug(Box<AstNode>),
    Css(CssStmt),
    Import(Vec<AstNode>),
    Content,
}

// ── Operators ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp { Add, Sub, Mul, Div, Mod, Eq, Ne, Lt, Gt, Le, Ge, And, Or }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp { Neg, Not }

// ── Param ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub default_value: Option<Box<AstNode>>,
}

impl Param {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), default_value: None }
    }
    pub fn with_default(name: impl Into<String>, default: AstNode) -> Self {
        Self { name: name.into(), default_value: Some(Box::new(default)) }
    }
}

// ── Aliases ───────────────────────────────────────────────────────────────

pub type TokenStream = SharedBoxedObservable<'static, Token, Infallible>;
pub type AstStream = SharedBoxedObservable<'static, AstNode, Infallible>;
pub type CssStream = SharedBoxedObservable<'static, CssStmt, Infallible>;
pub type OutputStream = SharedBoxedObservable<'static, String, Infallible>;

// ── Enums ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputSyntax {
    #[default]
    Scss,
    Css,
    Sass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputStyle {
    #[default]
    Expanded,
    Compressed,
    Nested,
}

// ── CompileError ───────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum CompileError {
    Io(String),
    Parse { pos: u32, message: String },
    Eval(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Io(msg) => write!(f, "IO error: {}", msg),
            CompileError::Parse { pos, message } => {
                write!(f, "Parse error at position {}: {}", pos, message)
            }
            CompileError::Eval(msg) => write!(f, "Eval error: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {}

impl From<std::io::Error> for CompileError {
    fn from(e: std::io::Error) -> Self {
        CompileError::Io(e.to_string())
    }
}
