# rxrust 1.0.0-rc.5 架构完整指南

> 基于源码阅读的完整参考,覆盖核心 traits、算子模式、上下文系统和调度器。

---

## 1. 核心 Trait 体系

### 1.1 Observer<Item, Err>
```rust
pub trait Observer<Item, Err> {
    fn next(&mut self, value: Item);
    fn error(self, err: Err);      // 消费 self
    fn complete(self);              // 消费 self
    fn is_closed(&self) -> bool;
}
```
- `next` 用 `&mut self`,可多次调用
- `error`/`complete` 消费 self,表示流终止
- `is_closed` 用于上游判断是否停止发射(如 `from_iter` 中配合 `take` 提前终止)

### 1.2 ObservableType
```rust
pub trait ObservableType {
    type Item<'a> where Self: 'a;
    type Err;
}
```
纯粹的 associated type 暴露 trait,让用户和编译器看到流的 Item/Err 类型。

### 1.3 CoreObservable<O>
```rust
pub trait CoreObservable<O>: ObservableType {
    type Unsub: Subscription;
    fn subscribe(self, observer: O) -> Self::Unsub;
}
```
流的纯逻辑内核,泛型 observer context `O`,同样的逻辑可在 Local/Shared 两种上下文中工作。

### 1.4 Observable (用户-facing)
```rust
pub trait Observable: Context {
    type Item<'a> where Self: 'a;
    type Err;
    fn subscribe<F, U>(self, f: F) -> U
    where F: for<'a> FnMut(Self::Item<'a>) { ... }
    fn map<F, Out>(self, f: F) -> Self::With<Map<Self::Inner, F>> { ... }
    // ... 其他算子
}
```
通过 `Context::transform` / `Context::lift` 提供链式调用。

---

## 2. Context 系统

rxrust 通过 `Context` trait 区分执行环境。

### 2.1 LocalCtx<T, S> vs SharedCtx<T, S>
```rust
pub struct LocalCtx<T, S> { pub inner: T, pub scheduler: S }
pub struct SharedCtx<T, S> { pub inner: T, pub scheduler: S }
```

**默认类型别名:**
```rust
pub type Local<T> = LocalCtx<T, LocalScheduler>;   // 单线程
pub type Shared<T> = SharedCtx<T, SharedScheduler>; // 多线程,tokio
```

**关键区别:**
- `Local`: 使用 `Rc<RefCell<T>>`,无需 `Send`
- `Shared`: 使用 `Arc<Mutex<T>>`,要求 `Send + 'static`
- `SharedCtx` 的 `Observer`/`Subscription` 实现额外要求 `Send`

### 2.2 Context Trait 核心方法
```rust
fn lift<U>(inner: U) -> Self::With<U>;                    // 用默认调度器创建
fn transform<U, F>(self, f: F) -> Self::With<U> where F: FnOnce(Self::Inner) -> U;
fn swap<U>(self, inner: U) -> (Self::Inner, Self::With<U>);
fn into_inner(self) -> Self::Inner;
```

**`transform` 的用途:** 在算子内部包裹 observer 时传递调度器。

---

## 3. 算子标准四部曲

这是 rxrust 内置算子的标准写法,自定义算子必须严格遵循。

### 3.1 以 `Map` 为例

```rust
// 1️⃣ 算子 struct (Clone, 持有源和函数)
#[derive(Clone)]
pub struct Map<S, F> {
    pub source: S,
    pub func: F,
}

// 2️⃣ Observer 包裹器
pub struct MapObserver<O, F> {
    observer: O,
    func: F,
}

// 3️⃣ 为包裹器实现 Observer
impl<O, F, Item, Out, Err> Observer<Item, Err> for MapObserver<O, F>
where
    O: Observer<Out, Err>,
    F: FnMut(Item) -> Out,
{
    fn next(&mut self, v: Item) { self.observer.next((self.func)(v)); }
    fn error(self, e: Err) { self.observer.error(e); }
    fn complete(self) { self.observer.complete(); }
    fn is_closed(&self) -> bool { self.observer.is_closed() }
}

// 4️⃣ 实现 ObservableType
impl<S, F, Out> ObservableType for Map<S, F>
where S: ObservableType, F: for<'a> FnMut(S::Item<'a>) -> Out {
    type Item<'a> = Out where Self: 'a;
    type Err = S::Err;
}

// 5️⃣ 实现 CoreObservable
impl<S, F, C, Out> CoreObservable<C> for Map<S, F>
where
    C: Context,
    S: CoreObservable<C::With<MapObserver<C::Inner, F>>>,
    F: for<'a> FnMut(S::Item<'a>) -> Out,
{
    type Unsub = S::Unsub;
    fn subscribe(self, context: C) -> Self::Unsub {
        let Map { source, func } = self;
        let wrapped = context.transform(|observer| MapObserver { observer, func });
        source.subscribe(wrapped)
    }
}
```

