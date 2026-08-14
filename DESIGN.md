# sasspile 2.0 —— 纯 Rust 函数式设计

> 从零重写，以学习 Rust 函数式编程为核心目标。
> 类型安全 + 不可变数据 + 迭代器管线 + 阶段编码。

---

## 1. 设计哲学

```
┌─────────────────────────────────────────────────────────┐
│                   核心原则                               │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  1. 阶段即类型 ── 编译阶段编码在类型系统中               │
│     Source → Lexed → Parsed → Evaluated → Serialized    │
│                                                         │
│  2. 数据不可变 ── 转换产生新值，不修改旧值               │
│     fold 替代 loop + mutation                           │
│                                                         │
│  3. 函数为一等公民 ── 阶段转换是纯函数                   │
│     fn transform(input: PhaseA) -> Result<PhaseB>       │
│                                                         │
│  4. 迭代器为核心 ── 所有集合处理用 Iterator              │
│     .map().filter().fold().try_collect()                 │
│                                                         │
│  5. 错误即值 ── Result + ? 替代异常                     │
│     错误类型用 enum，传播用 ?                            │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## 2. 编译管线（类型状态机）

```
                         类型状态转换
    ════════════════════════════════════════════

    ┌────────┐    lex()    ┌────────┐   parse()   ┌────────┐
    │ Source │ ──────────▶ │ Lexed  │ ──────────▶ │ Parsed │
    └────────┘             └────────┘             └────────┘
      String                  Vec<Token>            Ast
                                                       │
                                                       │ evaluate()
                                                       ▼
    ┌──────────┐  serialize()  ┌──────────┐
    │Serialized│ ◀──────────── │Evaluated │
    └──────────┘               └──────────┘
      String                     Vec<CssNode>

    类型保证：不可能在 Lexed 之前调用 parse()
    类型保证：不可能在 Parsed 之前调用 evaluate()
```

### 关键设计

```rust
// 每个阶段是新类型，不是 type alias
pub struct Source { text: String }        // 原始源码
pub struct Lexed { tokens: Vec<Token> }   // token 流
pub struct Parsed { ast: Ast }            // 抽象语法树
pub struct Evaluated { nodes: Vec<CssNode> } // CSS 节点
pub struct Serialized { css: String }     // 最终 CSS

