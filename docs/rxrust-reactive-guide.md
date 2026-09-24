# rxrust 响应式编程指南

> **核心认知**：rxrust 不是模仿 Flux/Reactor，是 Rust 所有权模型对响应式问题的天然解答。
> **用途**：让 AI 用 Rust 闭包的 ownership 三态设计管线，用 rxrust 算子组合声明。

---

## 元认知：为什么不能从 Flux 出发

### Reactor (Java) 的 GC 世界

```
Reactor 的核心假设：一切数据都被 GC 管理，任何 lambda 都能自由捕获/共享/修改任意对象。

↓ 这意味着什么？

Java lambda 闭包捕获 = 隐性共享 = 运行时混沌的温床：
  - ConcurrentModificationException
  - 数据竞争 (编译器不阻止)
  - 隐式反馈循环 (Flux.create 内再 onNext)
  - 不确定的修改顺序 (state visible to everyone)

Flux 的所有规则 (no re-entrant subscribe, doOnXxx instead of in-place mut)
都是为了修补 GC 带来的问题。
```

### Rust 的所有权世界

```
Rust 闭包的 capture 是编译期 X 光：

  move  |||||||    所有权转移, 他处不可再访问
  &     |||||||    不可变借用, 读者之间可共享, 但不能写
  &mut  |||||||    可变借用, 编译器保证排他, 无锁无线程安全问题

↓ 这意味着什么？

"自由捕获"这个概念本身在 Rust 不存在。每次闭包捕获时：
  - 编译器知道谁拥有什么
  - 编译器追踪读写冲突到行号级
  - 没有 GC 可以让"错误"偷偷溜过

Rust + rxrust 不是把响应式编码变得"更难而是更难"，
是 Rust ownership 天然实现了 Flux 想做但做不到的事：
编译期零运行时灾难。
```

---

## 第 1 章：rxrust 是 Rust Ownership 的 DSL

### 1.1 响应式算子 = 所有权模式的组合

rxrust 没有一个算子需要 "模仿 Flux"——每个算子都直接是 Rust 闭包语义的逻辑延伸：

| Rust 闭包所有权语义 | rxrust 算子 | 说明 |
|--------------------|-----------|------|
| 消费 `T` 产生`U` | `map(\|x\| -> U)` | 所有权转移：T 被消费，U 给下游 |
| 借用 `&T` 判断 bool | `filter(\|x\| -> bool)` | 不消费，只检查，下游拿到同样的 T |
| `&mut Acc` 就地修改 | `scan_map(init, \|acc, x\| out)` | **核心差异**：零 clone 零锁的状态机 |
| 消费 owning iterator | `from_iter(iter)` | Rust 的 IntoIterator = Observable |
| 异步 .await 结果 | `from_future(fut)` | Future 状态机 = mono-like Observable |
| merge two owned values | `merge(other)` | 两个 owned Observable 合并 |

### 1.2 为什么 Rust 不需要 Flux 的 "Shared" vs "Mono"

Flux 用 Mono 和 Flux 区分语义单值/流值。Rust 不需要：

```rust
// Mono<T> = 异步单值，等价于 Rust 的 Future 或单元素 Iterable
// Flux<T> = 异步流，等价于 Rust 的 Stream/Iterator/Observable

// 区别在于 Rust 不强制运行时抽象
let observable: Shared<_> = Shared::from_iter(0..1);   // 既有单值语义
let observable: Shared<_> = Shared::from_iter(0..100); // 也有流语义
// 类型统一，Semantics 由数据决定
```

Flux 要 Mono 和 Flux 两种类型是因为 GC 无法在编译期保证"只发一次"。
Rust 的 ownership + trait 系统天然保证。

---

## 第 2 章：Rust 闭包的三态 × rxrust 算子

### 2.1 `move`：所有权转移 (Terminal 闭包)

