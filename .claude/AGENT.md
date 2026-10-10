# rxrust Agent Guide

> rxrust 1.0.0-rc.5 响应式编程库完整参考 — 基于源码分析

## 1. 核心架构

### 1.1 双上下文模型

rxrust 通过 **Context** trait 区分两种执行环境：

| 上下文 | 智能指针 | 线程安全 | 调度器 | 典型用途 |
|--------|----------|----------|--------|----------|
| `Local` | `Rc<RefCell<T>>` | 单线程 | `LocalScheduler` | 同步流水线、解析器 |
| `Shared` | `Arc<Mutex<T>>` | `Send + Sync` | `SharedScheduler` | 跨线程广播、模块系统 |

```rust
// Local: 单线程，轻量
let local_stream = Local::of(42).map(|x| x * 2);

// Shared: 跨线程，广播
let shared_subject = Shared::subject::<i32, CompileError>();
shared_subject.clone().subscribe(|v| tracing::info!("thread A: {}", v));
// 在另一个线程:
shared_subject.next(1); // thread A 会收到
```

### 1.2 三大核心 Trait

```rust
// Observable: 流的抽象，提供算子方法
trait Observable {
    type Item<'a>;
    type Err;
    type Inner;
    fn flat_map<F, Inner>(self, f: F) -> FlatMap<Self::Inner, F, Inner>;
    fn filter_map<F>(self, f: F) -> FilterMap<Self::Inner, F>;
    fn scan_map<S, F>(self, init: S, f: F) -> ScanMap<Self::Inner, S, F>;
    fn collect<'a, C>(self) -> Collect<Self::Inner, C>;
    fn box_it(self) -> Self::With<Self::BoxedCoreObservable<'a, Self::Item<'b>, Self::Err>>;
    // ...
}

// Observer: 消费流发射的值
trait Observer<Item, Err> {
    fn next(&mut self, value: Item);
    fn error(self, err: Err);
    fn complete(self);
    fn is_closed(&self) -> bool;
}

// Subscription: 可取消的订阅句柄
trait Subscription {
    fn unsubscribe(self);
    fn is_closed(&self) -> bool;
}
```

## 2. 创建 Observable

### 2.1 工厂方法（`ObservableFactory` trait）

所有方法通过 `Local::` 或 `Shared::` 调用：

```rust
use rxrust::prelude::*;

// ── 值构造 ──
Local::of(42)                              // 发射单个值后完成
Local::from_iter(vec![1, 2, 3])            // 从迭代器逐个发射
Local::from_iter(0..5)                     // Range 也支持
Local::from_fn(|| compute_value())         // 订阅时惰性求值
Local::empty::<i32>()                      // 立即完成，无值
Local::never::<i32>()                      // 永不发射，永不完成
Local::throw_err("error".to_string())      // 立即错误

// ── 闭包构造（自定义发射逻辑）──
Local::create(|emitter| {
    emitter.next(1);
    emitter.next(2);
    emitter.complete();                    // 必须手动完成
    ()                                     // 返回 Subscription（unit 类型即可）
})

Shared::create(|emitter| {
    emitter.next("broadcast");
    emitter.complete();
    ()
})
```

### 2.2 闭包签名约束

`create` 的闭包签名：

```rust
fn create<Item, Err, F, U>(f: F) -> Self::With<Create<F, Item, Err>>
where
    F: FnOnce(&mut dyn Emitter<Item, Err>) -> U,
    U: Subscription,
```

- `FnOnce`：每个订阅调用一次
- `&mut dyn Emitter`：通过 `emitter.next()` / `emitter.complete()` / `emitter.error()` 发射
- 返回值实现 `Subscription`：通常返回 `()`（unit 已实现 Subscription）

### 2.3 BehaviorSubject

```rust
// 需要初始值，新订阅者会立即收到最新值
let bhv = Local::behavior_subject::<i32, CompileError>(0);
bhv.clone().subscribe(|v| tracing::info!("got: {}", v)); // 立即打印 "got: 0"
bhv.next(42);
bhv.clone().subscribe(|v| tracing::info!("got: {}", v)); // 立即打印 "got: 42"
```

## 3. Subject（多播）

### 3.1 类型对应

| 类型 | Local | Shared |
|------|-------|--------|
| Subject | `Local::subject()` / `LocalSubject` | `Shared::subject()` / `SharedSubject` |
| BehaviorSubject | `Local::behavior_subject(init)` | `Shared::behavior_subject(init)` |

### 3.2 Subject API

```rust
let subject = Shared::subject::<MyEvent, CompileError>();

// subscribe 返回 Subscription 句柄
let sub = subject.clone().subscribe(|evt| { /* ... */ });

// 发射
subject.clone().next(MyEvent::Click);     // 广播给所有订阅者
subject.clone().complete();               // 通知所有订阅者流结束
subject.clone().error(MyError::Failed);   // 通知错误

// 订阅者管理
sub.unsubscribe();                        // 取消单个订阅
```

