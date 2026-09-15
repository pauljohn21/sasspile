# rx-pipeline: 反应式根源重构

## Why

当前 sasspile 是**命令式分阶段编译器**。即使引入 rxrust，如果只是在现有代码外包一层 Observable，等于旧酒装新瓶。

**真正的问题不是"哪个模块需要改"，而是"整个编译器的抽象单位错了"。**

- 命令式：编译器 = 对象（Lexer/Parser/Evaluator）+ 方法调用
- 反应式：编译器 = 流变换函数（tokenize/parse/evaluate/serialize）

## What Changes

**从"对象 + 方法"到"流变换 + 状态累加器"。**

```
Source chars ──flat_map──▶ Tokens ──scan──▶ Nodes ──scan──▶ CssNodes ──flat_map──▶ CSS chars
              tokenize         parse          evaluate           serialize
```

### 核心改变

**1. 没有 Lexer/Parser/Evaluator 结构体——只有流变换函数**

```rust
fn tokenize(chars: Observable<char>) -> Observable<Token> { ... }
fn parse(tokens: Observable<Token>) -> Observable<Node> { ... }
fn evaluate(nodes: Observable<Node>) -> Observable<CssNode> { ... }
fn serialize(nodes: Observable<CssNode>) -> Observable<char> { ... }
```

**2. 状态在 scan 累加器中自动传播**

`ParseState` 管理 brace 嵌套栈，`EvalState` 管理 Env 作用域链。不需要手动线程化。

**3. 现有代码是算法参考，不是被包装的对象**

- `lex/lexer.rs` → 参考词素识别算法 → 重写为 `LexerState::feed(char)`
- `parse/parser.rs` → 参考语法规则 → 重写为 `ParseState::absorb(token)`
- `eval/env*.rs` → 参考作用域语义 → 重写为 `EvalState::eval(node)`
- `eval/builtin_*.rs` → **直接复用**（纯函数不变）

**4. 模块系统 = 流的 merge**

`@use` 不是"加载文件返回 Ast"——是把另一个文件的主流 merge 进来。

**5. OTel 是流操作符**

散布的 `#[instrument]` → `.pipe(otel_span("lex"))`，只在 Observable 链中存在。

### 不受影响

- **内建函数**（builtin_*.rs）：纯函数，无需改动
- **AST 类型**（parse/ast/*.rs）：数据类型不变
- **tracing 基础设施**：保留，作为备选
- **CLI 接口**：`compile_expanded/Compressed` 签名不变

## Migration Impact

- **破坏性**：整体架构从"对象 + 方法"变为"流变换 + 状态累加器"
- **新增依赖**：`rxrust` crate (feature: `reactive`)
- **算法迁移**：词法/语法/求值算法需从递归/迭代器形态重写为 scan 累加器形态（一次性投入）
- **向后兼容**：默认 feature 不变，保留经典 Reactor 路径
- **性能**：首字节入 → 首字节出，无全量中间分配
