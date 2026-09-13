> ⛔ **禁止参照 dart-sass**：dart-sass 依赖 GC（垃圾回收），其嵌套结构依赖 GC 保。sasspile 是纯 Rust 项目，无 GC，所有权语义完全不同。任何实现必须基于 Rust 所有权模型和 sass-spec 规范，不得参照 dart-sass 的实现。

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
│  1. 阶段即状态 ── 编译阶段编码在泛型参数中               │
│     Reactor<StateRaw> → StateLexed → StateParsed         │
│              → StateEvaluated → StateSerialized          │
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
// 单类型 + 泛型状态参数（类型状态机）
pub struct Reactor<S = StateRaw> {
    text: String,
    base_path: Option<PathBuf>,
    load_paths: Vec<PathBuf>,
    tokens: Option<Vec<Token>>,
    ast: Option<Ast>,
    serialized: Option<String>,
    imports_seen: Vec<PathBuf>,
    css_nodes: Vec<CssNode>,
    trace: ReactorTrace,
    _state: PhantomData<S>,
}

// IO 层: src/runtime.rs — 全局 tokio runtime + block_on 桥接

// 状态标记（零大小类型）
pub struct StateRaw;
pub struct StateLexed;
pub struct StateParsed;
pub struct StateEvaluated;
pub struct StateSerialized;

// 阶段方法绑定到特定状态
impl Reactor<StateRaw> {
    pub fn lex(self) -> Result<Reactor<StateLexed>> { /* ... */ }
}
impl Reactor<StateParsed> {
    pub fn evaluate(self) -> Result<Reactor<StateEvaluated>> { /* ... */ }
}
```

---

## 3. 模块结构

```
sasspile/
├── Cargo.toml
├── DESIGN.md                    ← 你正在读的文档
├── src/
│   ├── lib.rs                   ── 公共 API（管线入口）
│   ├── main.rs                  ── CLI
│   ├── runtime.rs               ── Tokio 全局运行时 + block_on 桥接
│   │
│   ├── lex/                     ── 词法分析
│   │   ├── mod.rs               ── Lexer（迭代器实现）
│   │   └── token.rs             ── Token 定义
│   │
│   ├── runtime.rs               ── Tokio 全局运行时 + block_on 桥接
│   │
│   ├── parse/                   ── 语法分析
│   │   ├── mod.rs               ── Parser（Iterator 模式）
│   │   ├── ast/                 ── AST + color_types + display
│   │   ├── ast_impl.rs          ── AST 实现（to_scss 等）
│   │   ├── at_rules.rs          ── @规则 解析
│   │   ├── expr/                ── 表达式解析
│   │   └── nodes.rs             ── 节点解析辅助
│   │
│   ├── eval/                    ── 求值 + 管线类型状态机
│   │   ├── mod.rs               ── Evaluator + eval_nodes
│   │   ├── reactor.rs           ── Reactor<S>（lex/parse/evaluate/serialize 管线）
│   │   ├── env.rs               ── Env/Scope/ModuleExports 类型定义
│   │   ├── env_impl.rs          ── Env 方法（链式 self -> Self）
│   │   ├── scope.rs             ── Scope 结构 + HashMap 7 字段
│   │   ├── module.rs            ── @use/@forward + load_module（tokio::fs 异步）
│   │   ├── import.rs            ── @import + load_import
│   │   ├── forward.rs           ── @forward 模块转发
│   │   ├── rule.rs              ── Rule 求值
│   │   ├── control_flow.rs      ── @if/@for/@each/@while
│   │   ├── mixin.rs             ── @mixin/@include
│   │   ├── extend.rs            ── @extend 后处理
│   │   ├── at_params.rs         ── @media/@supports 参数
│   │   ├── file_resolver.rs     ── 文件解析 + 冲突检测
│   │   ├── module_helpers.rs    ── bind_exports + merge_module_cache
│   │   ├── module_dispatch.rs   ── 内建函数注册 #[derive(BuiltinRegistry)]
│   │   ├── plain_css.rs         ── plain CSS 模式检查
│   │   ├── builtin.rs           ── call_builtin 分派入口
│   │   ├── meta_ops.rs          ── meta.apply / meta.load-css
│   │   ├── builtin/             ── 内建函数子模块
│   │   └── value/               ── 值类型 (Value + ops + display)
│   │
│   ├── css/                     ── CSS 生成
│   │   ├── mod.rs               ── Serializer
│   │   └── node.rs              ── CssNode 定义
│   │
│   └── error.rs                 ── 统一错误类型
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
    Number(String),         // 数字（含单位）
    String(String),         // 字符串
    Hash(String),           // #color

    // 符号
    LParen, RParen,         // ( )
    LBrace, RBrace,         // { }
    LBracket, RBracket,     // [ ]
    Colon, Semicolon,       // : ;
    Comma, Dot,             // , .
    Plus, Minus, Star, Slash, Percent,

    // 特殊
    AtIdent(String),        // @import, @mixin 等
    DollarIdent(String),    // $variable
    Whitespace,
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
    Rule {
        selector: Selector,
        body: Vec<Node>,
    },

    /// 声明：property: value;
    Decl {
        property: String,
        value: Value,
        important: bool,
    },

    /// 变量声明：$name: value;
    Variable {
        name: String,
        value: Value,
    },

    /// @规则：@media, @mixin 等
    AtRule {
        name: String,
        params: Option<String>,
        body: Option<Vec<Node>>,
    },

    /// 注释
    Comment(String),
}