```rust
// subscribe 闭包内的 move 关键字:
.subscribe(move |vec: Vec<String>| {
    // move 之后 tx 不再被外部拥有
    // Drop => channel 关闭
    let _ = tx.send(vec.join("\n"));
});
// tx 在此处不可再用（已被 move 进闭包）
```

### 2.2 `&`：借引用 (Transform 闭包)

```rust
// map 闭包消费 T 返回 U，但 transform 函数可用 &借用中间数据
.map(|node: CssNode| render_node(&node))
//                            ^^^^^
//                            借用 node, 不需要 clone
//                            node 被 map 自动传回给下游
```

### 2.3 `&mut Acc`：状态栖息地 (scan_map 独有)

```rust
// scan_map 是唯一能"持有状态"的地方
// &mut Acc 保证：
//   1. Acc 完全在 Observer 内部，外部不可见
//   2. 每个 subscription 收到独立的 Acc 克隆
//   3. 没有 Rc<RefCell>/Arc<Mutex>，零开销并发安全

.scan_map(CompileState::new(), |state: &mut CompileState, line: String| -> Vec<String> {
    state.line_count += 1;    // &mut 排他修改，编译器保证安全
    state.dispatch(line)      // 返回 Vec<T>
})
```

---

## 第 3 章：React 式 operators 速查表

### 3.1 转换与过滤 (1:1 map/filter)

```rust
// map:FnMut(T) -> U
let doubled = source.map(|x| x * 2);

// filter:FnMut(&T) -> bool
let filtered = source.filter(|x: &i32| x > &10);
```

### 3.2 展平 (1:N flat_map)

```rust
// flat_map 闭包返回 Observable（不是裸 vec/iterator）
source
    .scan_map(CompileState::new(), reducer)          // 产出 Vec<String>
    .flat_map(|v: Vec<String>| Shared::from_iter(v)) // 子流逐个 String
```

**关键约束**：flat_map 的闭包返回值必须实现 `Observable`，
所以 `Vec<T>` 必须用 `Shared::from_iter(vec)` 包装。

### 3.3 汇聚 (complete 时刻发射)

```rust
// collect: 当 source complete 时，emit 集合
.collect::<Vec<String>>()

// last: 只保留最后一次采集的值
.collect::<Vec<String>>().last()

// reduce: 累积到单一值
.reduce_initial(init, |acc, v| acc + v)
```

### 3.4 分流 (group_by)

```rust
// group_by 返回 Observable<GroupedObservable<Key, Subject>>
source.group_by(|token: &Token| token.kind())
    .flat_map(|group: Shared<GroupedObservable<_, _>>| {
        let key = group.inner().key;
        match key {
            TokenKind::Rule => group.map(handle_rule).collect::<Vec<_>>().last(),
            TokenKind::Var  => group.tap(handle_var).last(),
            _ => group,
        }
    })
```

### 3.5 副作用 (tap = doOnXxx)

```rust
// tap: 对每个元素执行副作用（不改流）
source.tap(|v: &String| tracing::info!(sas_token = %v))
```

### 3.6 生命周期挂钩 (on_complete/on_error/finalize)

```rust
// on_complete: complete 时执行 closure（一次）
.on_complete(|| tracing::info!("pipeline complete"))

// finalize: complete/error/unsubscribe 都执行，保证一次
.finalize(|| cleanup())
```

---

## 第 4 章：Subject = Hot Observable (天生多播)

### 4.1 为什么不需要 publish().connect()

Flux 需要 `publish().connect()` 是因为 Mono/Flux 默认是 Cold (每个订阅重放)：
```java
// Flux: 必须先调 publish 转 hot
ConnectableFlux<T> publish = flux.publish();
publish.subscribe(obs1);
publish.subscribe(obs2);
publish.connect(); // ← 显式触发
```

