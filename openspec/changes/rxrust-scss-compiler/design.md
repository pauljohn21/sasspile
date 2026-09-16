## Context

`sasspile-rx` 是一个 Rust SCSS 编译器项目，使用 **rxrust 1.0.0-rc.5** 作为核心抽象哲学与实现基础。设计完全从 rxrust 四原语（Observable/Observer/Subscription/Operator）出发，不参照任何现有实验性代码。

> ⚠️ 仓库中当前的 `main.rs` 只是一个概念实验产物，不代表最终架构。最终设计以 rxrust 算子机制为唯一根源。

本项目彻底拒绝任何命令式编程范式。所有计算——包括状态变更、错误传播、副作用、模块缓存——都必须通过 rxrust 的四原语表达。

**外部验证基准：**
- `sass-spec`（git submodule, --depth 1）= **唯一**行为契约（HRX 文件定义 input/output 对，排除弃用目录）
- `bootstrap`（git submodule, --depth 1）= 企业级 `@import` 范式验证
- `element-plus`（git submodule, --depth 1）= 企业级 `@use ... as *` + `!global` 范式验证

**NOT dart-sass = 禁止参照。**

---

## 核心哲学: 四原语

rxrust 的一切皆可归约为四个原语。sasspile-rx 的整个编译器皆是这四原语的组合：

```
┌─────────────────────────────────────────────────────────────────┐
│                   rxrust 四原语                                   │
├─────────────────────────────────────────────────────────────────┤
│  Observable<Item, Err>    ─  可被订阅的流（编译器核心载体）          │
│  Observer<Item, Err>      ─  流的消费者（next/error/complete）      │
│  Subscription             ─  订阅句柄（生命周期控制）               │
│  Operator (via .pipe())   ─  纯函数变换（stage 的组合子）          │
└─────────────────────────────────────────────────────────────────┘
```

### Observable：编译流的载体

`Observable` 是惰性的，只有 `subscribe()` 才开始计算。编译器的每一帧数据——一个 char、一个 Token、一个 Node、一个 CssNode——皆以 Item 形式在 Observable 中传播。

```
Source Observable  →  .pipe(stage₁)  →  .pipe(stage₂)  →  ...  →  .last()  →  subscribe(Observer)
```

### Observer：编译器在 subscribe 端点被表达

不是编译器订阅用户，而是**编译器作为用户订阅 Observable**。最终 Observer 是 `FnMut(Result<String, CompileError>)`，处理编译输出。

```rust
.last()
.subscribe(|result: Result<String, CompileError>| match result {
    Ok(css)  => info!(stage = "output", bytes = css.len(), "{css}"),
    Err(e)   => error!(stage = "output", error = %e),
});
```

### Operator：stage 的纯函数语义

所有 stage 算子遵循 rxrust 的 Observer 包装模式：

```rust
// 如 Map 算子的结构（rxrust/src/ops/map.rs）
pub struct Map<S, F> {      // 用户-facing operator
    source: S,
    func: F,
}
pub struct MapObserver<O, F> {  // observer wrapper
    observer: O,
    func: F,
}
impl Observer for MapObserver { fn next(&mut self, v) { self.observer.next((self.func)(v)); } }
```

sasspile-rx 的每个 stage 必须遵循同等模式。

### Subscription：编译生命周期

`subscribe()` 返回 `Subscription` handle，可用于 `.unsubscribe()` 终止编译。`.last()` 在产出唯一值后自动 unsubscribe。

---

## Goals / Non-Goals

**Goals：**
- 建立 rx-first 编译器架构，每个 stage 都是 `Observable<In> → Observable<Out>` 的纯算子
- sass-spec HRX 作为**唯一**行为契约（排除 libsass 系弃用目录）
- error-as-value：`Infallible` 管道 + `Result<T, CompileError>` 作为 Item
- 所有副作用通过 `tap` 注入，流变换保持纯函数语义
- 模块缓存通过 `publish`/`multicast` + `ref_count`（Connectable 模式）实现共享
- 单文件 ≤ 500 行（优先保证逻辑内聚）
- tracing span 必须进入每个 stage 入口/出口（使用 `tap` 桥接）
- 企业级 100% 通过：Bootstrap + Element Plus 所有 SCSS 编译零错误

