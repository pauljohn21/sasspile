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
        /// 选择器文本。
        selector: String,
        /// 规则体内的子节点列表。
        body: Vec<Node>,
    },
    /// 属性声明——`property: value;`。
    Decl {
        /// CSS 属性名。
        property: String,
        /// 属性值表达式。
        value: Value,
        /// 是否标记 `!important`。
        important: bool,
    },
    /// 变量声明——`$name: value;`。
    Variable {
        /// 变量名（不含 `$`）。
        name: String,
        /// 变量值表达式。
        value: Value,
        /// `!default` / `!global` 标志。
        flags: VarFlags,
    },
    /// 注释。
    Comment(String, bool), // (text, is_silent)

    // —— 控制流 ——
    /// `@if` / `@else if` / `@else` 条件分支。
    If {
        /// 条件分支列表，每项为 `(条件, 体)`。
        branches: Vec<(Value, Vec<Node>)>,
        /// `@else` 体（无条件分支）。
        else_body: Option<Vec<Node>>,
    },
    /// `@for` 循环。
    For {
        /// 循环变量名。
        var: String,
        /// 起始值表达式。
        from: Value,
        /// 结束值表达式。
        to: Value,
        /// `through` = true（含上界），`to` = false（不含）。
        inclusive: bool,
        /// 循环体节点列表。
        body: Vec<Node>,
    },
    /// `@each` 遍历。
    Each {
        /// 解构变量名列表。
        vars: Vec<String>,
        /// 待遍历的列表/Map 表达式。
        list: Value,
        /// 循环体节点列表。
        body: Vec<Node>,
    },
    /// `@while` 循环。
    While {
        /// 循环条件表达式。
        cond: Value,
        /// 循环体节点列表。
        body: Vec<Node>,
    },

    // —— Mixin / Function ——
    /// mixin 定义——`@mixin name(params) { ... }`。
    MixinDef {
        /// mixin 名称。
        name: String,
        /// 参数定义列表。
        params: Vec<Param>,
        /// mixin 体节点列表。
        body: Vec<Node>,
    },
    /// mixin 包含——`@include name(args)`。
    Include {
        /// 要包含的 mixin 名称。
        name: String,
        /// 调用参数列表。
        args: Vec<Arg>,
        /// `@content` 块内容。
        content: Option<Vec<Node>>,
    },
    /// `@content` 占位——mixin 体中标记调用者内容块插入位置。
    Content,
    /// 函数定义——`@function name(params) { ... }`。
    FunctionDef {
        /// 函数名称。
        name: String,
        /// 参数定义列表。
        params: Vec<Param>,
        /// 函数体节点列表。
        body: Vec<Node>,
    },
    /// `@return` 返回语句。
    Return(Value),

    // —— 模块系统 ——
    /// `@use` 模块加载。
    Use {
        /// 模块 URL。
        url: String,
        /// `as` 指定的命名空间。
        namespace: Option<String>,
        /// `as *` 通配导入标志。
        star: bool,
        /// `with ($x: val)` 配置参数列表。
        config: Vec<(String, Value)>,
    },
    /// `@forward` 模块转发。
    Forward {
        /// 要转发的模块 URL。
        url: String,
        /// `show` 白名单成员。
        show: Vec<String>,
        /// `hide` 黑名单成员。
        hide: Vec<String>,
        /// `as prefix-*` 前缀重映射。
        prefix: Option<String>,
    },
    /// `@import` 导入。
    Import {
        /// 要导入的文件 URL。
        url: String,
    },

    // —— 其他指令 ——
    /// `@extend` 继承。
    Extend {
        /// 要继承的选择器。
        selector: String,
        /// `!optional` 标志——不存在匹配时不报错。
        optional: bool,
    },
    /// `@at-root` 根级输出。
    AtRoot {
        /// 查询条件（如 `(without: media)`）。
        query: Option<String>,
        /// 体节点列表。
        body: Vec<Node>,
    },
    /// 通用 @规则。
    AtRule {
        /// @规则名称。
        name: String,
        /// @规则参数文本。
        params: Option<String>,
        /// @规则体（`None` 表示无 body）。
        body: Option<Vec<Node>>,
    },
    /// `@warn` 警告指令。
    Warn(Value),
    /// `@debug` 调试指令。
    Debug(Value),
    /// `@error` 错误指令。
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
    /// 运算符类型。
    pub op: BinOpKind,
    /// 左操作数。
    pub left: Value,
    /// 右操作数。
    pub right: Value,
}

/// 二元运算符类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOpKind {
    /// 加法 `+`。
    Add,
    /// 减法 `-`。
    Sub,
    /// 乘法 `*`。
    Mul,
    /// 除法 `/`。
    Div,
    /// 取模 `%`。
    Mod,
    /// 等于比较 `==`。
    Eq,
    /// 不等于比较 `!=`。
    NotEq,
    /// 小于比较 `<`。
    Lt,
    /// 大于比较 `>`。
    Gt,
    /// 小于等于比较 `<=`。
    LtEq,
    /// 大于等于比较 `>=`。
    GtEq,
    /// 逻辑与 `and`（短路求值）。
    And,
    /// 逻辑或 `or`（短路求值）。
    Or,
}

/// 一元运算。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    /// 一元负号 `-`。
    Neg,
    /// 逻辑非 `not`。
    Not,
}

/// 颜色。
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    /// 红色通道（0-255）。
    pub r: u8,
    /// 绿色通道（0-255）。
    pub g: u8,
    /// 蓝色通道（0-255）。
    pub b: u8,
    /// Alpha 通道（0.0-1.0）。
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
    /// 逗号分隔——`(a, b, c)`。
    Comma,
    /// 空格分隔——`(a b c)`。
    Space,
    /// 斜杠分隔——`(a / b / c)`。
    Slash,
    /// 未确定——单元素或待推断。
    Undecided,
}

/// AST 容器。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ast {
    /// 顶层语法树节点列表。
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
                    // 检查是否为命名颜色，优先输出名称（如 red 而非 #ff0000）
                    if let Some(name) = crate::eval::Evaluator::reverse_lookup_named_color(c) {
                        write!(f, "{name}")
                    } else {
                        write!(f, "#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
                    }
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