/// 选择器。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Selector {
    pub raw: String,
}

/// 值表达式。
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64, Option<String>),  // 值 + 单位
    String(String, bool),         // 内容 + 是否引号
    Color(Color),
    List(Vec<Value>, Separator),
    Variable(String),
    Call(String, Vec<Value>),    // 函数调用
}

#[derive(Debug, Clone, PartialEq)]
pub enum Separator { Comma, Space, Slash }

#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub r: u8, pub g: u8, pub b: u8, pub a: f32,
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
        declarations: Vec<Declaration>,
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
    },
    Comment(String),
}

pub struct Declaration {
    pub property: String,
    pub value: String,
    pub important: bool,
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

// Reactor lex: Reactor<StateRaw> → Reactor<StateLexed>
impl Reactor<StateRaw> {
    pub fn lex(self) -> Result<Reactor<StateLexed>> {
        let tokens = Lexer::new(&self.text)
            .filter(|t| !t.as_ref().is_ok_and(Token::is_trivia))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Reactor {
            text: self.text, base_path: self.base_path, load_paths: self.load_paths,
            tokens: Some(tokens), ast: None, serialized: None,
            imports_seen: vec![], css_nodes: vec![],
            trace: self.trace, _state: PhantomData,
        })
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

// Reactor parse: Reactor<StateLexed> → Reactor<StateParsed>
impl Reactor<StateLexed> {
    pub fn parse(self) -> Result<Reactor<StateParsed>> {
        let ast = Parser::new(self.tokens.unwrap_or_default()).parse()?;
        Ok(Reactor {
            text: self.text, base_path: self.base_path, load_paths: self.load_paths,
            tokens: self.tokens, ast: Some(ast), serialized: None,
            imports_seen: vec![], css_nodes: vec![],
            trace: self.trace, _state: PhantomData,
        })
    }
}
```

### 5.3 Evaluator —— fold + 不可变环境

```rust
/// 求值环境——不可变，查找返回引用。
#[derive(Debug, Clone)]
pub struct Env {
    bindings: im::HashMap<String, Value>,  // 不可变 HashMap
}

impl Env {
    pub fn new() -> Self {
        Self { bindings: im::HashMap::new() }
    }

    /// 返回新环境（不修改自身）。
    pub fn bind(&self, name: String, value: Value) -> Self {
        let mut new = self.clone();
        new.bindings.insert(name, value);
        new
    }