### 3.3 ⚠️ 不可重入（Re-Entrancy）

```rust
// ❌  panic! — 在 subscribe 回调内部调用 next 会导致 panic
subject.clone().subscribe(|evt| {
    subject.clone().next(evt); // 运行时 panic!
});
```

如果必须反馈循环，使用 `delay` 引入异步边界。

## 4. 算子（Operators）

### 4.1 转换算子

```rust
stream
    .map(|x| x * 2)                    // 1:1 映射
    .map_to("constant")                // 忽略原值，发射固定值
    .filter(|x| x > &10)               // 过滤
    .filter_map(|x| {                  // 过滤 + 映射
        if x > 10 { Some(x * 2) } else { None }
    })
    .scan_map(init_state, |s, val| {   // 有状态的折叠（类似 fold）
        s.update(val);
        s.output()                     // 返回每个输入对应的输出
    })
    .flat_map(|x| Local::from_iter(..)) // 1:N 展平（返回 Observable）
    .switch_map(|x| fetch_async(x))    // 取消前一个内部流，切换到新流
```

### 4.2 聚合算子

```rust
stream
    .collect::<Vec<_>>()               // 收集所有值为 Vec（源完成时发射）
    .reduce(|acc, x| acc + x)          // 归约（源完成时发射最终值）
    .default_if_empty(default)         // 源为空时发射默认值
```

### 4.3 截取/跳过

```rust
stream
    .take(5)                           // 取前 5 个
    .take_while(|x| x < 100)           // 取直到条件不满足
    .take_until(other_stream)          // 取直到另一个流发射
    .skip(3)                           // 跳过前 3 个
    .skip_while(|x| x == 0)            // 跳过直到条件不满足
    .last()                            // 只取最后一个
```

### 4.4 Lifecycle / Tap

```rust
stream
    .tap(|x| tracing::debug!("debug: {}", x)) // 不改变流，仅用于副作用
    .finalize(|| cleanup())            // 完成或错误时执行清理
```

### 4.5 组合

```rust
Local::combine_latest(stream_a, stream_b)    // 任一发射时取最新配对
Local::merge(stream_a, stream_b)             // 合并两个流
Local::zip(stream_a, stream_b)               // 拉链配对
stream.with_latest_from(other)               // 从流使用主流触发
```

## 5. 类型擦除与 Boxed Observable

### 5.1 为什么需要擦除

算子链（如 `flat_map` 后的 `filter_map`）产生嵌套泛型类型。函数边界需要统一类型时，必须擦除。

### 5.2 Boxed 类型

| 类型别名 | Context | Clone |
|----------|---------|-------|
| `LocalBoxedObservable` | Local | ❌ |
| `LocalBoxedObservableClone` | Local | ✅ |
| `SharedBoxedObservable` | Shared | ❌ |
| `SharedBoxedObservableClone` | Shared | ✅ |

```rust
// 定义类型别名时使用
pub type AstStream = SharedBoxedObservable<'static, AstNode, CompileError>;
```

### 5.3 `box_it()` vs `box_it_clone()`

```rust
// .box_it() — 转换为非 Clone 的 boxed 类型
let boxed: SharedBoxedObservable<AstNode, CompileError> =
    stream.flat_map(|x| Shared::of(x)).box_it();

// .box_it_clone() — 转换为 Clone 的 boxed 类型
// ⚠️ 算子链后不能直接 .box_it_clone()
let boxed: SharedBoxedObservableClone<AstNode, CompileError> =
    Shared::of(node).box_it_clone();
```

### 5.4 ⛔ 关键陷阱：算子链后不能 `.box_it_clone()`

```rust
// ❌ 编译失败：FlatMap<Box<dyn ... + Send>, F, Inner> 不满足 Clone
let result: SharedBoxedObservableClone<AstNode, CompileError> =
    stream.flat_map(|x| Shared::of(x)).box_it_clone();  // ERROR!

// ✅ 正确：使用 .box_it() + SharedBoxedObservable
let result: SharedBoxedObservable<AstNode, CompileError> =
    stream.flat_map(|x| Shared::of(x)).box_it();        // OK
```

## 6. 订阅模式

### 6.1 基本订阅

```rust
stream.subscribe(|value| {
    tracing::info!("got: {}", value);
});
```

### 6.2 同步收集结果 — 使用 collect 算子

```rust
// ✅ 正确: 使用 collect 终止算子 + extract
let nodes: Vec<AstNode> = ast_stream
    .collect::<Vec<_>>()
    .into_iter()
    .next()
    .unwrap_or_default();
```

### 6.3 带生命周期的 Subject 订阅