**Non-Goals：**
- 初期实现完整 SCSS 特性集（逐步覆盖）
- 并行编译（当前 `Local` 单线程语义已足够）
- 错误恢复 / 部分编译（fail-fast 作为 error 值）

---

## Architecture: rxrust 算子驱动的编译器

### 总览

```
┌────────────────────────────────────────────────────────────────────�
│  entry_point (≤ 50 行): subscribe handler, 只做 tap(output)       │
└──────────────────────────�─────────────────────────────────────────┘
                           │
�──────────────────────────▼─────────────────────────────────────────┐
│  pipeline.rs (编排): Local::<str, Infallible>::from_iter(src)       │
│    .pipe(tokenize_dst()).box_it()   ← Creation + Transformation     │
│    .pipe(parse_dst()).box_it()      ← Transformation + scan 模式    │
│    .pipe(evaluate_dst()).box_it()   ← flat_map(指令展开) + publish  │
│    .pipe(serialize_dst()).box_it()  ← map 渲染                      │
│    .last()                          ← Aggregation                   │
│    .subscribe(|result| { ... })     ← Observer 消费                 │
└────────────────────────────────────────────────────────────────────�
                           │
     ┌─────────────────────┼─────────────────────┐
     ▼                     ▼                     ▼
• tap(副作用注入)     • scan(状态累积)      • flat_map(嵌套展开)
• filter(去重Token)   • distinct(选择器去重) • publish(共享模块)
• reduce(收敛为String) • box_it(类型擦除)    • Infallible(error-as-value)
```

### Stage 算子的 rxrust 签名

| Stage | rxrust 签名 | 内部主算子 | Observer 包装 |
|-------|------------|-----------|--------------|
| `tokenize` | `Observable<char> → Observable<Token>` | `scan(initial, reducer)` | `TokenizerObserver<O, ScannerState>` |
| `parse` | `Observable<Token> → Observable<Node>` | `scan(ASTBuilder, feed)` | `ParserObserver<O, ParseState>` |
| `evaluate` | `Observable<Node> → Observable<Result<CssNode, CompileError>>` | `flat_map(directive_expand)` + `publish(module_cache)` | `EvaluatorObserver<O, Scope>` |
| `serialize` | `Observable<Result<CssNode, CompileError>> → Observable<char>` | `map(CssNode::render)` | `SerializerObserver<O>` |

### 算子使用清单（每个类必须用到）

| rxrust 类别 | 必需算子 | 在 sasspile-rx 中的应用 |
|-------------|---------|------------------------|
| Creation | `Local::from_iter`, `Local::of` | 创建 char 源、常量 Token |
| Transformation | `map`, `scan`, `flat_map`, `scan_map` | Token 转换、AST 累积、@extend 展开 |
| Filtering | `filter`, `distinct`, `distinct_until_changed` | 去重 Token、@use 同名过滤、选择器去重 |
| Combination | `merge`, `merge_all`, `zip`, `with_latest_from` | 多文件并行、sourcemap 合并 |
| Utility | `tap`, `finalize`, `delay` | tracing 注入、资源清理、延迟加载 |
| Aggregation | `reduce`, `last`, `collect`, `sum` | 收敛为 Result<String, CompileError> |
| Connectable | `multicast`, `publish`, `ref_count` | 模块缓存的 shared 语义 |

---

## Decisions（以 rxrust 为根本的决策）

### Decision 1: 错误通道必须是 Infallible

选择：管道签名全部使用 `Observable<Item, Infallible>`。

**rxrust 哲学依据：** `Observer::error(self, err: Err)` 消费 observer，调用后流终止。若错误通过 error 通道传播，一旦遇到第一个错误，整个管道就停止——这不可能实现"错误作为 spec 产出"的语义。