### 3.2 以 `ScanMap` 为例 (带累加器)

```rust
#[derive(Clone)]
pub struct ScanMap<S, F, Acc> {
    pub source: S,
    pub func: F,
    pub initial_value: Acc,
}

pub struct ScanMapObserver<O, F, Acc> {
    observer: O,
    func: F,
    acc: Acc,  // 累加器在 observer 内部
}

impl<S, F, C, Acc, Output> CoreObservable<C> for ScanMap<S, F, Acc> { ... }
```

### 3.3 以 `Collect` 为例 (终结操作)

```rust
#[derive(Clone)]
pub struct Collect<S, C> {
    pub source: S,
    pub collection: C,
}

pub struct CollectObserver<O, C> {
    observer: O,
    collection: C,
}

impl<O, C, Item, Err> Observer<Item, Err> for CollectObserver<O, C>
where O: Observer<C, Err>, C: Extend<Item> {
    fn next(&mut self, value: Item) { self.collection.extend(Some(value)); }
    fn complete(mut self) {
        self.observer.next(self.collection);  // 完成时发射集合
        self.observer.complete();
    }
}
```

---

## 4. 源 (Observable Sources)

### 4.1 FromIter
```rust
pub fn from_iter<Iter>(iter: Iter) -> FromIter<Iter>
where Iter: IntoIterator

impl<C, Iter> CoreObservable<C> for FromIter<Iter> {
    type Unsub = ();  // 同步,无取消
    fn subscribe(self, context: C) -> Self::Unsub {
        let mut observer = context.into_inner();
        for item in self.iter {
            if observer.is_closed() { break; }  // 支持提前终止
            observer.next(item);
        }
        if !observer.is_closed() { observer.complete(); }
    }
}
```

### 4.2 FromStream
```rust
#[derive(Clone)]
pub struct FromStream<St, S> {
    pub stream: St,
    pub scheduler: S,
}

// FromStreamTask 是一个 Future,通过 scheduler 调度
impl<St, O> Future for FromStreamTask<St, O> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        // loop { poll_next -> Ready(Some) 发射; Ready(None) 完成; Pending 返回 }
    }
}
```

**关键:** `FromStream` 内部已经写好了 loop,用户不需要手写。

### 4.3 工厂方法 (ObservableFactory)
```rust
pub trait ObservableFactory: Context<Inner = ()> {
    fn of<V>(v: V) -> Self::With<Of<V>>;
    fn empty() -> Self::With<Empty>;
    fn never() -> Self::With<Never>;
    fn throw_err<E>(err: E) -> Self::With<ThrowErr<E>>;
    fn from_iter<Iter>(iter: Iter) -> Self::With<FromIter<Iter>>;
    fn from_stream<St>(stream: St) -> Self::With<FromStream<St, Self::Scheduler>>;
    fn from_future<F>(future: F) -> Self::With<FromFuture<F, Self::Scheduler>>;
    fn interval(period: Duration) -> Self::With<Interval<Self::Scheduler>>;
    fn timer(due: Duration) -> Self::With<Timer<Self::Scheduler>>;
    // ...
}
```

为所有 `Context` 提供 blanket impl。

---

## 5. 调度器系统

### 5.1 LocalScheduler vs SharedScheduler

| 特性 | LocalScheduler | SharedScheduler |
|------|---------------|-----------------|
| 线程 | 单线程 | 多线程 (tokio) |
| `Send` 要求 | 无 | 要求 `Send + 'static` |
| 内部运行时 | 无 / thread-local | 全局 tokio Runtime |
| `schedule` 返回值 | `TaskHandle` | `TaskHandle` |
| Unsub 类型 | `()` (同步) | `TaskHandle` (异步) |

