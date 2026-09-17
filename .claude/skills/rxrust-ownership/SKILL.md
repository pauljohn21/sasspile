---
name: rxrust-ownership
description: Rust 所有权 + rxrust 算子驱动 SCSS 编译器开发.当写 Rust 代码、实现编译器功能、修复 spec 失败时激活.核心原则:rxrust IS Rust 所有权 — scan_map 消费旧状态产出新状态, flat_map 转移所有权展开子流, collect/last 消费整个流.禁止 Rc<RefCell>/for 循环/subscribe+push 等 GC 模式。
allowed-tools: Read, Write, Edit, MultiEdit, ListDir
license: MIT
metadata:
  author: sasspile-rx
  version: "4.1"
---

# v4.1 全量内化版：基于 rxrust 1.0.0-rc.5 完整源码阅读后编写

本 SKILL 来自对 rxrust 1.0.0-rc.5 **全量核心源码**的精确阅读。写代码时**只查本 SKILL**，绝不回头查源码。

## ⚠️ 核心铁律:chain 是一条完整管道,不要拆分中间变量

`box_it()` 把 `Local<ConcreteCombinator>` 擦成 `Local<Box<dyn DynCoreObservable>>`,已经是具体 `LocalCtx<Box<dyn...>>`。再用 `Local::new(boxed)` 包起来会形成 `LocalCtx<LocalCtx<Box<dyn...>>>`,内层 nested Context 不满足 `Observable` blanket impl。

**正确做法**:只在最终出口 `.box_it()` 一次。中间步骤不需要擦除类型,也不需要命名绑定(`let stage = ...`)。类型多大都留在 chain 内让编译器推断即可。

---

## 0. 类型别名精确表

```
// 来自 context.rs
pub type Local<T> = LocalCtx<T, LocalScheduler>;
pub type Shared<T> = SharedCtx<T, SharedScheduler>;

// 来自 observable/boxed.rs
pub type LocalBoxedObservable<'a, Item, Err = ()> =
    Local<BoxedCoreObservable<'a, Item, Err, LocalScheduler>>;

pub type LocalBoxedObservableClone<'a, Item, Err = ()> =
    Local<BoxedCoreObservableClone<'a, Item, Err, LocalScheduler>>;
```

---

## 1. Observable 关系图

```
Observable (用户API trait)
  ├── 继承: Context
  ├── Item<'a>, Err (关联类型)
  ├── map/filter/scan_map/flat_map/collect/last/box_it ... (全部返回 Self::With<Op<Inner>>)
  └── subscribe<F>(f: F) -> U    ← 唯一终点,不是 subscribe_with(closure)

Context (执行环境 trait)
  ├── Inner: 内部类型
  ├── Scope: LocalScope / SharedScope
  ├── With<T>: 同家族新 Context
  ├── transform(f) -> With<U>: Functor map
  └── lift(v) -> With<U>: 构造新 Context

CoreObservable<O>
  └── subscribe(self, ctx: O) -> Unsub   ← 内部 trait,用户不直接调

ObservableFactory: Context<Inner = ()>
  └── of(v) / from_iter(iter) / empty() / nothrow() ...
       ← blanket impl for any Context<Inner = ()>: Local / LocalCtx<(), S>
```

**关键事实（源码 2487-2498 行）**:
```rust
impl<T> Observable for T
where
    T: Context,
    T::Inner: ObservableType,
```
任何实现了 `Context` 且 `Inner: ObservableType` 的类型都是 `Observable`，**自动获得所有方法**。

---

## 2. 创建 Observable

```rust
use rxrust::prelude::*;

// 单次发射 (Err = Infallible)
let obs: Local<Of<i32>> = Local::of(42);

// 迭代器发射 (Err = Infallible)
let obs: Local<FromIter<Chars<'_>>> = Local::from_iter(input.chars());
let obs: Local<FromIter<std::vec::IntoIter<Token>>> = Local::from_iter(tokens_vec);
let obs: Local<FromIter<Range<i32>>> = Local::from_iter(0..10);

// 空完成 / 永不完成
let obs: Local<Empty> = Local::empty();
let obs: Local<Never> = Local::never();
```

### 工厂方法签名（来自 factory.rs）

```rust
fn of<V>(v: V) -> Self::With<Of<V>>                       // 单次发射
fn from_iter<I: IntoIterator>(iter: I) -> Self::With<FromIter<I>>
fn empty() -> Self::With<Empty>
fn never() -> Self::With<Never>
fn throw_err<E>(error: E) -> Self::With<ThrowErr<E>>
```

