---
name: rxrust
description: Use when working with rxrust reactive streams, Observable patterns, or designing data processing pipelines. This skill covers the two-layer architecture (CoreObservable vs Observable), Context system (Local/Shared), Observer/Emitter traits, operator internals (scan vs scan_map, flat_map = map + merge_all, collect terminal), box_it type erasure, Create source mechanics, Subject broadcasting, Subscription lifecycle, and anti-patterns discovered from source code audits. Use when the user mentions rxrust, reactive streams, Observable, SharedSubject, or when designing stream-based data pipelines in Rust.
---

# RxRust Skill — Source Code Deep Dive

> **基于 rxrust v1.0.0-rc.5 全量源码精读（ops/、observer.rs、context.rs、factory.rs、boxed.rs 等）总结。**

## Architecture Overview

```
Layer 1: CoreObservable<C>  — 纯逻辑内核，泛型 Observer Context
   ├── fn subscribe(self, context: C) -> Unsub
   └── 算子核心：包装下游 Observer，传给上游

Layer 2: Observable (用户 API trait)  — 推断 Item/Err 类型
   ├── fn box_it(self) -> SharedBoxedObservable
   ├── fn map/filter/flat_map/...  → 返回具体 struct (如 Map<S, F>)
   └── 消费 self 传入新算子 struct
```

**双层关系**：`Shared<T>` = `SharedCtx<T, SharedScheduler>` = `{ inner: T, scheduler: SharedScheduler }`。每个 Observable 方法（如 `.map(...)`) 调用 `self.transform(|inner| Map { source: inner, func })`，即在 `inner` 上包裹一层算子 struct。

## Context System — 源码级理解

```rust
// context.rs L170-184
pub struct LocalCtx<T, S> {   // inner=T, scheduler=S
    pub inner: T,
    pub scheduler: S,
}
pub struct SharedCtx<T, S> {   // Send+Sync 版本
    pub inner: T,
    pub scheduler: S,
}
pub type Shared<T> = SharedCtx<T, SharedScheduler>;
pub type Local<T> = LocalCtx<T, LocalScheduler>;
```

| Context | RcMut | RcCell | 内部指针 | 用途 |
|---------|-------|--------|---------|------|
| `Local` | `MutRc<T> = Rc<RefCell<T>>` | `CellRc<T> = Rc<Cell<T>>` | 无需 Send | WASM / 单线程 UI |
| `Shared` | `MutArc<T> = Arc<Mutex<T>>` | `CellArc<T> = Arc<AtomicCell<T>>` | 需要 Send+Sync | 多线程服务端 |

**rx-scss 选择 `Shared`**：全 `'static`、`Send + Sync`，可跨线程调度。

## Observer vs Emitter — 核心区分（易混淆！）

```rust
// observer.rs L19-43 — Observer trait
pub trait Observer<Item, Err> {
    fn next(&mut self, value: Item);     // &mut self，可多次调用
    fn error(self, err: Err);             // 消费 self，终止
    fn complete(self);                    // 消费 self，终止
    fn is_closed(&self) -> bool;          // &self 查询
}

// observer.rs L95-99 — Emitter trait  
pub trait Emitter<Item, Err> {
    fn next(&mut self, value: Item);
    fn error(&mut self, err: Err);        // &mut self — 主要区别！
    fn complete(&mut self);               // &mut self — 主要区别！
    // ⚠️ 注意：Emitter 没有 is_closed() 方法！
}
```

**`Emitter` 存在的理由**（源码注释 L56-72）：
- 支持 `&mut dyn Emitter` —— 无需 `Box<dyn Observer>` 堆分配
- 上游 Observable 无需知道下游具体 Observer 类型
- 零成本类型擦除：避免每次 create 都 Box 分配 Observer

## Create 算子 — 源码级理解

