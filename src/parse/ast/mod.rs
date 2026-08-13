//! AST 定义——语法分析器的产出。

mod display;

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
    /// `if()` 冒号语法的条件表达式——`if(condition: value; else: other)`。
    /// 当此字段有值时，`value` 是条件为真时的返回值，`name` 为 `Some("else")` 表示 else 分支。
    pub condition: Option<Value>,
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
        /// 要导入的文件 URL 列表（支持逗号分隔的 CSS @import）。
        urls: Vec<String>,
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
    /// 括号表达式——保留括号用于 CSS 透传。
    Paren(Box<Value>),
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

/// 颜色格式——追踪颜色创建方式，影响序列化输出。
#[derive(Debug, Clone, Default)]
pub enum ColorFormat {
    /// 自动：hex / 命名颜色 / rgba（默认行为）。
    #[default]
    Auto,
    /// rgb(r, g, b) / rgba(r, g, b, a)——不转 hex 或命名。
    Rgb,
    /// hsl(h, s%, l%) / hsla(h, s%, l%, a)——存储原始 HSL 值 (h: 0-360, s/l: 0-1)。
    Hsl(f64, f64, f64),
    /// hwb(h w% b% / a)——存储原始 HWB 值 (h: 0-360, w/b: 0-1)。
    Hwb(f64, f64, f64),
}

/// 颜色。
#[derive(Debug, Clone)]
pub struct Color {
    /// 红色通道（0-255）。
    pub r: u8,
    /// 绿色通道（0-255）。
    pub g: u8,
    /// 蓝色通道（0-255）。
    pub b: u8,
    /// Alpha 通道（0.0-1.0）。
    pub a: f64,
    /// 颜色格式（追踪创建方式）。
    pub format: ColorFormat,
}

/// 颜色相等性仅比较 RGBA 值，忽略格式。
impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b && self.a == other.a
    }
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 1.0,
            format: ColorFormat::Auto,
        }
    }
}

impl Color {
    /// 创建 RGB 颜色。
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 1.0, format: ColorFormat::Auto }
    }
    /// 创建 RGBA 颜色。
    pub fn rgba(r: u8, g: u8, b: u8, a: f64) -> Self {
        Self { r, g, b, a, format: ColorFormat::Auto }
    }
    /// 创建带格式的 RGB 颜色。
    pub fn rgb_fmt(r: u8, g: u8, b: u8, format: ColorFormat) -> Self {
        Self { r, g, b, a: 1.0, format }
    }
    /// 创建带格式的 RGBA 颜色。
    pub fn rgba_fmt(r: u8, g: u8, b: u8, a: f64, format: ColorFormat) -> Self {
        Self { r, g, b, a, format }
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

/// 格式化 hue 值——整数不带小数点，否则保留原始精度。
fn format_hue(h: f64) -> String {
    if h.fract() == 0.0 {
        format!("{}", h as i64)
    } else {
        format!("{h}")
    }
}

/// 格式化百分比值（0.0-1.0 → 0%-100%），浮点精度截断到 11 位小数。
fn format_pct(v: f64) -> String {
    let pct = v * 100.0;
    // 修复浮点精度问题（如 60.00000000000001 → 60）
    let pct = (pct * 1e10).round() / 1e10;
    if pct.fract() == 0.0 {
        format!("{}", pct as i64)
    } else {
        format!("{pct}")
    }
}

/// 格式化 alpha 值。
fn format_alpha(a: f64) -> String {
    if a.fract() == 0.0 {
        format!("{}", a as i64)
    } else {
        let s = format!("{a}");
        s
    }
}
