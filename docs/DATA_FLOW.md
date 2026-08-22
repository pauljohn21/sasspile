# 数据流与类型定义

## 1. 管线类型状态机

每个阶段是一个 struct，携带该阶段的数据。阶段间通过 `impl TryFrom` 转换：

```rust
/// 源码文本。
pub(crate) struct Source {
    text: String,
    base_path: Option<PathBuf>,
    load_paths: Vec<PathBuf>,
}

/// 词法分析完成。
pub(crate) struct Lexed {
    tokens: Vec<Token>,
    base_path: Option<PathBuf>,
    load_paths: Vec<PathBuf>,
}

/// 语法分析完成。
pub(crate) struct Parsed {
    ast: Ast,
    base_path: Option<PathBuf>,
    load_paths: Vec<PathBuf>,
}

/// 求值完成。
pub(crate) struct Evaluated {
    nodes: Vec<CssNode>,
}

/// 序列化完成。
pub(crate) struct Serialized {
    css: String,
}
```

转换链：
```rust
impl TryFrom<Source> for Lexed { ... }   // lex()
impl TryFrom<Lexed> for Parsed { ... }   // parse()
impl TryFrom<Parsed> for Evaluated { ... } // evaluate()
impl TryFrom<Evaluated> for Serialized { ... } // serialize(style)
```

## 2. Token 定义

```rust
pub(crate) enum Token {
    // 字面量
    Ident(String),
    Number(f64, Option<String>),  // value, unit
    String(String, QuoteStyle),
    Color(u8, u8, u8, Option<f64>),  // r, g, b, alpha
    
    // 语法
    LBrace, RBrace,      // { }
    LParen, RParen,      // ( )
    LBracket, RBracket,  // [ ]
    Colon, Semicolon,    // : ;
    Comma, Dot,           // , .
    Hash,                 // #
    Dollar,               // $
    At,                   // @
    Ampersand,            // &
    
    // 运算符
    Plus, Minus, Star, Slash,  // + - * /
    Percent,                     // %
    Eq,                          // =
    Gt, Lt, Gte, Lte,           // > < >= <=
    Arrow,                       // =>
    
    // 特殊
    Interpolation(String),  // #{...}
    Comment(String),         // /* */
    SilentComment(String),   // //
    Eof,
}
```

## 3. AST Node 定义

```rust
pub(crate) enum Node {
    // CSS
    Rule { selector: String, body: Vec<Node> },
    Decl { property: String, value: Value, important: bool },
    Comment(String),
    
    // 变量
    Variable { name: String, value: Value, flags: VarFlags },
    
    // 控制流
    If { branches: Vec<(Value, Vec<Node>)>, else_body: Option<Vec<Node>> },
    For { var: String, from: Value, to: Value, inclusive: bool, body: Vec<Node> },
    Each { vars: Vec<String>, list: Value, body: Vec<Node> },
    While { cond: Value, body: Vec<Node> },
    
    // mixin/function
    MixinDef { name: String, params: Vec<Param>, body: Vec<Node> },
    FunctionDef { name: String, params: Vec<Param>, body: Vec<Node> },
    Include { name: String, args: Vec<Arg>, content: Option<Vec<Node>> },
    Content,
    Return(Value),
    
    // 模块系统
    Use { url: String, namespace: Option<String>, star: bool, config: Vec<(String, Value)> },
    Forward { url: String, show: Vec<String>, hide: Vec<String>, prefix: Option<String>, config: Vec<(String, Value)> },
    Import { url: String, modifier: Option<String> },
    
    // 其他
    AtRoot { query: Option<String>, body: Vec<Node> },
    AtRule { name: String, params: String, body: Option<Vec<Node>> },
    Extend { selector: String, optional: bool },
    Warn(Value),
    Debug(Value),
    Error(Value),
}
```

## 4. CssNode 定义

```rust
pub(crate) enum CssNode {
    Rule { selector: String, declarations: Vec<CssNode>, children: Vec<CssNode> },
    Declaration { property: String, value: String, important: bool },
    Comment(String),
    AtRule { name: String, params: String, children: Vec<CssNode>, has_body: bool },
    AtRoot(Vec<CssNode>),
    Return(Value),
}
```

## 5. Value 定义

```rust
pub(crate) enum Value {
    Number(f64, Option<String>),  // value, unit
    String(String, QuoteStyle),
    Color(Box<Color>),
    Bool(bool),
    Null,
    List(Vec<Value>, Separator, bool),  // items, separator, brackets
    Map(Vec<(Value, Value)>),
    Function(FunctionRef),
    ArgList(Vec<Value>),
}

pub(crate) struct Color {
    r: f64, g: f64, b: f64, a: f64,
    format: ColorFormat,
}
```

## 6. 后处理纯函数链

```rust
/// @extend 替换——纯函数，Vec -> Vec
pub(crate) fn apply_extends(nodes: Vec<CssNode>, extends: &[(String, String, bool)]) -> Vec<CssNode>;

/// CSS @import 提升——纯函数，Vec -> Vec
pub(crate) fn hoist_css_imports(nodes: Vec<CssNode>) -> Vec<CssNode>;

/// 检查 extend target 是否存在——只读
pub(crate) fn check_extend_targets(css: &[CssNode], extends: &[(String, String, bool)]) -> Result<()>;
```

调用链：
```rust
let css = Evaluator::evaluate(&ast, env)?;    // Vec<CssNode>
let css = apply_extends(css, &extends);         // Vec<CssNode>
check_extend_targets(&css, &extends)?;
let css = hoist_css_imports(css);                // Vec<CssNode>
let css = Serializer::serialize(css, style);   // Serialized
```