```rust
// factory.rs L147-153
fn create<Item, Err, F, U>(f: F) -> Self::With<Create<F, Item, Err>>
where
    F: FnOnce(&mut dyn Emitter<Item, Err>) -> U,
    U: Subscription,   // ⚠️ 闭包返回值必须实现 Subscription！
{
    Self::lift(Create::new(f))
}
```

```rust
// create.rs L60-72
impl<C, F, Item, Err, U> CoreObservable<C> for Create<F, Item, Err>
where
    C: Context, C::Inner: Observer<Item, Err>,
    F: FnOnce(&mut dyn Emitter<Item, Err>) -> U,
    U: Subscription,
{
    type Unsub = U;
    fn subscribe(self, context: C) -> Self::Unsub {
        let observer = context.into_inner();      // 从 Context 解包具体 Observer
        let mut emitter = CreateEmitter(Some(observer));  // 包装为 Emitter
        (self.f)(&mut emitter)                    // 调用用户闭包，返回 Subscription
    }
}
```

**关键发现**：
1. **闭包参数是 `&mut dyn Emitter`** —— 没有 `is_closed()`，只有 `next`/`error`/`complete`
2. **闭包返回值必须实现 `Subscription`** —— 可以用 `()`（空 tuple 已实现 Subscription）
3. **`CreateEmitter` 内部是 `Option<O>`** —— `error`/`complete` 会 take() 该 Option，确保只调用一次
4. **`is_closed` 存在于 Observer trait，但不在 Emitter trait** —— 所以 `Shared::create` 内部无法检查下游是否关闭

## 算子内部模式 — Observer 包装

每个算子遵循相同的 **Observer 包装模式**：

```rust
// 以 scan_map.rs L50-72 为例
pub struct ScanMapObserver<O, F, Acc> {
    observer: O,        // 下游 observer
    func: F,            // 用户闭包
    acc: Acc,           // 累积状态
}

impl<O, F, Item, Acc, Output, Err> Observer<Item, Err> 
    for ScanMapObserver<O, F, Acc>
where
    O: Observer<Output, Err>,
    F: FnMut(&mut Acc, Item) -> Output,
{
    fn next(&mut self, value: Item) {
        let output = (self.func)(&mut self.acc, value);  // &mut Acc 零 clone
        self.observer.next(output);                       // 转发给下游
    }
    fn error(self, err: Err) { self.observer.error(err); }    // 透传
    fn complete(self) { self.observer.complete(); }           // 透传
    fn is_closed(&self) -> bool { self.observer.is_closed() } // 透传
}
```

## scan vs scan_map — 源码级差异（极其重要！）

```rust
// scan.rs L57-70 —— 每帧 clone！
fn next(&mut self, value: Item) {
    self.acc = (self.func)(self.acc.clone(), value);   // clone acc 作为输入
    self.observer.next(self.acc.clone());               // clone acc 作为输出
}
// 要求: F: FnMut(Output, Item) -> Output, Output: Clone

// scan_map.rs L57-67 —— 零 clone
fn next(&mut self, value: Item) {
    let output = (self.func)(&mut self.acc, value);   // &mut Acc 直接修改
    self.observer.next(output);
}
// 要求: F: FnMut(&mut Acc, Item) -> Output  （Acc 和 Output 可不同！）
```

**决策规则**：
| 场景 | 选择 | 原因 |
|------|------|------|
| Acc = Output 且类型 Clone 成本极低（如 i32） | `scan` | 简单，语义清晰 |
| Acc ≠ Output | `scan_map` | scan 类型系统不允许 |
| Acc 是 Vec/HashMap/复杂结构 | `scan_map` | 避免每帧全量 clone |
| rx-scss eval 器（frames: Vec<Frame>） | `scan_map` | 深度嵌套规则不 clone |

## flat_map — map + merge_all 的语法糖

```rust
// flat_map.rs L44-58 —— flat_map 的 subscribe 实现
impl<S, F, Inner, C> CoreObservable<C> for FlatMap<S, F, Inner>
where ... 
{
    fn subscribe(self, context: C) -> Self::Unsub {
        MergeAll {
            source: Map { source: self.source, func: self.func },
            concurrent: self.concurrent,
        }
        .subscribe(context)   // 委托给 MergeAll
    }
}
```

