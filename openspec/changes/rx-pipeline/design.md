# rx-pipeline Design

## 核心原则：编译器 = 流的变换链

**不从现有代码出发。从 rxrust 的算子语义出发，推导每个阶段。**

```
Source chars ──scan──▶ Tokens ──scan──▶ Nodes ──scan──▶ CssNodes ──flat_map──▶ CSS chars
              tokenize        parse           evaluate           serialize
```

每个阶段是一个**纯流变换算子**，不持有"阶段对象"，不调用"阶段方法"。

---

## 已完成阶段

### Phase I: 依赖 & 骨架 ✅

**改动**:
- `Cargo.toml`: `rxrust = "1.0.0-rc.5"` 升为核心依赖（不再是 optional）
- `src/lib.rs`: 移除 `#[cfg(feature = "reactive")]` 门控
- `src/pipeline/mod.rs`: `compile()` 全管线组装入口
- `src/pipeline/otel_op.rs`: OTel 操作符

**验证**: 核心测试 241/241 全通过

### Phase II: 词素发射（tokenize）✅

**核心发现**: scan 只能 emit 一个值 per 输入。多字符 token（ident、number、string）需要特殊处理。

**关键设计**: `scan + flat_map` 模式（见 `rxrust-patterns` skill）

```rust
// Accumulator = LexerScanner { state, emitted }
// 每步: feed(char) → Vec<Token>（0..N 个产出）
// flat_map: 将 Vec<Token> 展开为单个 Token 流

chars
    .scan(LexerScanner::new(), |scanner, ch| scanner.feed(ch))
    .flat_map(|scanner| Observable::from_iter(scanner.emitted))
    .flat_map(|token| Observable::from_iter(token.to_string().chars()))
```

**flush 问题**: 末尾追加空白字符以触发最后累积的多字符 token 产出。
`compile_to_string` 结果 `trim_end()` 去除 flush 空白。

**文件**:
- `src/lex/stream.rs` — `LexerState` + `feed()` 纯函数
- `src/pipeline/mod.rs` — `compile()` 接入 tokenize
- `tests/pipeline_test.rs` — 10 个测试全通过（含旧 Lexer 对比）

**算法参考**: `src/lex/scanner.rs` 的词法识别逻辑

---

## 阶段 3: 结构累积（parse）

**算子选择**: `scan` — 需要累积状态（brace 嵌套栈）

**不是**: 把现有递归下降 Parser 包一层。
**是**: Token 流上的结构累积——遇到 `{` 压栈，遇到 `}` 弹栈并 emit 完整节点。

```
tokens:  Selector(".a") LBrace Prop("color") Colon Value("red") Semicolon RBrace
          │                 │    │                            │
scan      ▼                 │    ▼                            ▼
state:   stack=[.a]         │    stack=[.a, body=[color:red]]   │
                                                    emit Rule(.a { color: red })
```

### 状态结构

```rust
pub struct ParseState {
    brace_stack: Vec<ParseFrame>,
    current_buffer: Vec<Token>,
}

struct ParseFrame {
    selector: Option<String>,
    props: Vec<Node>,
}
```

### 纯函数

```rust
pub fn absorb(state: ParseState, token: Token) -> (ParseState, Vec<Node>) {
    match token {
        Token::LBrace => {
            // 压栈：新 frame，selector 从 buffer 提取
            let frame = ParseFrame { selector: parse_selector(&state.current_buffer), props: vec![] };
            let mut new_state = state;
            new_state.brace_stack.push(frame);
            new_state.current_buffer.clear();
            (new_state, vec![])
        }
        Token::RBrace => {
            // 弹栈：emit 完整 Rule
            let mut new_state = state;
            let frame = new_state.brace_stack.pop().expect("unmatched }");
            let node = Node::Rule { selector: frame.selector, body: frame.props };
            (new_state, vec![node])
        }
        other => {
            // 累积到 buffer
            let mut new_state = state;
            new_state.current_buffer.push(other);
            (new_state, vec![])
        }
    }
}
```

### 管线接入

```rust
fn parse(tokens: Observable<Token>) -> Observable<Node> {
    tokens
        .scan(ParseState::new(), absorb)
        .flat_map(|(_, nodes)| Observable::from_iter(nodes))
}
```

### 算法参考

- `src/parse/at_rules.rs` — @media/@keyframes 结构
- `src/parse/at_rules_flow.rs` — @if/@else 结构
- `src/parse/at_rules_modules.rs` — @use/@forward 结构

### 待创建文件