### 5.2 TaskHandle
```rust
impl TaskHandle {
    fn finished() -> Self;           // 已完成
    fn is_finished(&self) -> bool;   // 是否完成
}
impl Future for TaskHandle {
    type Output = ();
    fn poll(...) -> Poll<()>;        // 等待完成
}
impl Subscription for TaskHandle {
    fn unsubscribe(self);            // 取消
    fn is_closed(&self) -> bool;
}
```

**用法:**
```rust
let handle = Shared::from_stream(stream)
    .map(...)
    .collect::<String>()
    .last()
    .subscribe(|s| { result = s; });

// 等待完成
handle.await;

// 或者取消
handle.unsubscribe();
```

### 5.3 自定义调度器
只需要实现 `Scheduler` trait + `SleepProvider`:
```rust
impl SleepProvider for MyScheduler {
    type SleepFuture = std::future::Ready<()>;
    fn sleep(&self, _duration: Duration) -> Self::SleepFuture {
        std::future::ready(())
    }
}
impl<S> Scheduler<S> for MyScheduler
where S: 'static + Schedulable<MyScheduler> {
    fn schedule(&self, source: S, _delay: Option<Duration>) -> TaskHandle { ... }
}
```
然后定义类型别名:
```rust
type MyLocal<T> = LocalCtx<T, MyScheduler>;
```

---

## 6. Subscription 体系

```rust
pub trait Subscription {
    fn unsubscribe(self);     // 消费 self
    fn is_closed(&self) -> bool;
}
```

**组合子:**
- `ClosureSubscription<F>` - 包装闭包
- `SourceWithHandle<U, H>` - 组合源 + 句柄 (用于 debounce/delay)
- `EitherSubscription<A, B>` - 二选一
- `TupleSubscription<A, B>` - 元组
- `DynamicSubscriptions` - 动态添加/移除

---

## 7. 完整管道示例

### 7.1 Local (单线程)
```rust
use rxrust::prelude::*;
use std::convert::Infallible;

let result = Local::from_iter(0..10)
    .filter(|v| v % 2 == 0)
    .map(|v| v * 2)
    .last()
    .subscribe(|v| println!("{}", v));
```

### 7.2 Shared (多线程)
```rust
use rxrust::prelude::*;
use futures::stream;

let handle = Shared::from_stream(stream::iter(vec![1, 2, 3]))
    .map(|v| v * 2)
    .collect::<Vec<_>>()
    .last()
    .subscribe(move |v| {
        *result.lock().unwrap() = v;
    });

// 等待异步完成
handle.await;
```

### 7.3 从 futures::stream 创建
```rust
let handle = Shared::from_stream(futures::stream::iter(chars))
    .scan_map(initial_state, |acc, c| { ... })
    .flat_map(|tokens| Shared::from_stream(futures::stream::iter(tokens)))
    .use_().mixin().include()
    .collect::<String>()
    .last()
    .subscribe(move |s| { *result.lock().unwrap() = s; });

handle.await;
```

---

## 8. 关键设计规则

### ✅ 必须做
1. **算子 struct 必须 `#[derive(Clone)]`** - 这是链式调用的基础
2. **Observer 包裹器** - 每个算子都需要一个 `XxxObserver<O, ...>` 结构
3. **使用 `context.transform`** - 在 `subscribe` 中包裹 observer,自动传递调度器
4. **Observer 方法签名** - `next(&mut self)` / `error(self)` / `complete(self)` / `is_closed(&self)`
5. **为 Shared 算子实现 `Send`** - 所有字段必须满足 `Send`

### ❌ 严禁做
1. **不要手写 mpsc / channel** - rxrust 的 `from_stream` 已经处理好了
2. **不要手写 tokio Runtime** - `SharedScheduler` 内部已有全局 runtime
3. **不要用 `PhantomData<fn()>`** - 这会让 struct 不满足 `Send`,改用具体类型参数
4. **不要在 `subscribe` 外暴露 `self::Inner`** - 通过 `into_inner()` 消费
5. **不要用 `Rc<RefCell>` 在 Shared 上下文中** - 用 `Arc<Mutex>` 或直接传值