**并发控制**：
- `concurrent: usize::MAX`（默认）= 无界并发，所有内部 Observable 立即订阅
- `concurrent: 1` = 顺序订阅（concat_map 行为）
- `concurrent: n` = 最多 n 个同时活跃的内部订阅

**内部 MergeAll 状态机**（merge_all.rs L42-55）：
```rust
pub struct MergeAllState<O, InnerObs> {
    observer: Option<O>,
    pending_observables: VecDeque<InnerObs>,  // 排队队列
    subscribed: usize,                        // 当前活跃数
    concurrent: usize,
    outer_completed: bool,
}
```

## collect 终端算子

```rust
// collect.rs L36-58
fn next(&mut self, value: Item) {
    self.collection.extend(Some(value));   // 累积，不发射
}
fn complete(mut self) {
    self.observer.next(self.collection);   // 终止时发射整个集合
    self.observer.complete();
}
```

**关键**：`collect::<Vec<_>>()` 是**终止算子** ——订阅后会收到单个 `Vec<T>` 项，不是逐个元素。

## from_iter — 不是魔法，就是 for 循环

```rust
// from_iter.rs L62-74
fn subscribe(self, context: C) -> Self::Unsub {
    let mut observer = context.into_inner();
    for item in self.iter {
        if observer.is_closed() { break; }   // 背压检查
        observer.next(item);
    }
    if !observer.is_closed() { observer.complete(); }
}
```

**零开销**：直接 for loop，比手动 `vec.into_iter()` 无任何额外成本。

## box_it — 类型擦除机制

```rust
// observable.rs L1944-1949
fn box_it<'a, 'b>(self) -> Self::With<Self::BoxedCoreObservable<'a, Self::Item<'b>, Self::Err>>
where
    Self::Inner: IntoBoxedCoreObservable<Self::BoxedCoreObservable<'a, Self::Item<'b>, Self::Err>>,
{
    self.transform(|inner| inner.into_boxed())
}
```

// boxed.rs L433-434
```rust
pub type SharedBoxedObservable<'a, Item, Err = ()> =
    crate::context::Shared<BoxedCoreObservableSend<'a, Item, Err, SharedScheduler>>;
```

**`box_it` 做了什么**：将链式嵌套的泛型类型（如 `Map<Filter<FromIter<...>>, F>`）擦除为 `Shared<Box<dyn DynCoreObserver>>`，统一类型以便存储在函数签名/Vec 中。

## 扫描规则：何时 box_it

**Rule: `box_it()` 只在函数返回边界调用一次。**

```rust
// ❌ 反模式 —— 算子链中间多次 box_it
fn bad_example() -> SharedBoxedObservable<'static, CssStmt, Infallible> {
    Shared::from_iter(nodes)
        .flat_map(emit_events)
        .box_it()          // 第一次 —— 不必要！
        .scan_map(...)
        .box_it()          // 第二次 —— 不必要！
        .filter_map(|x| x)
        .box_it()          // 第三次 —— 唯一需要的
}

// ✅ 正确 —— 只在最终边界 box_it
fn good_example() -> CssStream {
    Shared::from_iter(nodes)
        .flat_map(emit_events)
        .scan_map(EvalState::root(), fold_frame)
        .filter_map(|opt| opt)
        .box_it()          // 唯一一次 —— 返回给上游调用者
}
```

**唯一需要 box_it 的场景**：
1. 函数需要返回统一类型（擦除内部算子嵌套类型）
2. 存储到 `Vec<SharedBoxedObservable<...>>`
3. 递归 Observable 的返回边界（match arm 类型不一致时）

## SharedSubject — 广播机制

```rust
// subject_core.rs L140
pub type SharedSubject<'a, Item, Err> = Shared<InnerSubjectSend<'a, Item, Err>>;
```