> **注意**: `Local::of(v)` / `Local::from_iter(iter)` 来自 `ObservableFactory` trait 的 blanket impl。
> `Local::new(inner: T)` 来自 `Context::new` —— 它包装任意 inner 类型进 LocalCtx（用于 **re-wrap boxed**）。

---

## 3. 核心算子（全部返回 Self::With<Op>，保持链式）

### 3.1 map

```rust
// observable.rs 167 行 (stable)
fn map<F, Out>(self, f: F) -> Self::With<Map<Self::Inner, F>>
where
    F: for<'a> FnMut(Self::Item<'a>) -> Out,

// 使用
Local::from_iter([1, 2, 3])
    .map(|v| v * 2)           // Item<'a> = i32, Out = i32
    .subscribe(|v| assert_eq!(*v % 2, 0));
```

### 3.2 filter / filter_map

```rust
fn filter<F>(self, filter: F) -> Self::With<Filter<Self::Inner, F>>
where
    F: for<'a> FnMut(&Self::Item<'a>) -> bool,

fn filter_map<F, Out>(self, f: F) -> Self::With<FilterMap<Self::Inner, F>>
where
    F: for<'a> FnMut(Self::Item<'a>) -> Option<Out>,
```

### 3.3 scan_map ← 核心状态转移算子

```rust
// scan_map.rs 定义
pub struct ScanMap<S, F, Acc> { source: S, func: F, initial_value: Acc }

// Observable trait 方法签名（observable.rs 402 行）
fn scan_map<Acc, Output, F>(self, initial: Acc, f: F) -> Self::With<ScanMap<Self::Inner, F, Acc>>
where
    F: for<'a> FnMut(&mut Acc, Self::Item<'a>) -> Output,
```

**reducer 签名**: `FnMut(&mut Acc, Item) -> Output`
- `&mut Acc` = 就地修改累加器（不 clone 分布）
- `Item` = 当前 item（by value — RxRust 把它 move 给 reducer）
- `Output` = 本次 emit 的值
- reducer 调 N 次（N = upstream item 数）→ 产出 N 个 Output

**error 类型**: `Self::Err = S::Err`（继承 upstream error）

```rust
// 实战示例: 字符扫描器
Local::from_iter(input.chars())
    .scan_map(Scanner::new(), |scanner, ch| {
        let emitted = scanner.feed(ch);   // &mut Scanner 就地修改
        emitted                           // Vec<Token> 是 Output
    })
    .flat_map(|toks| Local::from_iter(toks))   // 展开 Vec<Token> 为子流
    .last()
    .subscribe(|tokens| { /* 使用 final tokens list */ });
```

### 3.4 flat_map ← 1→N 展开

```rust
// observable.rs 2400 行
fn flat_map<F, Inner>(self, f: F) -> Self::With<FlatMap<Self::Inner, F, Inner>>
where
    F: for<'a> FnMut(Self::Item<'a>) -> Inner,
    Inner: Context<Inner: ObservableType<Err = Self::Err>>,
    //                                                      ^^^^^^^^^ 
    //                                                      Inner 的 Err 必须 = 外层 Err
```

**闭包签名**: `FnMut(Item) -> Inner`
- `Item` = 外层 item（by value）
- `Inner` 必须是一个 `Context`：它本身也有 `Inner: ObservableType`
- Inner 的 ObservableType::Err = Self::Err（类型约束）

```rust
// 实战: scan_map 产出 Vec<Token>，flat_map 展开
.scan_map(Scanner::new(), |s, ch| s.feed(ch))    // scan_map 产出 Vec<Token>
.flat_map(|toks: Vec<Token>| Local::from_iter(toks))   // Vec → FromIter → 子流合并
                                                       // Local::from_iter(toks) 是 Local<FromIter<Vec<Token>>>
                                                       // 它的 Item = Token, Err = Infallible

// 其他例子
Local::from_iter([1, 2, 3])
    .flat_map(|x| Local::from_iter([x, x + 10]))
    // emits: 1, 11, 2, 12, 3, 13
```

### 3.5 collect ← 终端聚合

```rust
// collect.rs
pub struct Collect<S, C> {
    pub source: S,
    pub collection: C,    // C: Extend<Item>
}

// Observable trait 方法（observable.rs 1515 行）
fn collect<C>(self) -> Self::With<Collect<Self::Inner, C>>
where
    C: Default,

fn collect_into<C>(self, initial: C) -> Self::With<Collect<Self::Inner, C>>
```