- `src/parse/stream.rs` — ParseState + absorb()

---

## 阶段 4: 流式求值（evaluate）

**算子选择**: `scan` — 需要跨节点传播状态（Env）

```
nodes:   Define($x, 1)  Prop(color, $x)  @if(cond, [Prop(a,1)])
          │               │                 │
scan      ▼               ▼                 ▼
env:     {x:1} ──自动──▶ {x:1} ──自动──▶ {x:1}
emit:    []              [color: 1]       [a: 1]（cond=true 时）
```

**关键**: Env 在 scan 累加器中**自动跨节点传递**。不需要手动把 Env 从第一个节点传到第二个节点。

### 状态结构

```rust
pub struct EvalState {
    env: Env,
    // Env 是 move 语义：每次 eval 返回新 Env
}
```

### 纯函数

```rust
pub fn eval_node(state: EvalState, node: Node) -> (EvalState, Vec<CssNode>) {
    let EvalState { env } = state;
    match node {
        Node::Define(name, value) => {
            let (val, env) = eval_expr(value, env);
            let env = env.bind(name, val);
            (EvalState { env }, vec![])
        }
        Node::Prop(name, value) => {
            let (val, env) = eval_expr(value, env);
            let css = CssNode::Decl { prop: name, value: val.to_string() };
            (EvalState { env }, vec![css])
        }
        Node::Rule { selector, body } => {
            let env = env.enter_scope();
            let (mut css_nodes, env) = eval_nodes(body, env);
            let env = env.exit_scope();
            let rule = CssNode::Rule { selector, body: css_nodes };
            (EvalState { env }, vec![rule])
        }
        Node::If(cond, then_body, else_body) => {
            let (val, env) = eval_expr(cond, env);
            if val.is_truthy() {
                eval_nodes_into_css(then_body, env)
            } else if let Some(else_body) = else_body {
                eval_nodes_into_css(else_body, env)
            } else {
                (EvalState { env }, vec![])
            }
        }
        // @for/@each 展开为多个节点注入 emissions
        Node::For(var, range, body) => {
            eval_for_loop(var, range, body, env)
        }
        Node::Each(var, list, body) => {
            eval_each_loop(var, list, body, env)
        }
    }
}
```

### 管线接入

```rust
fn evaluate(nodes: Observable<Node>) -> Observable<CssNode> {
    nodes
        .scan(EvalState::new(), eval_node)
        .flat_map(|(_, css_nodes)| Observable::from_iter(css_nodes))
}
```

### 算法参考

- `src/eval/env_impl.rs` — Env 作用域操作
- `src/eval/control_flow.rs` — @if/@else/@for/@each/@while
- `src/eval/rule.rs` — 规则求值
- `src/eval/extend.rs` — @extend 处理

### Env 与 scan 的关键适配

现有 Env 设计已经是 move 语义（`Env → (T, Env)`），完美适配 scan：
```rust
// scan 闭包: (EvalState, Node) -> (EvalState, Vec<CssNode>)
// 消费旧 Env，返回新 Env——零 clone
```

### 待创建文件

- `src/eval/stream.rs` — EvalState + eval_node()

---

## 阶段 5: 序列化展开（serialize）

**算子选择**: `flat_map` — 1 CssNode → N CSS 字符

```
css_nodes:  CssNode::Rule(...)  CssNode::Decl(...)
                │                      │
flat_map        ▼                      ▼
chars:      ".a {\n  color: red;\n}\n"  "..."
```

### 纯函数

```rust
fn render_node(node: &CssNode) -> String {
    match node {
        CssNode::Rule { selector, body } => {
            let inner = body.iter().map(render_node).collect::<Vec<_>>().join("\n");
            format!("{} {{\n{}\n}}\n", selector, indent(inner))
        }
        CssNode::Decl { prop, value } => {
            format!("  {}: {};", prop, value)
        }
        CssNode::AtRule { name, params, body } => {
            // @media, @keyframes, etc.
        }
    }
}
```

### 管线接入

```rust
fn serialize(nodes: Observable<CssNode>) -> Observable<char> {
    nodes
        .flat_map(|node| {
            let rendered = render_node(&node);
            Observable::from_iter(rendered.chars())
        })
}
```

### 算法参考

- `src/css/serialize.rs` — 现有序列化逻辑
- `src/css/serialize_write.rs` — CSS 写入格式化

### 待创建文件

- `src/css/stream.rs` — render_node() + serialize()

---

## 全管线组装