**使用模式**：
```rust
// 创建
let subject: SharedSubject<'static, VarEvent, Infallible> = Shared::subject();

// 发射（手动推送）
subject.next(VarEvent::Bind { name: "x".into(), val: Value::Number(42.0, None) });

// 订阅（转换为 Observable 链）
subject
    .filter_map(|e| match e { VarEvent::Bind { name, val } => Some((name, val)), _ => None })
    .subscribe(|(name, val)| tracing::debug!(%name, ?val, "var bound"));

// 共享给多个消费者
let subject2 = subject.clone();
subject2.subscribe(|e| { /* 另一个处理分支 */ });
```

## Subscription 与生命周期

```rust
// 空 Subscription（用于 Shared::create 返回值）
Shared::create(|emitter| {
    emitter.next(1);
    emitter.complete();
    ()  // () 已实现 Subscription trait
}).subscribe(|v| { ... });

// RAII 自动取消订阅
let sub = stream.subscribe(|v| { ... });
let _guard = sub.unsubscribe_when_dropped();  // 持有 guard 期间活跃
// _guard 离开作用域 → Observable 取消订阅
```

## 算子速查表

| 算子 | 类型签名 | 内部行为 | Clone 成本 | 何时用 |
|------|---------|---------|-----------|--------|
| `map` | `T -> U` | 逐项转换 | 无 | 简单 1:1 映射 |
| `filter` | `T -> bool` | 条件通过 | 无 | 不需要的元素 |
| `filter_map` | `T -> Option<U>` | map+filter 合并 | 无 | 转换同时过滤 |
| `flat_map` | `T -> Observable<U>` | map + merge_all(MAX) | 无 | 1:N 展平，需并发 |
| `concat_map` | `T -> Observable<U>` | map + merge_all(1) | 无 | 1:N 展平，保序 |
| `expand` | `T -> Observable<T>` | 自引用 merge_all | 无 | 递归树展开 |
| `scan` | `(State, T) -> State` | 累积并发射中间态 | **每帧 clone** | Acc=Output 且廉价 |
| `scan_map` | `(&mut Acc, T) -> Output` | 累积并转换 | **零 clone** | 大状态、Acc≠Output |
| `reduce` | `(State, T) -> State` | 折叠最终值 | 无（终止） | 聚合为单个值 |
| `collect<Vec>` | 终止算子 | 累积全部后发射 Vec | 无（终止） | 流 → Vec 一次性 |
| `merge` | (Obs, Obs) -> Obs | 交错合并 | 无 | 合并两个流 |
| `tap` | `T -> ()` | 副作用 | 无 | 调试、日志 |
| `take(n)` | 取前 n | 计数关闭 | 无 | 限制数量 |

## rx-scss Pipeline 标准模式

```
Source(String)
    │
    ▼ Shared::create(lexer) + box_it()          ← Lexer 阶段边界
TokenStream = SharedBoxedObservable<'static, Token, Infallible>
    │
    ▼ .collect::<Vec<_>>().into_iter().next()   ← 终止+提取 Vec
Vec<Token>
    │
    ▼ parse_stream(tokens) + box_it()            ← Parser 阶段边界
AstStream = SharedBoxedObservable<'static, AstNode, Infallible>
    │
    ▼ .flat_map(emit_events)                     ← AST → EvalEvents
    ▼ .scan_map(EvalState::root(), fold_frame)   ← &mut 零 clone 累积
    ▼ .filter_map(|opt| opt)                   ← 提取完成的 CssStmt
    ▼ .box_it()                                  ← Evaluator 唯一边界
CssStream = SharedBoxedObservable<'static, CssStmt, Infallible>
    │
    ▼ .collect::<Vec<_>>() + serialize          ← 终止+序列化
String (CSS 输出)
```