```rust
let subject = Shared::subject::<MyEvent, CompileError>();
let sub = subject.clone().subscribe(|evt| {
    handle_event(evt);
});
// 稍后取消订阅
sub.unsubscribe();
```

## 7. 线程安全约束

### 7.1 Shared 上下文要求

```rust
// ✅ AstNode 的所有字段都是 Send + Sync
pub enum AstNode {
    VariableDecl { name: String, value: Value },  // String + Value = Send + Sync
    RuleSet { selector: String, inner: Vec<AstNode> },
    // ...
}

// ❌ 如果包含 Rc，编译失败
pub enum BadNode {
    State(Rc<RefCell<Data>>),  // Rc 不是 Send
}
```

### 7.2 闭包捕获

```rust
// Shared::create 闭包必须 Send
let shared_data = Arc::new(Mutex::new(0));  // Arc = Send + Sync
let stream = Shared::create(move |emitter| {
    let mut data = shared_data.lock().unwrap();
    emitter.next(*data);
    emitter.complete();
    ()
});
```

## 8. 算子分类速查

### 创建
`create`, `of`, `from_iter`, `from_fn`, `empty`, `never`, `throw_err`, `defer`, `timer`, `interval`, `from_future`, `from_stream`

### 转换
`map`, `map_to`, `map_err`, `filter`, `filter_map`, `scan`, `scan_map`, `flat_map`, `switch_map`, `reduce`, `collect`

### 截取
`take`, `take_last`, `take_while`, `take_until`, `skip`, `skip_last`, `skip_while`, `skip_until`, `last`, `contains`

### 组合
`merge`, `merge_all`, `zip`, `combine_latest`, `with_latest_from`, `start_with`, `pairwise`, `group_by`

### 调度和生命周期
`observe_on`, `subscribe_on`, `delay`, `debounce`, `throttle`, `sample`, `tap`, `finalize`, `retry`, `default_if_empty`, `distinct`, `distinct_until_changed`, `buffer`, `buffer_count`, `buffer_time`, `average`

### 多播
`ref_count`, `fork`, `connect`

## 9. 设计模式

### 9.1 Shared::create 桥接上游流（正确模式）

当需要从上游 Observable 桥接并转换数据到下游时，`Shared::create` 是标准做法：

```rust
fn bridge_stream(input: TokenStream) -> AstStream {
    Shared::create(move |subscriber| {
        input.subscribe_all(
            move |tok| {
                for node in parser_feed(&mut state, tok) {
                    subscriber.next(node);
                }
            },
            move |err| { subscriber.error(err); },
            move || { subscriber.complete(); },
        );
        ()  // empty Subscription
    }).box_it()
}
```

**要点**:
- `subscribe_all` 的 `on_next` 调用 `subscriber.next()` 转发数据
- `on_error` 调用 `subscriber.error()` 传播错误
- `on_complete` 调用 `subscriber.complete()` 或检查最终状态后 error
- `box_it()` 仅在最终边界调用一次

### 9.2 同步有界迭代优先用 `Shared::from_iter`

```rust
// ✅ 对已知 Vec 使用 from_iter + iterator combinators
let nodes: Vec<AstNode> = (0..iterations)
    .flat_map(|_| extract_css(&body, child_ctx.clone()))
    .collect();
Shared::from_iter(nodes).box_it()

// ✅ 单个值用 of
Shared::of(AstNode::Css(stmt)).box_it()

// ✅ 空流用 create + complete
Shared::create(|s| { s.complete(); }).box_it()
```

### 9.3 跨模块类型别名共享

```rust
// types.rs — 定义统一的流类型
pub type AstStream = SharedBoxedObservable<'static, AstNode, CompileError>;
pub type CssStream = SharedBoxedObservable<'static, CssStmt, CompileError>;

// 所有地方使用统一类型，避免泛型爆炸
pub fn evaluate(stream: AstStream, ctx: Arc<EvalContext>) -> CssStream { ... }
```

## 10. 调试技巧

### 10.1 使用 `tap` 插桩

```rust
stream
    .tap(|x| tracing::debug!(value = ?x, "after filter"))
    .flat_map(|x| /* ... */)
```

### 10.2 使用 `scan_map` 追踪状态

```rust
stream
    .scan_map(ParserState::new(), |state, token| {
        tracing::debug!(token = ?token, "parse step");
        state.feed(token);
        state.try_parse()
    })
```

## 11. 与 `tracing` 集成

```rust
use tracing::{debug_span, info_span};

fn process_node(node: AstNode, ctx: Arc<EvalContext) -> CssStream {
    let span = info_span!("process_node", node = ?node, scope_id = ctx.scope_id);
    let _guard = span.entered();

    match node {
        AstNode::Rule { selector, inner } => {
            let span = debug_span!("rule_set", %selector);
            let _guard = span.entered();
            // ...
        }
    }
}
```