```rust
// ✓ 正确: 错误是 Item
Observable<Result<CssNode, CompileError>, Infallible>

// ✗ 错误: 错误终止管道
Observable<CssNode, CompileError>
```

### Decision 2: 状态累积使用 scan，终值约聚使用 reduce/last

**rxrust 语义区分：**
- `scan(initial, f)`: 每步发射中间累积值 → tokenizer 内部适用
- `reduce(f)`: 仅流 complete 时发射一次 → 管道末端收敛为单值
- `last()`: 等价于 `reduce(|_, x| x)` → 取最后一帧 CssNode 流

#### Scenario: scan 在 parse 阶段累积 Token
```rust
tokens.scan(ASTBuilder::new(), |builder, token| builder.feed(token))
      .filter(|builder| builder.has_complete_rule())
      .map(|builder| builder.flush_rule())
```

### Decision 3: 嵌套结构展开使用 flat_map + merge_all

SCSS 核心是嵌套规则展开（`a { b { c: d } }` → `a b { c: d }`）。这天然对应 rxrust 的 `flat_map`:

```rust
nodes.flat_map(|node| match node {
    Nested(parent, children) => expand_nesting(parent, children), // 返回 Observable<Node>
    Other(n) => Local::of(n),                                     // 单值 Observable
})
```

`flat_map` 内部等价于 `map(closure).merge_all(concurrent)`，其中 `concurrent` 控制并发展开深度。

### Decision 4: 模块缓存共享使用 publish + ref_count（Connectable 模式）

**rxrust 机制：**
- `source.multicast(subject)` 创建 `ConnectableObservable`
- `.fork()` 创建多个下游 Observable（共享同一上游）
- `.connect()` 触发上游实际执行
- `.ref_count()` 自动管理连接（只剩一个订阅者时断开）

**在 sasspile-rx：**
```rust
// 模块加载结果缓存并共享
let module_stream = file_requests
    .flat_map(|path| load_and_compile(path))
    .publish(subject)     // multicast: 下游共享同一份
    .ref_count();         // 自动连/断

// 多文件可 fork() 订阅同一模块
let fork1 = module_stream.clone();
let fork2 = module_stream.clone();
```

### Decision 5: 所有副作用通过 tap 注入

**rxrust 哲学依据：** `tap` 是 rxrust 中专门承担副作用的算子，它不修改流值（不调用 map 的职责），在 Observer 的 `next` 中插入旁路逻辑。

sasspile-rx 中，以下 MUST 通过 `tap` 实现：
- 所有 `tracing::info!/debug!/warn!/error!` 调用
- 模块缓存写入 (`cache.insert(path, module.clone())`)
- 性能计数器递增
- SourceMap 记录

```rust
// ✓ 正确
nodes.tap(|node| debug!(?node, "parsed"))
     .pipe(evaluate)
     .tap(|cssn| debug!(?cssn, "evaluated"))

// ✗ 错误 — 不要将副作用塞进 map
nodes.map(|n| { debug!(?n); eval(n) })
```

### Decision 6: 阶段间用 box_it() 类型擦除

**rxrust 机制：** `box_it()` 调用 `BoxedObservable`，将复杂的嵌套泛型（如 `FlatMap<Map<Scan<FromIter<char>, ...>>, ...>`）擦除为统一的 `LocalBoxedObservable<T, E>`。

每个 `.pipe(stage)` 返回后 MUST 跟随 `.box_it()`，否则类型署名会指数爆炸：

```rust
Local::from_iter(chars)
    .pipe(tokenize_dst()).box_it()    // ← 擦除为 LocalBoxedObservable<Token, Infallible>
    .pipe(parse_dst()).box_it()       // ← 擦除为 LocalBoxedObservable<Node, Infallible>
    .pipe(evaluate_dst()).box_it()    // ← 擦除为 LocalBoxedObservable<Result<CssNode, CE>, Infallible>
    .pipe(serialize_dst()).box_it()   // ← 擦除为 LocalBoxedObservable<char, Infallible>
```