- `C: Extend<Item>` + `C: Default`（collect 调用 `C::default()` 再 feed）
- **item 类型不变**: Collect 的 `Item<'a>` = `C`（不是 `Item<'a>`！）
- 即 collect 把 N 个 item 聚成 1 个 C，然后 emit C

```rust
Local::from_iter([1, 2, 3])
    .collect::<Vec<i32>>()    // Item = Vec<i32>, emits once: [1, 2, 3]
    .subscribe(|v: Vec<i32>| assert_eq!(v, vec![1, 2, 3]));

Local::from_iter(input.chars())
    .collect::<String>()       // Extend<char> for String; Item = String
    .subscribe(|s: String| println!("{s}"));
```

### 3.6 last ← 终端取末值

```rust
// last.rs
pub struct Last<S> { pub source: S }

// Observable trait 方法（observable.rs 573 行）
fn last(self) -> Self::With<Last<Self::Inner>>
// Item<'a> = S::Item<'a> — 保持 item 类型不变
```

- `last` 完全消费 upstream，emit **最后一个** item，然后 complete
- 若 upstream 空，complete 不 emit

```rust
Local::from_iter([1, 2, 3])
    .last()    // emits 3, completes
    .subscribe(|v| assert_eq!(*v, 3));
```

### 3.7 last_or（带默认值）

```rust
fn last_or<'a>(self, default_value: Self::Item<'a>)
    -> Self::With<DefaultIfEmpty<Last<Self::Inner>, Self::Item<'a>>>
```

- upstream 空时 emit `default_value`
- 否则与 last 同

### 3.8 tap ← 只读副作用

```rust
// tap.rs
fn tap<F>(self, f: F) -> Self::With<Tap<Self::Inner, F>>
where
    F: for<'a> FnMut(&Self::Item<'a>),
```

- 闭包拿 `&Self::Item`（只读），不修改 item
- Item 类型不变

### 3.9 take

```rust
fn take(self, count: usize) -> Self::With<Take<Self::Inner>>
```

---

## 4. type erasure: box_it ↔ Local::new

### 4.1 box_it（类型擦除为统一类型）

```rust
// observable.rs 1944 行
fn box_it<'a, 'b>(self) -> Self::With<Self::BoxedCoreObservable<'a, Self::Item<'b>, Self::Err>>
where
    Self::Inner: IntoBoxedCoreObservable<Self::BoxedCoreObservable<'a, Self::Item<'b>, Self::Err>>,
```

**返回**: `Local<BoxedCoreObservable<'a, Item, Err, LocalScheduler>>`
- 即 `LocalBoxedObservable<'a, Item, Err>`（boxed.rs 427 行类型别名）
- 类型擦除后 **只保留 Item & Err**，其他全消失

### 4.2 Local::new（⚠️ 禁止用于包装 boxed observable）

`Local::new(inner)` 只在构造裸 Observable 时使用(如 `Local::new(FromIter::new(v))`)。

**禁止**: `Local::new(boxed_observable)` — 这会把 `Local<Box<dyn...>>` 再包成 `Local<Local<Box<dyn...>>>`,形成 nested LocalCtx 嵌套。

nested LocalCtx 违背 `Observable` blanket impl:
```rust
impl<T> Observable for T
where T: Context, T::Inner: ObservableType
// 外层 Local<Local<Box>> 要求 Inner = Local<Box> 也实现 Observable
// 但 scan_map 末 emit Item 而非 Observable,类型关系断裂 → E0599
```

### 4.3 实战流程模式 — ✅ 单一 chain + 末尾 box_it

```rust
// sasspile-rx pipeline.rs:一条完整 chain,中间不 box_it,中间不命名绑定
pub fn build(input: &str) -> LocalBoxedObservable<'static, char, Infallible> {
    let chars: Vec<char> = input.chars().collect();

    Local::from_iter(chars)
        // Stage 1: char stream → token stream
        .scan_map(Scanner::new(), |state, ch| state.feed(ch))
        .flat_map(|toks| Local::from_iter(toks))
        .tap(|tok| tracing::trace!(?tok, stage = "tokenize"))
        // Stage 2: token stream → node stream
        .scan_map(AstBuilder::new(), |builder, tok| builder.feed(tok))
        .flat_map(|node_vec| Local::from_iter(node_vec))
        .tap(|node| tracing::trace!(?node, stage = "parse"))
        // Stage 3: node stream → CssNode stream
        .flat_map(|node| Local::from_iter(eval_node_vec(node)))
        .tap(|css| tracing::trace!(?css, stage = "evaluate"))
        // Stage 4: CssNode stream → char stream
        .flat_map(|css_node| Local::from_iter(render_node_to_chars(css_node)))
        .tap(|ch| tracing::trace!(char = %ch, stage = "serialize"))
        .box_it()   // ← 最终出口擦除类型,得到 LocalBoxedObservable<char>
}
```