**多入口共享管线**（`build_css_stream`）：
```rust
fn build_css_stream(nodes: Vec<AstNode>, ctx: Arc<EvalContext>, bus: CompilerBus) -> CssStream {
    Shared::from_iter(nodes)
        .flat_map(move |node| emit_events(node, None, ctx.clone(), bus.clone()))
        .scan_map(EvalState::root(), fold_frame)
        .filter_map(|opt| opt)
        .box_it()   // 唯一 type erasure 边界
}
```

## 自定义源（Shared::create）正确使用

```rust
// ✅ 正确：Shared::create 用于自定义发射逻辑
pub(crate) fn emit_events(
    node: AstNode,
    parent_sel: Option<String>,
    ctx: Arc<EvalContext>,
    bus: CompilerBus,
) -> Shared<Create<impl FnOnce(&mut dyn Emitter<EvalEvent, Infallible>) -> (), EvalEvent, Infallible>> {
    Shared::create(move |emitter| {
        // emitter: &mut dyn Emitter —— 只能 next/error/complete
        // ⚠️ 不能调用 emitter.is_closed() —— Emitter trait 没有此方法！
        emit_ast_node(&node, parent_sel, &ctx, &bus, emitter);
        emitter.complete();
        ()  // 返回空 Subscription
    })
    // 注意：这里不 box_it！让调用者（flat_map）统一处理类型
}

fn emit_ast_node(
    node: &AstNode,
    parent_sel: Option<String>,
    ctx: &Arc<EvalContext>,
    bus: &CompilerBus,
    emitter: &mut dyn Emitter<EvalEvent, Infallible>,  // 直接传递 Emitter
) {
    match node {
        AstNode::Rule { selector, inner } => {
            emitter.next(EvalEvent::EnterRule(combined.clone()));
            for child in inner {
                emit_ast_node(child, Some(combined.clone()), ctx, bus, emitter);
            }
            emitter.next(EvalEvent::LeaveRule);
        }
        AstNode::Terminal(stmt) => {
            emitter.next(EvalEvent::Terminal(stmt.clone()));
        }
    }
}
```

```rust
// ❌ 错误：手动模拟响应式，声称"响应式"
fn bad_custom_source() -> impl Observable<Item = EvalEvent> {
    let mut events = Vec::new();        // 预先收集——不是惰性的！
    collect_events_recursive(node, &mut events);
    Shared::from_iter(events).box_it()  // 绕过了响应式管道
}

// ❌ 错误：Arc<Mutex<Vec>> + subscribe 收集（GC 思维）
let results = Arc::new(Mutex::new(Vec::new()));
stream.subscribe(|x| results.lock().unwrap().push(x));  // 反模式！
let final = results.lock().unwrap().clone();
```

## 反模式清单（基于源码审计）

| ❌ 反模式 | ✅ 正确做法 | 源码依据 |
|----------|-----------|----------|
| `&mut dyn Emitter` 上调用 `.is_closed()` | 在 `from_iter`、`take` 等用 `observer.is_closed()` | Emitter trait 仅为 `next`/`error`/`complete` |
| `Shared::create` 闭包不返回 `Subscription` | 返回 `()` 或 `ClosureSubscription` | `create.rs` 约束 `U: Subscription` |
| `scan` 累积 `Vec<Frame>` 大状态 | 用 `scan_map` —— `&mut self.acc` 零 clone | scan.rs L68 有两次 `.clone()` |
| `flat_map` 闭包内 `Vec::new()` + push + `from_iter` | 闭包直接返回 `impl Observable`（惰性） | `flat_map` = `Map` + `MergeAll`，原生支持惰性 |
| 每个算子后 `.box_it()` | 只在返回边界调用一次 | `box_it` 调用 `into_boxed()` 引入 `Box<dyn DynCoreObservable>` |
| subscribe 闭包收集到 `Arc<Mutex<Vec>>` | 终端 `.collect::<Vec<_>>()` | collect.rs 用 `Extend` 一次性累积 |
| `for x in items { result.push(f(x)) }` | `items.into_iter().map(f).collect()` | 函数式风格，无副作用 |
| `if-else` 链分派枚举 | `match` 表达式 | Rust 习惯，编译器检查穷尽 |
| `match Err(e) => return Err(e)` | `?` 传播 | 简洁，保持一致 |
| GC 思维：修改状态后返回原引用 | move 语义 + 返回新值或 `(T, State)` | Rust 所有权规则 |