// 阶段转换 trait（可扩展）
trait StageInput { type Output; }
impl StageInput for Source { type Output = Lexed; }
// ...
```

---

## 3. 模块结构（当前实现）

```
sasspile/
├── Cargo.toml
├── DESIGN.md                    ← 你正在读的文档
├── src/
│   ├── lib.rs                   (405) ── 公共 API（管线入口）+ init_tracing
│   ├── main.rs                  (49)  ── CLI 入口
│   ├── error.rs                 (95)  ── 统一错误类型 (SassError)
│   │
│   ├── lex/                     ── 词法分析
│   │   ├── mod.rs               (499) ── Lexer + Iterator impl（scan_* 方法）
│   │   └── token.rs             (170) ── Token 枚举定义 + Display impl
│   │
│   ├── parse/                   ── 语法分析（Pratt + 递归下降）
│   │   ├── mod.rs               (102) ── Parser 结构 + parse() 入口 + paren_depth
│   │   ├── ast/                 ── AST 类型定义
│   │   │   ├── mod.rs           (420) ── Node, Value, Color, BinOp, Separator 等
│   │   │   └── display.rs       (348) ── Display trait + escape 函数 + round_alpha
│   │   ├── ast_impl.rs          (289) ── Node::to_scss() 实现
│   │   ├── at_rules.rs          (536) ── 所有 @ 规则解析
│   │   ├── nodes.rs             (594) ── parse_node/parse_rule/parse_decl/parse_body
│   │   └── expr/                ── 表达式解析
│   │       ├── mod.rs           (328) ── Pratt 解析 + has_other_operator_at_top_level
│   │       └── prefix.rs        (512) ── parse_number/parse_hash_color
│   │
│   ├── eval/                    ── 求值器
│   │   ├── mod.rs               (526) ── Env + Evaluator + evaluate/eval_nodes
│   │   ├── rule.rs              (169) ── eval_rule + combine_selectors
│   │   ├── value/               ── 值求值
│   │   │   ├── mod.rs           (449) ── eval_value + eval_interp_str + eval_simple_expr
│   │   │   ├── ops.rs           (290) ── add/sub/mul/div/modulo/compare
│   │   │   └── display.rs       (186) ── inspect_value + 值格式化
│   │   ├── control_flow.rs      (150) ── eval_if/eval_for/eval_each/eval_while
│   │   ├── mixin.rs             (264) ── eval_include + bind_params + call_function
│   │   ├── extend.rs            (76)  ── apply_extends
│   │   ├── module.rs            (302) ── resolve_file + load_module + call_module_function
│   │   ├── color.rs             (621) ── hsl_to_rgb/hwb_to_rgb + builtin_rgba/darken/lighten/mix
│   │   ├── builtin.rs           (497) ── call_builtin 分派入口
│   │   ├── builtin/             ── 内建函数按类别分文件
│   │   │   ├── color.rs         (553) ── 颜色函数（invert/hsl/hwb/adjust-color/...）
│   │   │   ├── list.rs          (282) ── 列表函数（length/nth/append/join/...）
│   │   │   ├── map.rs           (302) ── 映射函数（map-get/map-merge/...）
│   │   │   ├── string.rs        (281) ── 字符串函数（str-length/str-slice/...）
│   │   │   └── selector.rs      (156) ── 选择器函数
│   │   ├── selector/            ── 选择器操作
│   │   │   ├── parse.rs         ── 选择器解析为结构化表示
│   │   │   └── algorithms.rs    ── 选择器算法（matches/unify/extend）
│   │   └── memory_limit.rs      (92)  ── 内存限制器（链式反应设计）
│   │
│   ├── css/                     ── CSS 序列化
│   │   ├── mod.rs               (350) ── Serializer（选择器净化 + @规则合并）
│   │   └── node.rs              (93)  ── CssNode 枚举
│   │
│   └── stage/                   ── 管线阶段类型（轻量包装）
│       ├── mod.rs               ── Stage trait
│       ├── source.rs            ── Source 类型
│       ├── lexed.rs             ── Lexed 类型
│       ├── parsed.rs            ── Parsed 类型
│       ├── evaluated.rs         ── Evaluated 类型
│       └── serialized.rs        ── Serialized 类型
```

---

## 4. 核心类型定义

### 4.1 Token

```rust
/// 词法单元——编译管线的第一个产出。
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // 字面量
    Ident(String),          // 标识符
    Number(f64, Option<String>),  // 数字 + 可选单位
    String(String, bool),   // 字符串 + 是否引号
    Color(u32),             // 十六进制颜色
    True, False, Null,      // 字面量
    And, Or, Not,           // 逻辑运算符

    // 符号
    LParen, RParen,         // ( )
    LBrace, RBrace,         // { }
    LBracket, RBracket,     // [ ]
    Colon, Semicolon,       // : ;
    Comma, Dot,             // , .
    Plus, Minus, Star, Slash, Percent,
    Amp,                    // & (父选择器引用)
    Caret, Tilde, Bang,     // ^ ~ !
    Assign,                 // =
    Eq, NotEq,              // == !=
    Less, Greater,          // < >
    LessEq, GreaterEq,      // <= >=
    DotDotDot,              // ... (展开)
    Pipe,                   // | (命名空间)

    // 特殊
    AtKeyword(String),      // @use, @mixin 等
    Dollar(String),         // $variable（字段为变量名）
    Eof,
}