> **类型多大都无所谓**: `Local<FlatMap<ScanMap<FromIter<char>, ...>, ...>>` 会是很长的 combinator 类型,
> 但只在出口 `.box_it()` 一次 — 擦成 `LocalBoxedObservable<char, Infallible>`。
> 中间链不需要手动标注类型,编译器会自动推断。

---

## 5. subscribe 终点（唯一正确终点 API）

```rust
// observable.rs 141 行 — 唯一 subscribe 方法
fn subscribe<F, U>(self, f: F) -> U
where
    F: for<'a> FnMut(Self::Item<'a>),
    Self::Inner: CoreObservable<Self::With<FnMutObserver<F>>, Unsub = U>,
```

- 闭包 `F: FnMut(Self::Item<'a>)` — 拿 by value
- **Err 必须是 Infallible** 时才能用这个 subscribe（大多数情况如此）— 因为 FnMutObserver<Item, Infallible>
- 若 Err != Infallible，用 `subscribe_with(observer)` 或 `.on_error(f).subscribe(g)`

```rust
// 最常见: Err = Infallible
obs.subscribe(|item| { /* 使用 item */ });

// 需要 error handling
obs.on_error(|e| error!(?e, "pipeline error"))
   .subscribe(|item| { ... });
```

> **没有 subscribe_boxed！** 旧版 SKILL 里写的 `subscribe_boxed` **不存在**。
> 正确做法: boxed 要么 `.subscribe(f)` 直接消费（boxed 自身就是 Observable），
> 要么 `Local::new(boxed)` re-wrap 后再 chain 更多。

---

## 6. 绝对禁止（GC 思维）

| 反模式 | 为什么违反 Rust 所有权 | 正确做法 |
|---|---|---|
| `Rc<RefCell<T>>` + `borrow_mut()` | 模拟 GC 共享可变 | `scan_map(State::new(), reducer)` |
| `let mut v = Vec::new(); for x in items { v.push(f(x)) }` | 命令式累积 | `items.flat_map(f).collect()` 或 `items.scan_map(accum, r)` |
| `while i < len { out.push(s[i]); i += 1 }` | 索引循环 | iterator + scan/fold |
| `obs.subscribe(\|x\| cell.borrow_mut().push(x))` | 手动偷取流值 | `obs.last().subscribe(\|r\| ...)` 或 `obs.collect().subscribe(...)` |
| helper 函数内构造 Observable | helper 绕开 pipeline | helper 内用原语迭代器 (`map/filter/fold/collect`) |
| `obs.subscribe_boxed(f)` ← **这个方法不存在！** | 幻觉 API | `obs.subscribe(f)` + `Local::new(boxed).subscribe(f)` |

---

## 7. 完整 Pipeline Pattern（sasspile-rx 实战）

### ❌ 反模式 — 中间 box_it + Local::new 导致 nested LocalCtx

```rust
// ❌ 别这么做:中间 box_it 后 Local::new 包出 LocalCtx<LocalCtx<...>>
let tokens: LocalBoxedObservable<'_, Token, Infallible> =
    Local::from_iter(input.chars())
        .scan_map(Scanner::new(), |s, c| s.feed(c))
        .flat_map(|toks| Local::from_iter(toks))
        .box_it();    // ❌ 中间类型擦除

let nodes: LocalBoxedObservable<'_, CssNode, Infallible> =
    Local::new(tokens)   // ❌ nested LocalCtx,E0599 报错
        .scan_map(Parser::new(), |p, t| p.feed(t))
        .flat_map(|ns| Local::from_iter(ns))
        .box_it();
```

### ✅ 正确模式 — 一条完整 chain + 末尾 box_it