rxrust 的 Subject 出厂就是 hot multicasting：
```rust
// rxrust: 不需要 publish/connect 转换
let s = Shared::subject::<String, Infallible>();
s.clone().subscribe(obs1);  // 立即开始接收
s.clone().subscribe(obs2);  // 立即开始接收
s.next("hello".to_string()); // 两个订阅者都收到
```

### 4.2 Re-entrancy 约束

Subject 不允许在回调内同步调用 next/complete/error：

```rust
// GC 反模式 (Rust 会 panic!)
s.clone().subscribe(|v| {
    if condition { s.next(v + 1); } // panic: re-entrant emission
});

// 正确：用 delay(0) 创建异步边界
s.clone().delay(Duration::from_millis(0))
    .subscribe(|v| {
        if v < 3 { /* 异步安全地 */ println!("{}", v); }
    });
```

**本质**：这是 Mutex 重入保护。Flux 的 publishOn 自动切线程打破 Recursion。
rxrust 用显式 delay(0)，意图更清晰。

---

## 第 5 章：所有权驱动的管线模式

### 5.1 核心管线模板 (sasspile 验证版)

```rust
pub fn compile_pipeline(input: &str) -> String {
    use rxrust::prelude::*;
    let subject = Shared::subject::<String, Infallible>();
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    subject.clone()
        // Phase 1: CompileState 消费自身 (&mut 就地修改)
        .scan_map(CompileState::new(), dispatch_pass)
        .flat_map(|v: Vec<String>| Shared::from_iter(v))
        // Phase 2: CssBuilder 消费自身 (&mut 就地修改)
        .scan_map(CssBuilder::new(), |builder: &mut CssBuilder, line: String| -> Vec<CssNode> {
            builder.feed(&line) // &借用 line
        })
        .flat_map(|v: Vec<CssNode>| Shared::from_iter(v))
        .collect::<Vec<CssNode>>().last()
        .map(|nodes| post_process(nodes))
        .flat_map(|nodes| Shared::from_iter(nodes))
        // Phase 3: 借用渲染 (&CssNode → String)
        .map(|node: CssNode| render_node(&node))
        .collect::<Vec<String>>().last()
        // subscribe = 执行边界, move tx 转移终态
        .subscribe(move |css_vec: Vec<String>| {
            let _ = tx.send(css_vec.join("\n"));
        });

    input.lines().for_each(|line| subject.clone().next(line.to_string()));
    subject.clone().complete();

    rx.recv().unwrap_or_default()
}
```

### 5.2 模式注解

```
subject (hot Subject, 'static + Send)
  ↓ scan_map(CompileState, dispatch_pass)
     每一行 → &mut state + 产出 Vec<String>   [消费自身]
  ↓ flat_map(v → Shared::from_iter(v))
     Vec<String> 逐个 String                   [1:N 展平]
  ↓ scan_map(CssBuilder, build)
     每行 → &mut builder + 产出 Vec<CssNode>  [消费自身]
  ↓ flat_map(v → Shared::from_iter(v))
     Vec<CssNode> 逐个 CssNode                [1:N 展平]
  ↓ collect::<Vec<CssNode>>().last()
     汇聚 → 最后一包                           [终点收敛]
  ↓ map(post_process) → flat_map
     后处理 + 展平
  ↓ map(render_node(&node))                  [借引用，首次 owned String]
  ↓ collect::<Vec<String>>().last()
     汇聚 → 最后一包
  ↓ subscribe(move |v| tx.send(v.join("\n")))  [move = 所有权转移终态]
```

---

## 第 6 章：IAsync 集成 (True Async)

### 6.1 Rust 的 Future vs Java 的 CompletableFuture

| 维度 | Java CompletableFuture | Rust Future |
|------|----------------------|-------------|
| 实现 | 线程池 + 线程阻塞 | 编译期状态机 + poll |
| 内存 | 线程栈 (通常 512KB~1MB) | 按实际使用的字段分配 |
| 调度 | 内核线程切换 (μs 级) | reactor_wake 唤醒 (ns 级) |
| 阻塞 | 可以 Block::park | poll_pending 自动让出 |