### ⚠️ 注意事项
1. **`flat_map` 用 `PhantomData<fn() -> Inner>`** - 它是特例,因为只需要类型标注;自定义算子不要模仿这个
2. **`for<'a>` 生命周期** - 涉及 `Item<'a>` 的闭包需要 Higher-Ranked Trait Bounds
3. **`FromStream` 已经内部 loop** - 不需要在外部手写 polling 循环
4. **`last()` 消费整个流** - 搭配 `collect()` 获取所有值

---

## 9. 内部 Observer 类型

```rust
// 闭包适配器
pub struct FnMutObserver<F>(pub F);
impl<F, Item> Observer<Item, Infallible> for FnMutObserver<F>
where F: FnMut(Item) { ... }

// Box dyn
pub type BoxedObserver<'a, Item, Err> = Box<dyn DynObserver<Item, Err> + 'a>;
pub type BoxedObserverSend<'a, Item, Err> = Box<dyn DynObserver<Item, Err> + Send + 'a>;

// MutRef (用于 in-place 修改)
pub type BoxedObserverMutRef<'a, Item, Err> = Box<dyn for<'m> DynObserver<&'m mut Item, Err> + 'a>;
```

---

## 10. 常用算子速查

| 算子 | 功能 | 示例 |
|------|------|------|
| `map` | 转换 | `.map(\|x\| x * 2)` |
| `filter` | 过滤 | `.filter(\|x\| x > 5)` |
| `filter_map` | 过滤+转换 | `.filter_map(\|x\| x.ok())` |
| `flat_map` | 展开子流 | `.flat_map(\|x\| Shared::from_iter(x))` |
| `scan_map` | 累加+转换 | `.scan_map(0, \|acc, v\| *acc += v)` |
| `collect` | 收集为集合 | `.collect::<Vec<_>>()` |
| `last` | 取最后一个 | `.last()` |
| `take` | 取前 N 个 | `.take(5)` |
| `skip` | 跳过前 N 个 | `.skip(3)` |
| `tap` | 副作用(日志) | `.tap(\|x\| debug!(?x))` |
| `take_while` | 条件取 | `.take_while(\|x\| x < 10)` |
| `skip_while` | 条件跳过 | `.skip_while(\|x\| x < 5)` |
| `start_with` | 开头插入 | `.start_with(0)` |
| `merge` | 合并流 | `.merge(other_stream)` |

---

## 11. 错误处理

```rust
// 错误Observer
fn error(self, err: Err);  // 消费 self

// 链中处理错误
observable
    .on_error(|e| error!(?e))  // 副作用
    .subscribe(|v| ...);
```

---

## 12. 命名约定

- 算子 struct: `XxxOp` 或 `Xxx` (如 `Map`, `Filter`)
- Observer 包裹器: `XxxObserver<O, ...>`
- 工厂函数: `from_xxx`
- 类型别名: `BoxedXxx`

---

## 附录: 文件结构

```
src/
├── lib.rs                    # 根模块
├── context.rs                # Context trait, LocalCtx, SharedCtx
├── observer.rs               # Observer trait, FnMutObserver
├── observable.rs             # ObservableType, CoreObservable, Observable
├── observable/
│   ├── from_iter.rs          # 同步迭代器源
│   ├── from_stream.rs        # 异步 Stream 源
│   ├── from_future.rs        # Future 源
│   ├── from_fn.rs            # 闭包源
│   ├── create.rs             # 自定义发射器
│   ├── of.rs                 # 单值源
│   ├── trivial.rs            # empty/never/throw_err
│   └── interval.rs           # 定时器
├── ops/
│   ├── map.rs                # 转换
│   ├── filter.rs             # 过滤
│   ├── scan_map.rs           # 累加转换
│   ├── flat_map.rs           # 展开
│   ├── collect.rs            # 收集
│   ├── last.rs               # 最后值
│   ├── take.rs / skip.rs     # 截取
│   └── ...
├── scheduler.rs              # Task, TaskHandle, Scheduler
├── subscription.rs           # Subscription trait
├── factory.rs                # ObservableFactory
└── subject.rs                # Subject (热observable)
```

---

*文档版本: rxrust 1.0.0-rc.5*
*生成时间: 2026-09-18*