## 类型系统要点——避免编译错误

### `Shared<T>` 只接受 1 个泛型参数

```rust
// ❌ 错误：Shared 是 SharedCtx 的别名，只接受一个参数
fn bad() -> Shared<impl CoreObservable<Shared, Item<'static> = EvalEvent, Err = Infallible>> { ... }
// Shared<T> = SharedCtx<T, SharedScheduler> —— 不接受 3 个参数！

// ✅ 正确：Shared 包装一个 CoreObservable 类型
fn good() -> Shared<Create<F, EvalEvent, Infallible>> { ... }

// ✅ 正确：用 impl Observable 让编译器推断
fn good2() -> impl Observable<Item = EvalEvent, Err = Infallible> { ... }
```

### `&mut dyn Emitter` 不能调用 `is_closed`

```rust
// ❌ 编译错误：Emitter trait 没有 is_closed 方法
Shared::create(|emitter| {
    while !emitter.is_closed() {  // E0599！
        emitter.next(something);
    }
});

// ✅ 正确：from_iter 内部用 observer.is_closed() —— 具体 Observer 类型
// ✅ 正确：自定义 create 不检查 is_closed，依赖下游 unsubscribe 触发 Subscription teardown
```

## Quick Start —— 30 秒上手

```rust
use rxrust::prelude::*;
use std::convert::Infallible;

// 创建 Observable
let stream = Shared::from_iter(vec![1, 2, 3, 4, 5])
    .filter(|x| x % 2 == 0)
    .map(|x| x * 10)
    .scan_map(0i32, |sum, x| { *sum += x; *sum });

// 终端收集
let results: Vec<i32> = stream
    .collect::<Vec<_>>()
    .into_iter()
    .next()
    .unwrap_or_default();
// results = [20, 60]  (2*10=20, sum=20; 20+4*10=60, sum=60)
```

## OpenTelemetry Tracing 集成

```rust
// 初始化（生产环境）
rx_scss::telemetry::init_tracing();

// 初始化（测试环境，配合 --nocapture）
rx_scss::telemetry::init_test_tracing();

// 运行调试
// RUST_LOG=debug cargo test test_name -- --nocapture
// RUST_LOG=rx_scss=trace cargo run -- input.scss

// 在代码中创建 span
#[tracing::instrument(skip(bus), fields(node_count = nodes.len()))]
fn eval_nodes(nodes: Vec<AstNode>, bus: &CompilerBus) -> Result<Vec<CssStmt>> {
    let _span = tracing::info_span!("parse_at_if").entered();
    tracing::debug!(?cond_val, is_truthy, "condition evaluated");
    // ...
}
```

## 参考

- **rxRust 官方文档**: https://rxrust.github.io/rxRust/
- **rx-scss 项目内文档**: `.claude/skills/rxrust/SKILL.md`
- **rxrust 源码**: `~/.cargo/registry/src/*/rxrust-1.0.0-rc.5/src/`
- **关键源文件**:
  - `observer.rs` —— Observer/Emitter trait 定义
  - `context.rs` —— Context/LocalCtx/SharedCtx 实现
  - `factory.rs` —— create/of/empty 工厂方法
  - `ops/scan.rs` —— scan 每帧 clone
  - `ops/scan_map.rs` —— scan_map &mut 零 clone
  - `ops/flat_map.rs` —— flat_map = map + merge_all
  - `ops/merge_all.rs` —— 并发控制队列
  - `ops/collect.rs` —— 终止算子
  - `observable/create.rs` —— Create 源实现
  - `observable/from_iter.rs` —— for loop 实现
  - `observable/boxed.rs` —— box_it 类型别名
  - `subject/subject_core.rs` —— Subject 广播