impl Token {
    /// 判断是否为空白 token。
    pub fn is_trivia(&self) -> bool {
        matches!(self, Token::Whitespace)
    }
}
```

### 4.2 AST

```rust
/// 抽象语法树节点。
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// 样式规则：selector { ... }
    Rule { selector: String, body: Vec<Node> },
    /// 声明：property: value;
    Decl { property: String, value: Value, important: bool },
    /// 变量声明：$name: value;
    Variable { name: String, value: Value, flags: VarFlags },
    /// @if / @else if / @else
    If { branches: Vec<(Value, Vec<Node>)>, else_body: Option<Vec<Node>> },
    /// @for 循环
    For { var: String, from: Value, to: Value, inclusive: bool, body: Vec<Node> },
    /// @each 循环
    Each { vars: Vec<String>, list: Value, body: Vec<Node> },
    /// @while 循环
    While { cond: Value, body: Vec<Node> },
    /// @mixin 定义
    MixinDef { name: String, params: Vec<Param>, body: Vec<Node> },
    /// @include 调用
    Include { name: String, args: Vec<Arg>, content: Option<Vec<Node>> },
    /// @function 定义
    FunctionDef { name: String, params: Vec<Param>, body: Vec<Node> },
    /// @use 模块加载
    Use { url: String, namespace: Option<String>, star: bool, config: Vec<(String, Value)> },
    /// @forward 模块转发
    Forward { url: String, show: Vec<String>, hide: Vec<String>, prefix: Option<String> },
    /// @import
    Import { url: String },
    /// @extend
    Extend { selector: String, optional: bool },
    /// @at-root
    AtRoot { query: Option<String>, body: Vec<Node> },
    /// 通用 @规则
    AtRule { name: String, params: Option<String>, body: Option<Vec<Node>> },
    /// @content 占位
    Content,
    /// @return
    Return(Value),
    /// @warn / @debug / @error
    Warn(Value), Debug(Value), Error(Value),
    /// 注释
    Comment(String, bool),
}

/// 值表达式。
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64, Option<String>),      // 值 + 单位
    String(String, bool),             // 内容 + 是否引号
    Color(Color),
    List(Vec<Value>, Separator, bool), // 元素 + 分隔符 + 是否含括号
    Map(Vec<(Value, Value)>),         // 键值对列表
    Variable(String),                 // 变量引用
    Bool(bool),
    Null,
    Call(String, Vec<Arg>),           // 函数调用
    Interp(String),                   // 插值 #{...}
    BinOp(Box<BinOp>),               // 二元运算
    UnaryOp(UnaryOp, Box<Value>),    // 一元运算
    Calc(String),                     // calc() 原样保留
    Spread(Box<Value>),              // ... 展开
    Raw(String),                      // 原始内容（不转义）
    Identifier(String),              // 标识符
}

#[derive(Debug, Clone, PartialEq)]
pub enum Separator { Comma, Space, Slash, SlashDiv, Undecided }

#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub r: u8, pub g: u8, pub b: u8, pub a: f64,
    pub format: ColorFormat,
}

/// AST 容器。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ast {
    pub nodes: Vec<Node>,
}
```

### 4.3 CssNode

```rust
/// CSS 中间表示——求值阶段的产出。
#[derive(Debug, Clone, PartialEq)]
pub enum CssNode {
    Rule {
        selector: String,
        declarations: Vec<CssNode>,
        children: Vec<CssNode>,
    },
    Declaration {
        property: String,
        value: String,
        important: bool,
    },
    AtRule {
        name: String,
        params: Option<String>,
        children: Vec<CssNode>,
        has_body: bool,
    },
    Comment(String),
    AtRoot(Vec<CssNode>),
    Raw(String),
    Return(Value),  // 不序列化，仅内部传播
}
```

---

## 5. 阶段实现细节

### 5.1 Lexer —— 迭代器风格

```rust
/// 词法分析器——将源码字符串转为 Token 流。
pub struct Lexer<'src> {
    source: &'src str,
    chars: std::str::Chars<'src>,
    pos: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            chars: source.chars(),
            pos: 0,
        }
    }
}