```rust
pub fn compile(input: &str) -> LocalBoxedObservable<'static, char, Infallible> {
    // flush: 追加空白字符触发最后 token 产出
    let mut chars: Vec<char> = input.chars().collect();
    if !chars.is_empty() {
        chars.push(' ');
    }

    Local::from_iter(chars)
        .scan(LexerScanner::new(), |s, ch| s.feed(ch))
        .flat_map(|s| Observable::from_iter(s.emitted))    // Tokens
        .scan(ParseState::new(), absorb)
        .flat_map(|(_, nodes)| Observable::from_iter(nodes)) // Nodes
        .scan(EvalState::new(), eval_node)
        .flat_map(|(_, css)| Observable::from_iter(css CssNodes))
        .flat_map(|node| Observable::from_iter(render_node(&node).chars())) // Chars
        .box_it()
}
```

**没有 Lexer 结构体、没有 Parser 结构体、没有 Evaluator 结构体。**
**只有流变换函数 + 状态结构体（scan 的累加器）。**

---

## 模块系统：流的 merge

**@use 不是"加载文件然后返回 Ast"——是"把另一个文件的主流 merge 进来"。**

```rust
fn evaluate_with_modules(nodes: Observable<Node>, base_path: PathBuf) -> Observable<CssNode> {
    nodes
        .scan(ModuleEvalState::new(base_path), |state, node| {
            match node {
                Node::Use(path) => {
                    let dep_stream = state.load_or_get(path);
                    EvalOutput::merge_dep(dep_stream)
                }
                Node::Forward(path) => {
                    // @forward 类似 @use，但不直接 emit
                    let dep_stream = state.load_or_get(path);
                    EvalOutput::merge_dep_forward(dep_stream)
                }
                other => state.eval(other),
            }
        })
        .flat_map(|out| out.emissions)
}
```

**模块缓存**在 `ModuleEvalState` 中管理——已加载的模块流不重复加载。

---

## OTel 操作符

```rust
fn otel_span<T: std::fmt::Debug>(name: &'static str) -> impl Fn(Observable<T>) -> Observable<T> {
    move |obs| obs.inspect(move |item| {
        tracing::debug!(stage = name, value = ?item);
    })
}

// 接入
Observable::from_iter(source.chars())
    .pipe(tokenize)
    .pipe(otel_span("lex"))
    .pipe(parse)
    .pipe(otel_span("parse"))
    .pipe(evaluate)
    .pipe(otel_span("eval"))
    .pipe(serialize)
```

---

## 与现有代码的关系

| 现有代码 | 如何对待 |
|---------|---------|
| `lex/scanner.rs` | ✅ **参考算法**，已重写为 `feed(char)` |
| `parse/at_rules.rs` | **参考语法规则**，待重写为 `absorb(token)` |
| `eval/env_impl.rs` | **参考作用域语义**，待重写为 `eval(node)` |
| `eval/builtin_*.rs` | **直接复用**——纯函数无需改动 |
| `parse/ast/*.rs` | **直接复用**——数据类型不变 |
| `css/serialize.rs` | **参考格式规则**，待重写为 `render_node()` |
| `eval/control_flow.rs` | **参考循环/条件逻辑**，待整合 |

---

## 模块结构

```
src/
├── pipeline/
│   ├── mod.rs          # compile() 全管线组装 ✅
│   ├── otel_op.rs      # otel_span 操作符 ✅
│   └── module_stream.rs # 模块流 merge（待创建）
├── lex/
│   └── stream.rs       # LexerState + tokenize() ✅
├── parse/
│   └── stream.rs       # ParseState + parse()（待创建）
├── eval/
│   └── stream.rs       # EvalState + evaluate()（待创建）
└── css/
    └── stream.rs       # render_node + serialize()（待创建）
```

---

## 性能与正确性

| 维度 | 目标 |
|------|------|
| 内存 | 无全量 Ast / Vec<CssNode> |
| 首字节 latency | O(1) — 第一个字符入 → 第一个 CSS 字节出 |
| 增量编译 | Subject 推入 → 下游自动重订阅 |
| 正确性 | sass-spec 基线 ≥ 当前 7620 |

---

## 限制

1. **算法迁移成本**: 现有词法/语法/求值算法需重写为流形态（一次性投入）
2. **调试复杂度**: scan 内部状态不如递归栈直观（用 OTel inspect 补偿）
3. **@content 闭包**: 需要嵌套订阅，需特殊处理
4. **flush 空白**: tokenize 末尾追加空白以触发 flush，对 round-trip 测试有影响