### Decision 7: 闭包必须标注 for<'a> Higher-Rank Trait Bound

**rxrust 库内所有算子签名都要求：**
```rust
F: for<'a> FnMut(Observable::Item<'a>) -> Output
```

sasspile-rx 的 stage 内的闭包 MUST 同样使用 `for<'a>` 注解，否则无法与 rxrust 算子 interoperate。

```rust
// tokenize 内部
let transform: Box<dyn for<'a> FnMut(char) -> Token> = Box::new(|c| classify(c));
```

---

## Shared Context 设计（rxrust 视角）

"Shared" 在 rxrust 语义下有双重含义：

### 含义 1: 跨订阅者数据共享（rxrust Connectable）
模块缓存的 shared 状态 SHOULD 通过 `Local::subject().publish()` 实现，确保多订阅者看到同一模块实例，且缓存命中时不会重复 tokenize/parse/evaluate。

### 含义 2: 单订阅者内状态传递（rxrust scan）
每个 stage 内部的状态累积（如 AST 构建器、变量 scope、文件位置追踪）通过 `scan(initial, reducer)` 的 `acc` 参数实现。这是"单线程内 Shared"——无需 Arc/Mutex。

### CompilerContext 在 rxrust 管道中的表达

```rust
pub struct CompilerContext {
    module_cache: MutRc<HashMap<PathBuf, EvaluatedModule>>,  // Rc for Local scope
    global_vars: MutRc<Scope>,                                // 可变的共享 scope
    span_ctx: MutRc<SpanContext>,                             // 用于 source map
}

// 注入方式: scan 携带
source.scan(CompilerCtx::new(), |ctx, item| ctx.apply(item))
      .tap(|(ctx, result)| { /* 访问 ctx.module_cache */ })
```

> **rxrust 关键约束：** `CompilerContext` 在 `Local` scope 下使用 `MutRc`（Rc<RefCell<T>>），而非 `MutArc`（Arc<Mutex<T>>）。单线程内 `RefCell` 的 borrow checker 已足够，避免 Mutex 开销。

---

## Requirement: 企业级 SCSS 项目 100% 通过测试

编译器 MUST 通过两个大型企业级 SCSS 项目的编译验证。

#### Scenario: 编译 Bootstrap SCSS（@import 范式）
- **WHEN** 处理 `bootstrap/scss/bootstrap.scss` 及其全部 `@import` 依赖树
- **THEN** rxrust 的 `flat_map(|file| compile_file(file))` MUST 递归展开所有 @import
- **THEN** MUST 输出与 `bootstrap/dist/css/bootstrap.css` 语义等价 CSS

#### Scenario: 编译 Element Plus SCSS（@use ... as * 范式）
- **WHEN** 处理 `element-plus/packages/theme-chalk/src/index.scss`
- **THEN** MUST 支持 `@use ... as *` 的命名空间扁平注入
- **THEN** MUST 支持 `$colors: map.deep-merge((...)) !global` 的全局副作用可见
- **THEN** 通过 `publish(subject).ref_count()` 确保 90%+ 文件共享同一份 `EvaluatedModule`

#### Scenario: 100% 覆盖率验收
- **WHEN** sass-spec 活跃目录 + Bootstrap + Element Plus 全量测试
- **THEN** 通过率 MUST 为 100%

---

## File Structure（≤ 500 行 / 文件）