/// Lexer 自身就是迭代器！
impl<'src> Iterator for Lexer<'src> {
    type Item = Result<Token, SassError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        let c = self.peek()?;
        Some(match c {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.scan_ident(),
            b'0'..=b'9' => self.scan_number(),
            b'{' => self.single(Token::LBrace),
            b'}' => self.single(Token::RBrace),
            b'(' => self.single(Token::LParen),
            b')' => self.single(Token::RParen),
            b':' => self.single(Token::Colon),
            b';' => self.single(Token::Semicolon),
            b',' => self.single(Token::Comma),
            b'.' => self.scan_dot(),
            b'$' => self.scan_variable(),
            b'@' => self.scan_at(),
            b'#' => self.scan_hash(),
            b'"' | b'\'' => self.scan_string(c),
            b'/' => self.scan_slash(),
            _ => Err(...),
        })
    }
}

// 使用：Source → Lexed
impl Source {
    pub fn lex(self) -> Result<Lexed, SassError> {
        let tokens = Lexer::new(&self.text)
            .filter(|t| !t.as_ref().is_ok_and(Token::is_trivia))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Lexed { tokens })
    }
}
```

### 5.2 Parser —— 递归下降 + Result 组合

```rust
/// 语法分析器——将 Token 流转为 AST。
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// 解析入口。
    pub fn parse(mut self) -> Result<Ast, SassError> {
        let mut nodes = Vec::new();
        while !self.at_end() {
            nodes.push(self.parse_node()?);
        }
        Ok(Ast { nodes })
    }

    fn parse_node(&mut self) -> Result<Node, SassError> {
        match self.peek() {
            Token::AtIdent(name) => self.parse_at_rule(&name),
            _ => self.parse_rule_or_decl(),
        }
    }

    // ? 操作符自动传播错误
    fn parse_rule_or_decl(&mut self) -> Result<Node, SassError> {
        let selector = self.parse_selector()?;
        self.expect(Token::LBrace)?;
        let body = self.parse_body()?;
        self.expect(Token::RBrace)?;
        Ok(Node::Rule { selector, body })
    }
}

// Lexed → Parsed
impl Lexed {
    pub fn parse(self) -> Result<Parsed, SassError> {
        Parser::new(self.tokens).parse().map(|ast| Parsed { ast })
    }
}
```

### 5.3 Evaluator —— fold + 不可变环境

```rust
/// 求值环境——可变，但通过 clone 实现作用域隔离。
#[derive(Debug, Clone)]
pub struct Env {
    vars: HashMap<String, Value>,
    mixins: HashMap<String, MixinDef>,
    functions: HashMap<String, FunctionDef>,
    namespaces: HashMap<String, ModuleExports>,
    /// 当前深度——防止循环导入栈溢出。
    pub(crate) depth: usize,
    /// 基准路径——用于相对 @import 解析。
    pub(crate) base_path: Option<PathBuf>,
    /// plain CSS 模式——嵌套规则不展开。
    pub(crate) plain_css: bool,
    /// 加载路径列表——用于 @use/@import 文件查找。
    pub(crate) load_paths: Vec<PathBuf>,
}

impl Env {
    pub(crate) fn new_env() -> Self { /* ... */ }
    pub(crate) fn bind(&mut self, name: String, value: Value) { /* ... */ }
    pub(crate) fn lookup(&self, name: &str) -> Option<&Value> { /* ... */ }
    pub(crate) fn has_var(&self, name: &str) -> bool { /* ... */ }
}

/// 求值器——fold 替代循环。
pub struct Evaluator {
    env: Env,
}

impl Evaluator {
    pub fn new() -> Self {
        Self { env: Env::new() }
    }