**本质**：Java 异步是"多线程伪装成异步"，Rust 异步是"单线程内真并发"。

### 6.2 在管线中接入异步

```rust
use rxrust::prelude::*;
use std::convert::Infallible;

// from_future: Future -> Observable
let future = async { tokio::fs::read_to_string("style.scss").await };
Shared::from_future(future)
    .flat_map(|content: String| Shared::from_iter(content.lines().map(|l| l.to_string())))
    .scan_map(CompileState::new(), reducer)
    ...
```

---

## 第 7 章：Context 子系统 (Local vs Shared)

### 7.1 选择规则

| Context | 内部结构 | 线程 | 适用 |
|---------|---------|------|------|
| `Local`  | `Rc<RefCell<Subscribers>>` | 单线程 | 测试、线程局部 |
| `Shared` | `Arc<Mutex<Subscribers>>` | 多线程 | sasspile 生产 (编译器并行) |

### 7.2 Shared 的 'static + Send 约束

上下文选择影响源类型：
- `Local::subject::<&str, E>()` — 可以用 `&str` (Rc 不需要 Send)
- `Shared::subject::<String, E>()` — 必须 `String` (Arc 需要 Send + 'static)

**sasspile 选择 Shared + String**，原因：
1. 编译器可能并行执行多个输入
2. Future 集成需要 Send
3. mpsc channel 转移终态需要 Send

---

## 第 8 章：Operator Implementation Pattern

### 99% 的 rxrust 算子遵从同一代码模式

```rust
// 1. Wrapper struct (持有 source + closure)
pub struct Map<S, F> {
    pub(crate) source: S,
    pub(crate) func: F,
}

// 2. WrapperObserver struct (持有内层 observer + closure)
pub struct MapObserver<O, F> {
    observer: O,
    func: F,
}

// 3. ObservableType for wrapper struct
impl<S, F> ObservableType for Map<S, F>
where S: ObservableType, F: FnMut(S::Item) -> U {
    type Item<'a> = U where Self: 'a;
    type Err = S::Err;
}

// 4. Observer trait for wrapper observer
impl<O, F, Item, Err> Observer<Item, Err> for MapObserver<O, F>
where O: Observer<U, Err>, F: FnMut(Item) -> U {
    fn next(&mut self, value: Item) {
        self.observer.next((self.func)(value));
    }
    fn error(self, err: Err) { self.observer.error(err); }
    fn complete(self) { self.observer.complete(); }
    fn is_closed(&self) -> bool { self.observer.is_closed() }
}

// 5. CoreObservable for wrapper struct
impl<S, F, C> CoreObservable<C> for Map<S, F>
where C: Context, S: CoreObservable<C::With<MapObserver<C::Inner, F>>> {
    type Unsub = S::Unsub;
    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped = context.transform(|observer| MapObserver { observer, func: self.func });
        self.source.subscribe(wrapped)
    }
}
```

**关键洞察**：`context.transform(...)` 是 rxrust 内部统一的 observer wrap 入口，
它自动处理 scheduler、Context 类型转换等繁琐包装。

---

## 第 9 章：AI 铁律（行为规范）

### ⛔ 铁律 1：不模仿 Flux，用 Rust 所有权思维

```
❌ 思维: "Flux 有 scan，所以 rxrust 的 scan_map 应该..."
✅ 思维: "&mut Acc 排他持有是 Rust 的 scan_map 能做的事，Flux 做不到。"
```

### ⛔ 铁律 2：不猜测 rxrust API（先读源码）

rxrust 算子签名和文档可能不一致。写任何算子调用前：
```bash
cat ~/.cargo/registry/src/index.crates.io-*/rxrust-1.0.0-rc.5/src/ops/<name>.rs
```

### ⛔ 铁律 3：flat_map 返回 Shared::from_iter，不是裸 Vec

```rust
// ❌ 编译失败: Vec<String> 不是 Observable
.flat_map(|v| v)

// ✅ 正确
.flat_map(|v: Vec<String>| Shared::from_iter(v))
```

### ⛔ 铁律 4：scan_map 是 Acc 唯一合法栖息地

```rust
// ❌ 反模式: 外部可变 + 共享
let mut state = CompileState::new();
source.map(|x| { state.update(x); x });

// ✅ 正确: state 在 scan_map 内, 每订阅隔离
source.scan_map(CompileState::new(), reducer)
```

### ⛔ 铁律 5：subscribe 内不嵌套触发 source (re-entrant panic)

```rust
// ❌ panic
s.clone().subscribe(|v| s.next(v + 1));

// ✅ 正确: delay(0) 异步边界 或 flat_map 连接子流
s.clone().flat_map(move |v| Shared::from_iter(vec![v + 1, v + 2]))
```

### ⛔ 铁律 6：is_closed 守卫 from_iter 循环

```rust
// from_iter 的 Observable 内部模式:
for item in iter {
    if observer.is_closed() { break; } // 必须
    observer.next(item);
}
```

### ⛔ 铁律 7：Shared 入口用 String，不用 &str

```rust
// ❌ 编译失败: &str 不满足 'static + Send
let s = Shared::subject::<&str, E>();

// ✅ 正确
let s = Shared::subject::<String, E>();
```

### ⛔ 铁律 8：管线终点收敛用 collect::<T>().last()

```rust
.collect::<Vec<String>>()  // 汇聚结果
.last()                    // 只发射终态
.subscribe(|v| tx.send(v.join("\n")));
```

---

## 第 10 章：源码导航速查

| 场景 | 读哪里 |
|------|--------|
| Observer trait 签名 + move 语义 | `src/observer.rs` |
| Subject re-entrant 逻辑 | `src/subject/subject_core.rs` |
| Subject broadcast 实现 (`broadcast_value`) | `src/subject/subscribers.rs` |
| Connectable (publish/connect) | `src/observable/connectable.rs` |
| scan_map (Acc 隔离实现) | `src/ops/scan_map.rs` |
| flat_map (MergeAll + concurrent) | `src/ops/flat_map.rs` + `src/ops/merge_all.rs` |
| group_by (分流 + GroupedObservable) | `src/ops/group_by.rs` |
| collect (complete emit) | `src/ops/collect.rs` |
| last (final value only) | `src/ops/last.rs` |
| from_iter + is_closed 守卫 | `src/observable/from_iter.rs` |
| delay(0) 异步边界 | `src/ops/delay.rs` |
| finalize (unsubs+complete+error) | `src/ops/finalize.rs` |
| tap (doOnNext 等价) | `src/ops/tap.rs` |
| distinct (去重) | `src/ops/distinct.rs` |
| reduce (累积终值) | `src/ops/reduce.rs` |
| Context (Local vs Shared) | `src/context.rs` |
| MutRc vs MutArc pointer | `src/rc.rs` |
| Scheduler (tokio 集成) | `src/scheduler.rs` |

---

## 附录 A：AI 行为契约（强制）

作为 sasspile 开发者，AI 保证：

1. ✅ 用 Rust ownership 三态 (move/&/&) 思考数据流，不强行翻译 Flux
2. ✅ rxrust 算子 API **先读源码**不猜测
3. ✅ 状态放 scan_map Acc，不捕获外部可变
4. ✅ flat_map Inner 用 Shared::from_iter，不返回裸 Iterator
5. ✅ subscribe 是执行唯一起点，不在 subscribe 内嵌套触发 source
6. ✅ 无 println/eprintln，一律 tracing 宏
7. ✅ src/ 无 unwrap/expect，可用 `?` 或 Option/Result
8. ✅ 单文件 ≤ 500 行
9. ✅ 用 SSH push github main，但 commit 后等用户确认

违反以上任何一条 = 任务失败。