## 12. 常见编译错误与解决

| 错误 | 原因 | 解决 |
|------|------|------|
| `Box<dyn DynCoreObservableClone + Send>: Clone` 未满足 | 算子链后使用 `.box_it_clone()` | 改用 `.box_it()` + `SharedBoxedObservable` |
| `Rc<RefCell<T>>` 不能跨线程安全发送 | Shared 上下文使用了非 Send 类型 | 改用 `Arc<Mutex<T>>` |
| `expected SharedCtx, found LocalCtx` | Local 和 Shared 类型混用 | 统一为 `Shared::`（多播场景） |
| `Local::(())` 语法错误 | 无参构造 | 改用 `Local::of(())` 或 `Local::create(\|_\| ())` |
| `Observable::create` 不存在 | API 改名 | 使用 `Local::create` / `Shared::create` |
| `Infallible` 不再使用 | 错误通道已迁移到 `CompileError` | 替换为 `CompileError` |

## 13. rx-scss 反模式防御（2026-10 更新）

### 13.1 禁止 subscribe-collect GC 模式

```rust
// ❌ 禁止: Arc<Mutex<Vec>> + subscribe(push) = GC thinking + lock overhead
let result = Arc::new(Mutex::new(None));
let r = result.clone();
collected.subscribe(move |stmts| { *r.lock().unwrap() = Some(stmts); });

// ✅ 必须: 使用 collect 终止算子
let nodes: Vec<AstNode> = ast_stream
    .collect::<Vec<_>>()
    .into_iter()
    .next()
    .unwrap_or_default();
```

### 13.2 禁止命令式模拟 rxrust 算子

```rust
// ❌ 禁止: 手动 queue + while pop（非响应式）
let mut queue: Vec<Work> = ...;
while let Some(work) = queue.pop() {
    match work { /* 手动展开 recursive */ }
}

// ✅ 必须: 算子链
Shared::from_iter(ast_nodes)
    .expand(emit_eval_events)              // 递归展开 AST 树
    .scan_map(EvalState::root(), fold)     // &mut 零 clone 状态累积
    .filter_map(emit_completed)           // 过滤 + 映射
    .box_it();
```

### 13.3 禁止 box_it() 滥用

```rust
// ❌ 禁止: 每个节点都 box_it
fn emit_events(node: AstNode) -> CssStream {
    let mut events = Vec::new();
    collect_events(node, &mut events);
    events.reverse();
    Shared::from_iter(events).box_it()  // 每个 AST 节点一次擦除!
}

// ✅ 正确: flat_map/expand 闭包返回 lazy Observable，只在最终边界 box_it
fn emit_events(node: AstNode) -> impl Observable<Item = EvalEvent> {
    // ... lazy observable
}
```

**原则**: `box_it()` 只在以下位置调用：
1. 函数最终返回 `SharedBoxedObservable` 类型边界处
2. `from_iter([a, b, c])` 内嵌套用统一异构 Observable
3. 存储到 `Vec<BoxedObs>` 时

### 13.4 禁止 src/ 内联测试

所有测试放在 `tests/` 目录，`src/` 保持纯生产代码。

### 13.5 clone 使用红线

优先使用 owned 数据、构造新的 child_scope、将 value move 进 enum。避免不必要的 `clone()`。

## 14. 与 rx-scss 编译器架构的关系

本项目的目标架构（全链路响应式管线，规划中）：

```
Source(String)
    │
    ▼ Shared::create (lexer)
TokenStream (SharedBoxedObservable<'static, Token, CompileError>)
    │
    ▼ Shared::create 桥接 (parser) — subscribe_all 逐 token 增量解析
AstStream (SharedBoxedObservable<'static, AstNode, CompileError>)
    │
    ▼ expand + scan_map + filter_map (eval)
CssStream (SharedBoxedObservable<'static, CssStmt, CompileError>)
    │
    ▼ fold + map (serialize) — 累积后 render
OutputStream (SharedBoxedObservable<'static, String, CompileError>)
    │
    ▼ collect_boxed (终端收集)
String (CSS 输出)
```

**关键设计决策**:
- 整个管线基于 `Shared`（而非 `Local`），使 `@mixin` 注册和 `@use` 模块加载可以通过 `SharedSubject` 跨订阅者广播
- 错误类型统一为 `CompileError`（从 `Infallible` 迁移）
- `expand` 替代 `flat_map` 做 AST 递归展开（深度优先语义正确）
- `scan_map(&mut)` 替代 `scan` 做帧栈累积（每帧 clone → 零 clone）
- `fold` 替代中间 `collect + serialize`
- `box_it()` 仅在阶段边界调用一次

**⚠️ 注意**: 当前代码库（截至 2026-10-10）仍处于 `Infallible` 阶段，openspec `full-reactive-pipeline` 是规划中的重构目标。