    /// 对 AST 的每个节点 fold。
    pub fn evaluate(mut self, ast: &Ast) -> Result<Vec<CssNode>, SassError> {
        ast.nodes
            .iter()
            .try_fold(Vec::new(), |mut acc, node| {
                let mut out = self.eval_node(node)?;
                acc.append(&mut out);
                Ok(acc)
            })
    }

    fn eval_node(&mut self, node: &Node) -> Result<Vec<CssNode>, SassError> {
        match node {
            Node::Rule { selector, body } => self.eval_rule(selector, body),
            Node::Decl { property, value, important } => {
                let val = self.eval_value(value)?;
                Ok(vec![CssNode::Declaration {
                    property: property.clone(),
                    value: val,
                    important: *important,
                }])
            }
            Node::Variable { name, value } => {
                let val = self.eval_value(value)?;
                self.env = self.env.bind(name.clone(), val);
                Ok(vec![])
            }
            Node::AtRule { name, params, body } => {
                self.eval_at_rule(name, params, body)
            }
            Node::Comment(text) => {
                Ok(vec![CssNode::Comment(text.clone())])
            }
        }
    }
}

// Parsed → Evaluated
impl Parsed {
    pub fn evaluate(self) -> Result<Evaluated, SassError> {
        Evaluator::new()
            .evaluate(&self.ast)
            .map(|nodes| Evaluated { nodes })
    }
}
```

### 5.4 Serializer —— Iterator 链

```rust
/// 序列化器——CssNode 树 → CSS 字符串。
pub struct Serializer {
    style: OutputStyle,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputStyle { Expanded, Compressed }

impl Serializer {
    pub fn new(style: OutputStyle) -> Self { Self { style } }

    pub fn serialize(&self, nodes: &[CssNode]) -> String {
        nodes.iter()
            .map(|node| self.serialize_node(node, 0))
            .collect::<Vec<_>>()
            .join(if matches!(self.style, OutputStyle::Compressed) { "" } else { "\n" })
    }

    fn serialize_node(&self, node: &CssNode, depth: usize) -> String {
        match node {
            CssNode::Declaration { property, value, important } => {
                let imp = if *important { " !important" } else { "" };
                format!("{}: {}{}", property, value, imp)
            }
            CssNode::Rule { selector, declarations, children } => {
                let indent = "  ".repeat(depth);
                let decls: Vec<String> = declarations.iter()
                    .map(|d| format!("{indent}  {};", self.serialize_node(&CssNode::Declaration { .. }, 0)))
                    .collect();
                format!("{indent}{selector} {{\n{}\n{indent}}}", decls.join("\n"))
            }
            CssNode::Comment(text) => format!("/* {} */", text),
            _ => String::new(),
        }
    }
}

// Evaluated → Serialized
impl Evaluated {
    pub fn serialize(&self, style: OutputStyle) -> Serialized {
        let css = Serializer::new(style).serialize(&self.nodes);
        Serialized { css }
    }
}
```

---

## 6. 错误处理

```rust
//! src/error.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SassError {
    #[error("词法错误: {message} (位置 {pos})")]
    Lex { message: String, pos: usize },

    #[error("语法错误: 期望 {expected}, 实际 {found}")]
    Parse { expected: String, found: String },

    #[error("求值错误: {0}")]
    Eval(String),

    #[error("类型错误: 期望 {expected}, 实际 {actual}")]
    Type { expected: String, actual: String },

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("模块错误: {0}")]
    Module(String),
}

pub type Result<T> = std::result::Result<T, SassError>;
```

---

## 7. 公共 API

```rust
//! src/lib.rs

pub mod css;
pub mod error;
pub mod eval;
pub mod lex;
pub mod parse;
pub mod stage;

pub use css::node::CssNode;
pub use error::{Result, SassError};
pub use stage::source::Source;
pub use stage::serialized::Serialized;

use stage::evaluated::Evaluated;
use stage::lexed::Lexed;
use stage::parsed::Parsed;

