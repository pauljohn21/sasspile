//! AST 定义——语法分析器的产出。

/// 变量标志——`!default`、`!global`。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VarFlags {
    /// `!default`——仅未定义时赋值。
    pub default: bool,
    /// `!global`——写入全局作用域。
    pub global: bool,
}

/// 函数/参数定义。
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// 参数名（不含 $）。
    pub name: String,
    /// 默认值。
    pub default: Option<Value>,
    /// 是否为剩余参数（`...`）。
    pub rest: bool,
}

/// 函数调用参数。
#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    /// 位置参数或关键字参数名。
    pub name: Option<String>,
    /// 参数值。
    pub value: Value,
    /// 是否展开剩余参数。
    pub spread: bool,
}

/// 语法树节点。
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// 样式规则——`selector { ... }`。
    Rule {
        selector: String,
        body: Vec<Node>,
    },
    /// 属性声明——`property: value;`。
    Decl {
        property: String,
        value: Value,
        important: bool,
    },
    /// 变量声明——`$name: value;`。
    Variable {
        name: String,
        value: Value,
        flags: VarFlags,
    },
    /// 注释。
    Comment(String, bool), // (text, is_silent)

    // —— 控制流 ——
    If {
        branches: Vec<(Value, Vec<Node>)>, // (condition, body)
        else_body: Option<Vec<Node>>,
    },
    For {
        var: String,
        from: Value,
        to: Value,
        inclusive: bool, // through=true, to=false
        body: Vec<Node>,
    },
    Each {
        vars: Vec<String>,
        list: Value,
        body: Vec<Node>,
    },
    While {
        cond: Value,
        body: Vec<Node>,
    },

    // —— Mixin / Function ——
    MixinDef {
        name: String,
        params: Vec<Param>,
        body: Vec<Node>,
    },
    Include {
        name: String,
        args: Vec<Arg>,
        content: Option<Vec<Node>>,
    },
    Content,
    FunctionDef {
        name: String,
        params: Vec<Param>,
        body: Vec<Node>,
    },
    Return(Value),

    // —— 模块系统 ——
    Use {
        url: String,
        namespace: Option<String>,
        star: bool,       // as *
        config: Vec<(String, Value)>, // with ($x: val)
    },
    Forward {
        url: String,
        show: Vec<String>,
        hide: Vec<String>,
        prefix: Option<String>, // as prefix-*
    },
    Import {
        url: String,
    },

    // —— 其他指令 ——
    Extend {
        selector: String,
        optional: bool,
    },
    AtRoot {
        query: Option<String>,
        body: Vec<Node>,
    },
    AtRule {
        name: String,
        params: Option<String>,
        body: Option<Vec<Node>>,
    },
    Warn(Value),
    Debug(Value),
    Error(Value),
}

/// 值表达式。
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// 数值——`16px`, `3.14`。
    Number(f64, Option<String>),
    /// 字符串——`red`, `"hello"`。bool=是否有引号。
    String(String, bool),
    /// 颜色。
    Color(Color),
    /// 列表。
    List(Vec<Value>, Separator, bool),
    /// Map。
    Map(Vec<(Value, Value)>),
    /// 变量引用。
    Variable(String),
    /// 布尔值。
    Bool(bool),
    /// null。
    Null,
    /// 函数调用——`name(args)`。
    Call(String, Vec<Arg>),
    /// 插值——`#{...}`。
    Interp(String),
    /// 二元运算——`left op right`。
    BinOp(Box<BinOp>),
    /// 一元运算——`op operand`。
    UnaryOp(UnaryOp, Box<Value>),
    /// calc() 原样保留。
    Calc(String),
    /// 剩余参数展开。
    Spread(Box<Value>),
}

/// 二元运算。
#[derive(Debug, Clone, PartialEq)]
pub struct BinOp {
    pub op: BinOpKind,
    pub left: Value,
    pub right: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

/// 一元运算。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// 颜色。
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self { r: 0, g: 0, b: 0, a: 1.0 }
    }
}

impl Color {
    /// 创建 RGB 颜色。
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 1.0 }
    }
    /// 创建 RGBA 颜色。
    pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

/// 列表分隔符。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Separator {
    Comma,
    Space,
    Slash,
    Undecided,
}

/// AST 容器。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ast {
    pub nodes: Vec<Node>,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