    pub fn lookup(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }
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

// Reactor evaluate: Reactor<StateParsed> → Reactor<StateEvaluated>
impl Reactor<StateParsed> {
    pub fn evaluate(self) -> Result<Reactor<StateEvaluated>> {
        let ast = self.ast.unwrap_or_default();
        let (nodes, env) = Evaluator::evaluate_with_env(ast)?;
        Ok(Reactor {
            text: self.text, base_path: self.base_path, load_paths: self.load_paths,
            tokens: self.tokens, ast: self.ast, serialized: None,
            imports_seen: vec![], css_nodes: nodes,
            trace: self.trace, _state: PhantomData,
        })
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

// Reactor serialize: Reactor<StateEvaluated> → Reactor<StateSerialized>
impl Reactor<StateEvaluated> {
    pub fn serialize(self, style: OutputStyle) -> Reactor<StateSerialized> {
        let css = Serializer::new(style).serialize(&self.css_nodes);
        Reactor {
            text: self.text, base_path: self.base_path, load_paths: self.load_paths,
            tokens: self.tokens, ast: self.ast, serialized: Some(css),
            imports_seen: vec![], css_nodes: vec![],
            trace: self.trace, _state: PhantomData,
        }
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
    #[error("词法错误: {message} (位置 {position})")]
    LexError { message: String, position: usize },

    #[error("语法错误: {expected}, 实际 {found}")]
    ParseError { expected: String, found: String },

    #[error("求值错误: {0}")]
    EvalError(String),

    #[error("类型错误: 期望 {expected}, 实际 {actual}")]
    TypeError { expected: String, actual: String },

    #[error("单位不兼容: 无法将 {from} 转为 {to}")]
    UnitError { from: String, to: String },

    #[error("未定义变量: {0}")]
    UndefinedVariable(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
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

pub use css::node::CssNode;
pub use error::{Result, SassError};
pub use eval::reactor::{Reactor, ReactorModule, StateLexed, StateParsed, StateRaw};

pub mod style {
    #[derive(Debug, Clone, Copy)]
    pub enum OutputStyle { Expanded, Compressed }
}

use style::OutputStyle;

/// 编译入口——完整管线（委托给 Reactor）。
pub fn compile(source: &str, style: OutputStyle) -> Result<String> {
    Reactor::new(source)
        .lex()?
        .parse()?
        .evaluate()?
        .serialize(style)
        .finish()
}

/// 编译（展开式）。
pub fn compile_expanded(source: &str) -> Result<String> {
    compile(source, OutputStyle::Expanded)
}

/// 编译（压缩式）。
pub fn compile_compressed(source: &str) -> Result<String> {
    compile(source, OutputStyle::Compressed)
}
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

## 9. 与旧版 sasspile 对比

| 维度 | 旧版 | 新版 |
|------|------|------|
| 架构 | 命令式 + 可变状态 | 函数式 + 管线 |
| 阶段 | 松散 struct | 类型状态机 |
| 求值 | &mut self | fold + 不可变 Env |
| 错误 | Box<dyn Error> | thiserror enum |
| 集合 | for loop | Iterator 链 |
| 测试 | 集成测试为主 | 单元 + property test |
| 文件 | 500 行上限 | 300 行上限（更聚焦） |

---

## 10. Cargo.toml 依赖

```toml
[package]
name = "sasspile"
version = "0.9.8"
edition = "2024"
rust-version = "1.85"

[dependencies]
thiserror = "2"
tokio = { version = "1", features = ["rt-multi-thread", "fs"] }
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-opentelemetry = { version = "0.28", optional = true }
opentelemetry = { version = "0.28", optional = true }
opentelemetry_sdk = { version = "0.28", features = ["trace"], optional = true }
opentelemetry-stdout = { version = "0.28", features = ["trace"], optional = true }

[features]
default = ["tracing"]
tracing = ["dep:tracing", "dep:tracing-subscriber"]
otel = ["dep:opentelemetry", "dep:opentelemetry_sdk", "dep:opentelemetry-stdout", "dep:tracing-opentelemetry"]

[dev-dependencies]
insta = "1"            # 快照测试
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
