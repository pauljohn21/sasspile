//! AST 定义——枚举基抽象语法树。
//!
//! AST 节点分为：
//! - `Node`: 顶层节点（规则、声明、变量、@规则）
//! - `Value`: 值表达式（数字、字符串、颜色、列表等）
//! - `Color`: 颜色值
//! - `Separator`: 列表分隔符
//! - `Ast`: AST 容器

/// 语法树节点。
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// 样式规则——`selector { ... }`。
    Rule {
        /// 选择器文本。
        selector: String,
        /// 子节点。
        body: Vec<Node>,
    },

    /// 属性声明——`property: value;`。
    Decl {
        /// 属性名。
        property: String,
        /// 属性值。
        value: Value,
        /// 是否 important。
        important: bool,
    },

    /// 变量声明——`$name: value;`。
    Variable {
        /// 变量名。
        name: String,
        /// 初始值。
        value: Value,
    },

    /// @规则——`@media`, `@mixin` 等。
    AtRule {
        /// 规则名。
        name: String,
        /// 参数。
        params: Option<String>,
        /// 体。
        body: Option<Vec<Node>>,
    },

    /// 注释。
    Comment(String),
}

/// 值表达式。
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// 数值——`16px`, `3.14`。
    Number(f64, Option<String>),

    /// 字符串——`red`, `"hello"`。
    String(String, bool),

    /// 颜色。
    Color(Color),

    /// 列表——`(1, 2, 3)`。
    List(Vec<Value>, Separator),

    /// 变量引用。
    Variable(String),

    /// 布尔值。
    Bool(bool),

    /// 函数调用——`name(args)`。
    Call(String, Vec<Value>),
}

/// 颜色。
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    /// 红。
    pub r: u8,
    /// 绿。
    pub g: u8,
    /// 蓝。
    pub b: u8,
    /// 透明度。
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 1.0,
        }
    }
}

/// 列表分隔符。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Separator {
    /// 逗号分隔。
    Comma,
    /// 空格分隔。
    Space,
    /// 斜杠分隔。
    Slash,
}

/// AST 容器。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ast {
    /// 顶层节点序列。
    pub nodes: Vec<Node>,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n, None) => write!(f, "{n}"),
            Value::Number(n, Some(unit)) => write!(f, "{n}{unit}"),
            Value::String(s, true) => write!(f, "{s:?}"),
            Value::String(s, false) => write!(f, "{s}"),
            Value::Color(c) => write!(f, "rgba({}, {}, {}, {})", c.r, c.g, c.b, c.a),
            Value::List(elements, _) => {
                let parts: Vec<String> = elements.iter().map(|e| e.to_string()).collect();
                write!(f, "{}", parts.join(" "))
            }
            Value::Variable(name) => write!(f, "${name}"),
            Value::Bool(true) => write!(f, "true"),
            Value::Bool(false) => write!(f, "false"),
            Value::Call(name, args) => {
                let parts: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                write!(f, "{}({})", name, parts.join(", "))
            }
        }
    }
}