Value::Number(n, None) => {
if n.is_infinite() { return write!(f, "calc(infinity)"); }
if n.is_nan() { return write!(f, "calc(NaN)"); }
if (n.fract() == 0.0) {
write!(f, "{}", *n as i64)
} else {
write!(f, "{n}")
}
}
Value::Number(n, Some(unit)) => {
if n.is_infinite() { return write!(f, "calc(infinity * 1{unit})"); }
if n.is_nan() { return write!(f, "calc(NaN * 1{unit})"); }
if (n.fract() == 0.0) {
write!(f, "{}{unit}", *n as i64)
} else {
write!(f, "{n}{unit}")
}
}
            Value::String(s, true) => write!(f, "\"{s}\""),
            Value::String(s, false) => write!(f, "{s}"),
            Value::Color(c) => {
                if (c.a - 1.0).abs() < f32::EPSILON {
                    write!(f, "#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
                } else {
                    write!(f, "rgba({}, {}, {}, {})", c.r, c.g, c.b, c.a)
                }
            }
            Value::List(elements, sep, bracketed) => {
                if elements.is_empty() {
                    if *bracketed { return write!(f, "[]"); }
                    return write!(f, "");
                }
                let sep_str = match sep {
                    Separator::Comma => ", ",
                    Separator::Space => " ",
                    Separator::Slash => "/",
                    Separator::Undecided => " ",
                };
                let parts: Vec<String> = elements.iter().map(|e| e.to_string()).collect();
                let inner = parts.join(sep_str);
                if *bracketed { write!(f, "[{}]", inner) } else { write!(f, "{}", inner) }
            }
            Value::Map(pairs) => {
                let parts: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect();
                write!(f, "({})", parts.join(", "))
            }
            Value::Variable(name) => write!(f, "${name}"),
            Value::Bool(true) => write!(f, "true"),
            Value::Bool(false) => write!(f, "false"),
            Value::Null => write!(f, "null"),
            Value::Call(name, args) => {
                let parts: Vec<String> = args.iter().map(|a| a.value.to_string()).collect();
                write!(f, "{}({})", name, parts.join(", "))
            }
            Value::Interp(s) => write!(f, "#{{{s}}}"),
            Value::BinOp(b) => write!(f, "{}", b.left),
            Value::UnaryOp(op, v) => match op {
                UnaryOp::Neg => write!(f, "-{v}"),
                UnaryOp::Not => write!(f, "not {v}"),
            },
            Value::Calc(s) => write!(f, "{s}"),
            Value::Spread(v) => write!(f, "{v}..."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_display() {
        assert_eq!(Value::Number(10.0, None).to_string(), "10");
        assert_eq!(Value::Number(3.14, None).to_string(), "3.14");
        assert_eq!(Value::Number(10.0, Some("px".into())).to_string(), "10px");
        assert_eq!(Value::Number(50.0, Some("%".into())).to_string(), "50%");
    }

    #[test]
    fn test_string_display() {
        assert_eq!(Value::String("red".into(), false).to_string(), "red");
        assert_eq!(Value::String("hello".into(), true).to_string(), "\"hello\"");
    }

    #[test]
    fn test_color_display() {
        assert_eq!(Value::Color(Color::rgb(255, 0, 0)).to_string(), "#ff0000");
        assert_eq!(Value::Color(Color::rgb(0, 0, 0)).to_string(), "#000000");
        assert_eq!(Value::Color(Color::rgba(0, 0, 0, 0.5)).to_string(), "rgba(0, 0, 0, 0.5)");
    }

    #[test]
    fn test_list_display() {
let list = Value::List(vec![
Value::Number(1.0, None),
Value::Number(2.0, None),
Value::Number(3.0, None),
], Separator::Comma, false);
        assert_eq!(list.to_string(), "1, 2, 3");

let space_list = Value::List(vec![
Value::String("a".into(), false),
Value::String("b".into(), false),
], Separator::Space, false);
        assert_eq!(space_list.to_string(), "a b");
    }

    #[test]
    fn test_map_display() {
        let map = Value::Map(vec![
            (Value::String("a".into(), false), Value::Number(1.0, None)),
            (Value::String("b".into(), false), Value::Number(2.0, None)),
        ]);
        assert_eq!(map.to_string(), "(a: 1, b: 2)");
    }

    #[test]
    fn test_bool_null_display() {
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
        assert_eq!(Value::Null.to_string(), "null");
    }

    #[test]
    fn test_color_rgb() {
        let c = Color::rgb(255, 128, 0);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
        assert!((c.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_color_rgba() {
        let c = Color::rgba(0, 0, 0, 0.5);
        assert_eq!(c.a, 0.5);
    }
}

/// AST 节点序列化——用于最小化工具将 AST 转回 SCSS 源码。
impl Node {
    /// 将 AST 节点序列化回 SCSS 源码——用于最小化工具。
    pub fn to_scss(&self, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        match self {
            Node::Rule { selector, body } => {
                let body: String = body.iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                if body.is_empty() {
                    format!("{pad}{selector} {{}}")
                } else {
                    format!("{pad}{selector} {{\n{body}\n{pad}}}")
                }
            }
            Node::Decl { property, value, important } => {
                let imp = if *important { " !important" } else { "" };
                format!("{pad}{property}: {value}{imp};")
            }
            Node::Variable { name, value, flags } => {
                let mut s = format!("{pad}${name}: {value}");
                if flags.default { s.push_str(" !default"); }
                if flags.global { s.push_str(" !global"); }
                s.push(';');
                s
            }
            Node::Comment(text, silent) => {
                if *silent { format!("{pad}// {text}") }
                else { format!("{pad}/* {text} */") }
            }
            // —— 以下变体在任务 3 实现 ——
            _ => format!("{pad}/* TODO: to_scss for this variant */"),
        }
    }
}

#[cfg(test)]
mod to_scss_tests {
    use super::*;

    #[test]
    fn test_rule_to_scss() {
        let node = Node::Rule {
            selector: "a".into(),
            body: vec![Node::Decl {
                property: "color".into(),
                value: Value::String("red".into(), false),
                important: false,
            }],
        };
        let scss = node.to_scss(0);
        assert!(scss.contains("a {"));
        assert!(scss.contains("color: red;"));
        assert!(scss.contains("}"));
    }

    #[test]
    fn test_decl_to_scss() {
        let node = Node::Decl {
            property: "width".into(),
            value: Value::Number(100.0, Some("px".into())),
            important: true,
        };
        let scss = node.to_scss(0);
        assert_eq!(scss, "width: 100px !important;");
    }

    #[test]
    fn test_variable_to_scss() {
        let node = Node::Variable {
            name: "color".into(),
            value: Value::String("blue".into(), false),
            flags: VarFlags { default: true, global: false },
        };
        let scss = node.to_scss(0);
        assert_eq!(scss, "$color: blue !default;");
    }

    #[test]
    fn test_comment_to_scss() {
        let silent = Node::Comment("hello".into(), true);
        let loud = Node::Comment("world".into(), false);
        assert_eq!(silent.to_scss(0), "// hello");
        assert_eq!(loud.to_scss(0), "/* world */");
    }
}