```
src/
  lib.rs               — compile_pipeline() + subscribe handler (≤ 50 行)
  error.rs             — CompileError 枚举 + 错误格式化
  pipeline.rs          — 管道编排 (≤ 150 行)
  tokenize_dst.rs      — Observable<char> → Observable<Token> stage (≤ 500 行)
  parse_dst.rs         — Observable<Token> → Observable<Node> stage (≤ 500 行)
  evaluate_dst.rs      — Observable<Node> → Observable<Result<CssNode, CE>> 入口 (≤ 500 行)
    evaluate/extend.rs — flat_map(@extend 展开逻辑)
    evaluate/mixin.rs  — flat_map(@mixin/@include)
    evaluate/use.rs    — publish(@use 模块共享)
    evaluate/import.rs — flat_map(@import)
  serialize_dst.rs     — Observable<Result<CssNode, CE>> → Observable<char> (≤ 500 行)
  shared/
    context.rs         — CompilerContext + scan 状态注入
    scope.rs           — Scope 类型（变量存储）
    module.rs          — EvaluatedModule 结构
tests/
  tokenize.rs          — tokenize_dst 单元测试
  parse.rs             — parse_dst 单元测试
  evaluate.rs          — evaluate_dst 单元测试
  serialize.rs         — serialize_dst 单元测试
  integration.rs       — 端到端 sass-spec HRX 测试
  bootstrap.rs         — Bootstrap 端到端编译
  element_plus.rs      — Element Plus 端到端编译
```

---

## Risks / Trade-offs

| 风险 | rxrust 对应 | Mitigation |
|------|-----------|------------|
| 嵌套 Observable 过深 | `flat_map` 子 Observable 生命周期 | 初期保持扁平 pipe，指令展开不嵌套 Subject |
| 单文件 ≤ 500 vs 可读性 | — | 逻辑内聚优先，不强行等分 |
| `MutRc<RefCell<T>>` 运行时 borrow 冲突 | Local scope 的 borrow checker | 使用 `tap` 中 quick borrow，不在 scan 闭包内嵌套 borrow |
| `publish/ref_count` 生命周期 | Connectable 必须 connect 才执行 | 用 `ref_count` 自动化、但保留显式 `connect()` 选项 |
| 类型擦除后难以调试 | `box_it()` vtable 跳转 | 在 `tap` 中加 `debug!(?item)` 绕过 |

---

## Migration Plan（从零构建，rxrust-first）

1. **pipeline.rs**: 建立 `Local::<str, Infallible>::from_iter(src).pipe(...).last().subscribe(...)` 主干
2. **Tokenizer dst**: 实现 `TokenizerObserver<O, State>` wrapper + `Tokenizer<S, State>`，支持 scan 累积
3. **Parser dst**: 实现 `ParserObserver<O, ASTBuilder>` wrapper + `Parser<S, State>`，产出 Node 流
4. **Evaluator dst**: 实现 `EvaluatorObserver<O, Scope>` wrapper，通过 flat_map 展开 directive
5. **Serializer dst**: 实现 `SerializerObserver<O>` wrapper，map 渲染 CssNode → char
6. **Shared context**: 新增 shared/context.rs，CompilerContext 通过 scan 在管道内传播
7. **模块共享**: evaluate 中使用 `.publish(Local::subject()).ref_count()` 共享模块缓存
8. **tap 桥接 tracing**: 所有 stage 入口/出口使用 tap 注入 span，map/scan 内不直接调用 tracing
9. **tests**: 按 tests/ 下文件补单元测试（`#[rxrust_macro::test(local)]`）
10. **spec walk**: 按 sass-spec 活跃目录逐条实现 directive 算子
11. **enterprise**: Bootstrap + Element Plus 端到端编译验证

---

## Open Questions

- [ ] evaluate.rs 中 directive 算子粒度：单文件 per directive 还是 `flat_map` 内部 enum dispatch？
- [ ] CompilerContext 在 pipe 链中如何跨 stage 传递？scan 携带 vs 独立 subject 订阅 vs channel？
- [ ] `@use ... as *` 符号冲突策略：first-use-wins 还是 error？
- [ ] `!global` 变量写入可见时序如何在 Observable 因果链中保证？（rxrust 的 scan 顺序性天然保证）
- [ ] "Shared" scope（跨线程）是否从 Day 1 即使用 MutArc，还是与 MutRc 通过 type alias 可切换？