pub mod style {
    #[derive(Debug, Clone, Copy)]
    pub enum OutputStyle { Expanded, Compressed }
}

use style::OutputStyle;

/// 编译入口——完整管线。
pub fn compile(source: &str, style: OutputStyle) -> Result<String> { /* ... */ }

/// 编译（展开式）。
pub fn compile_expanded(source: &str) -> Result<String> { /* ... */ }

/// 编译（压缩式）。
pub fn compile_compressed(source: &str) -> Result<String> { /* ... */ }

/// 文件编译。
pub fn compile_file(path: &PathBuf, style: OutputStyle) -> Result<String> { /* ... */ }

/// 带加载路径的文件编译。
pub fn compile_file_with_load_paths(path: &PathBuf, style: OutputStyle, paths: Vec<PathBuf>) -> Result<String> { /* ... */ }

/// 批量编译（Bootstrap / Element Plus 验证用）。
pub fn compile_batch(file_paths: &[PathBuf], style: OutputStyle) -> BatchResult { /* ... */ }

/// 初始化 tracing 订阅器。
pub fn init_tracing() { /* ... */ }
```

---

## 8. 学习里程碑

```
Milestone 1: Hello CSS
══════════════════════════
目标：编译 "a { color: red; }" → "a {\n  color: red;\n}\n"
涉及：Source, Lexer, Parser, Evaluator, Serializer
学到：Iterator, Result, enum, match

Milestone 2: 变量和表达式
══════════════════════════
目标：编译 "$x: 10px; a { width: $x; }"
涉及：Env, Value, eval_value
学到：不可变数据结构, 闭包

Milestone 3: 嵌套规则
══════════════════════════
目标：编译 ".outer { .inner { color: red; } }"
涉及：递归 fold, 选择器拼接
学到：递归, try_fold

Milestone 4: @规则
══════════════════════════
目标：编译 @media, @mixin, @include
涉及：AtRule 求值, Mixin 展开
学到：类型状态, 效应处理

Milestone 5: 错误处理
══════════════════════════
目标：友好的错误信息涉及 thiserror
学到：Error trait, From, ?

Milestone 6: 完整 sass-spec
══════════════════════════
目标：通过 sass-spec 核心测试
涉及：所有功能整合
学到：测试, 重构
```

---

## 9. 设计权衡

| 维度 | 决策 | 理由 |
|------|------|------|
| 解析器 | Pratt（算符优先）手写 | 更好的错误信息 + CSS 透传 + SCSS 特殊语法 |
| 求值环境 | HashMap + clone | 简单够用，im-rs 已移除以降低依赖 |
| 内建函数 | 按类别分文件 | color/list/map/string/selector 各一个文件 |
| 错误类型 | thiserror enum | 类型安全 + 精确错误位置 |
| 序列化 | 直接写入 String | 避免 format! 链的多次分配 |
| 内存管理 | 链式反应设计 | 超限返回 Err → Rust 所有权自动释放 |
| 测试验证 | BS + EP 双框架 | Bootstrap 验证兼容性，EP 验证 SCSS 高级特性 |

---

## 10. Cargo.toml 依赖

```toml
[package]
name = "sasspile"
version = "0.5.0"
edition = "2024"
rust-version = "1.97"

[dependencies]
thiserror = "2"
tracing = { version = "0.1", optional = true }
tracing-subscriber = { version = "0.3", optional = true, features = ["env-filter", "fmt"] }

[features]
default = ["tracing"]
tracing = ["dep:tracing", "dep:tracing-subscriber"]

[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "enterprise_bench"
harness = false
```

---

## 11. 下一步

```
选一个开始：

A) 创建 cargo new sasspile 项目，搭模块骨架
B) 写第一行代码：Source + Lexer 骨架
C) 写第一个测试：端到端编译 "a { color: red; }"
```

---

> **核心理念**：不是「尽可能多地用函数式」，而是「让每一行代码都有明确的输入和输出，没有隐藏的副作用」。
