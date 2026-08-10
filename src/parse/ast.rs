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
        star: bool,                   // as *
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
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 1.0,
        }
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
                if n.is_infinite() {
                    return write!(f, "calc(infinity)");
                }
                if n.is_nan() {
                    return write!(f, "calc(NaN)");
                }
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            Value::Number(n, Some(unit)) => {
                if n.is_infinite() {
                    return write!(f, "calc(infinity * 1{unit})");
                }
                if n.is_nan() {
                    return write!(f, "calc(NaN * 1{unit})");
                }
                if n.fract() == 0.0 {
                    write!(f, "{}{unit}", *n as i64)
                } else {
                    write!(f, "{n}{unit}")
                }
            }
            Value::String(s, true) => {
                let (quote, escaped) = Self::escape_quoted_string(s);
                write!(f, "{quote}{escaped}{quote}")
            }
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
                    if *bracketed {
                        return write!(f, "[]");
                    }
                    return write!(f, "");
                }
                let sep_str = match sep {
                    Separator::Comma => ", ",
                    Separator::Space => " ",
                    Separator::Slash => " / ",
                    Separator::Undecided => " ",
                };
                let parts: Vec<String> = elements.iter().map(|e| e.to_string()).collect();
                let inner = parts.join(sep_str);
                if *bracketed {
                    write!(f, "[{}]", inner)
                } else {
                    write!(f, "{}", inner)
                }
            }
            Value::Map(pairs) => {
                let parts: Vec<String> = pairs.iter().map(|(k, v)| format!("{k}: {v}")).collect();
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

impl Value {
    /// 转义引用字符串中的特殊字符为 CSS 转义序列。
    ///
    /// 返回 (quote_char, escaped_content)。
    /// - 如果字符串包含 `"` 但不包含 `'`，用单引号包裹，避免转义
    /// - 否则用双引号包裹，转义 `"`
    /// - `\` → `\\`
    /// - NULL (U+0000) → `\0 ` (with trailing space if needed)
    /// - 控制字符和私有区字符 → `\XXXX` (lowercase hex)
    /// - 其他非 ASCII 字符保持原样（会触发 @charset 前缀）
    fn escape_quoted_string(s: &str) -> (char, String) {
        let has_double = s.contains('"');
        let has_single = s.contains("'");
        // 如果包含双引号但不含单引号，用单引号包裹
        let quote = if has_double && !has_single { '\'' } else { '"' };

        let chars: Vec<char> = s.chars().collect();
        let mut result = String::new();
        for (i, &c) in chars.iter().enumerate() {
            match c {
                '\\' => result.push_str("\\\\"),
                '"' if quote == '"' => result.push_str("\\\""),
                '\'' if quote == '\'' => result.push_str("\\'"),
                '\0' => result.push_str("\\0 "),
                c if c.is_control() || ('\u{E000}'..='\u{F8FF}').contains(&c) => {
                    let hex = format!("{:x}", c as u32);
                    result.push('\\');
                    result.push_str(&hex);
                    // 仅在下一个字符是十六进制数字或空白时添加空格终止转义
                    let next = chars.get(i + 1).copied();
                    if next.is_some_and(|nc| nc.is_ascii_hexdigit() || nc.is_whitespace()) {
                        result.push(' ');
                    }
                }
                _ => result.push(c),
            }
        }
        (quote, result)
    }
}