```rust
pub fn build(input: &str) -> LocalBoxedObservable<'static, char, Infallible> {
    let chars: Vec<char> = input.chars().collect();

    Local::from_iter(chars)
        .scan_map(Scanner::new(), |state, ch| state.feed(ch))
        .flat_map(|toks| Local::from_iter(toks))
        .tap(|tok| tracing::trace!(?tok, stage = "tokenize"))
        .scan_map(AstBuilder::new(), |builder, tok| builder.feed(tok))
        .flat_map(|node_vec| Local::from_iter(node_vec))
        .tap(|node| tracing::trace!(?node, stage = "parse"))
        .flat_map(|node| Local::from_iter(eval_node_vec(node)))
        .tap(|css| tracing::trace!(?css, stage = "evaluate"))
        .flat_map(|css_node| Local::from_iter(render_node_to_chars(css_node)))
        .tap(|ch| tracing::trace!(char = %ch, stage = "serialize"))
        .box_it()
}
```

**入口消费**:
```rust
let (tx, rx) = mpsc::channel();
pipeline::build(input)
    .collect::<String>()
    .last()
    .subscribe(move |css| { let _ = tx.send(css); });
let css = rx.recv().unwrap_or_default();
```

---

## 8. 常见错误对照表

| 错误 | 原因 | 正确 |
|---|---|---|
| `impl LocalObservable<LocalScheduler> for StageBuilder` | **没有 LocalObservable trait** | impl 自定义 struct，内部封装 rxrust ops；不需要 impl 任何 Observable trait |
| `obs.subscribe_boxed(f)` | **没有 subscribe_boxed 方法** | `obs.subscribe(f)` 或 `Local::new(obs).subscribe(f)` |
| `let mid = chain.box_it(); Local::new(mid).scan_map(...)` | 中间 box_it 后再 Local::new 形成 nested `LocalCtx<LocalCtx<...>>`,内层不满足 Observable | **禁止中间 box_it**,一条 chain 走到底,只在最后一次 `.box_it()` |
| `let stage1 = ...; let stage2 = ...;` 分阶段命名绑定 | 打断链式、强制固化中间类型、让 nested 错误浮出水面 | 单一 chain + 末尾 box_it,不命名中间变量 |
| `boxed.box_it().box_it()` | box_it 不是幂等的两次调用有意义 | box_it 一次即可;boxed 仍是 Local,chain 后再 box 一次也行 |
| `.collect::<String>()` 但 upstream Item = String | collect 要求 `Item = char`（Extend<char> for String） | 用 `scan_map(String::new(), \|acc, s\| acc + s).last()` |
| `.flat_map(\|t\| Local::from_iter(&vec))` | 传引用而非 owned 值 | `.flat_map(\|toks\| Local::from_iter(toks))`（toks 是 owned） |
| reducer `*state = state.advance(ch); return state;` | reducer 签名是 `FnMut(&mut Acc, Item) -> Output`，不是 `FnMut(Acc, Item) -> (Acc, Output)` | 纯 &mut 就地修改，返回 Output；RxRust 自动持有 state |
| 闭包捕获外部 `&mut` 变量如 `let mut result = vec![]; .map(\|x\| result.push(x))` | 捕获可变引用在 FnMut 违反唯一借用 | `.scan_map(Vec::new(), \|acc, x\| { acc.push(x); acc.clone() })` |

---

## 9. 提交前自检

每个 Rust 文件提交前：

- [ ] **0 个 `Rc<RefCell>`** 在 helper/reducer/pipeline
- [ ] **0 个命令式 `for + push` 累积 → flat_map**
- [ ] **状态 enum 化** — 不用 bool flags
- [ ] **reducer 内嵌纯函数** — 不内嵌 50 行 if-else
- [ ] **0 个 helper 构造 Observable** — helper 用原语迭代器
- [ ] **tracing 在 tap** — 不在 map 闭包
- [ ] **endpoint 仅用 collect/last + subscribe** — 不 subscribe_boxed
- [ ] **100% 不回头查 rxrust 源码** — 有疑问看本 SKILL 第 8 节对照表

---

## 10. 验收标准

- `cargo test` 通过
- 无新增 `Rc<RefCell>` / `borrow_mut()` 调用
- 无 `while` / 索引 for 循环 / subscribe_boxed 幻觉 API
- 无 helper 函数构造 Observable
- 无 reducer 内捕获外部 &mut 变量

输出格式:
```
Stage: <stage>
Pattern: <scan_map | flat_map | collect | tap | map | ...>
Before: <违反所有权的写法>
After: <正确写法>
SourceRef: <来自 rxrust src 哪一行>
```
