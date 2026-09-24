---
name: rxrust-ownership
description: Rust 所有权 + rxrust 算子驱动 SCSS 编译器开发.当写 Rust 代码、实现编译器功能、修复 spec 失败时激活.核心原则:rxrust IS Rust 所有权 — scan_map 算子即状态机, flat_map 有序组合, mpsc channel 终端单次值转移.禁止 Rc<RefCell>/Arc<Mutex>/for 循环/subscribe+push 等 GC 模式。
allowed-tools: Read, Write, Edit, MultiEdit, ListDir
license: MIT
metadata:
  author: sasspile-rx
  version: "6.0"
---

# v6.0 Shared 多线程版 — 基于实际管线编译验证

本 SKILL 来自 sasspile 管线的实际编译通过代码 + rxrust 1.0.0-rc.5 源码阅读。

## ⚠️ 第零铁律 (最高优先级): 读源码优先, 从框架内部实现

**写代码前, 必须先读 rxrust 源码。不猜 API, 不造轮子。**

| 问题 | 读哪里 |
|------|--------|
| 算子签名 / 返回类型 | `src/observable.rs` |
| 创建 Observable | `src/factory.rs` |
| Shared vs Local | `src/rc.rs` (MutRc / MutArc 对比) |
| subscribe 返回类型 | `src/observable.rs` 141 行 |
| box_it 类型擦除 | `src/observable.rs` 1944 行 |
| Subscription 嵌套 | `src/subscription/source_with_dynamic.rs` |
| 调度器 | `src/scheduler.rs` |

源码路径: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rxrust-1.0.0-rc.5/`

---

## ⚠️ 核心铁律: chain 是一条完整管道, 不要拆分中间变量

### 三阶段反应式范式

```
1. 入口流      Shared::subject::<String, E>()           把外部数据变成 Observable
2. 内部迭代器  scan_map(Acc::new(), reducer)             消费旧状态 → 产出新状态 + emit 值
3.  合并收集  flat_map → collect/last → subscribe        展开子 Vec,消费整条流
```

**口诀: 流进来, 算子中间过, subscribe 消费输出。**

---

## 0. 所有权 vs GC 思维对照

| GC 思维 (绝对禁止) | Rust 所有权 (正确) |
|---|---|
| `Arc<Mutex<T>>` 共享可变 | `std::sync::mpsc::channel` 单次值转移 |
| `Rc<RefCell<T>>` + `borrow_mut()` | `scan_map(State::new(), reducer)` |
| `let mut v = vec![]; for x in items { v.push(x) }` | `items.flat_map(f).collect()` |
| `obs.subscribe(\|x\| shared.lock().push(x))` | `obs.last().subscribe(\|r\| ...)` |
| `std::mem::take(&mut *lock)` 读共享 | `rx.recv()` 消费 channel |

---

## 1. 算子签名速查 (来自 rxrust 源码)

### 1.1 scan_map — 状态机核心

```rust
// src/observable.rs 402 行
fn scan_map<Acc, Output, F>(self, initial: Acc, f: F) -> Self::With<ScanMap<Self::Inner, F, Acc>>
where
    F: for<'a> FnMut(&mut Acc, Self::Item<'a>) -> Output,
```

**reducer 签名**: `FnMut(&mut Acc, Item) -> Output`
- `&mut Acc` = 就地修改累加器 (不 clone)
- `Item` = 当前 item (by value move)
- `Output` = 本次 emit 的值

### 1.2 flat_map — 有序组合

```rust
// src/observable.rs 2400 行
fn flat_map<F, Inner>(self, f: F) -> Self::With<FlatMap<Self::Inner, F, Inner>>
where
    F: for<'a> FnMut(Self::Item<'a>) -> Inner,
    Inner: Context<Inner: ObservableType<Err = Self::Err>>,
```

**实战**:
```rust
// Shared 多线程有序组合
.flat_map(|v: Vec<String>| Shared::from_iter(v))
.flat_map(|v: Vec<CssNode>| Shared::from_iter(v))
```

### 1.3 collect + last — 终端汇聚

```rust
fn collect<C>(self) -> Self::With<Collect<Self::Inner, C>> where C: Default
fn last(self) -> Self::With<Last<Self::Inner>>
```

### 1.4 subscribe — 唯一终点

```rust
fn subscribe<F, U>(self, f: F) -> U
where
    F: for<'a> FnMut(Self::Item<'a>),
    Self::Inner: CoreObservable<Self::With<FnMutObserver<F>>, Unsub = U>,
```

---

## 2. 终端模式: std::sync::mpsc channel

```rust
let (tx, rx) = std::sync::mpsc::channel::<String>();

// ... 管线链 ...
    .subscribe(move |css_vec: Vec<String>| {
        let _ = tx.send(css_vec.join("\n"));
    });

// 注入数据 + complete
input.lines().for_each(|line| subject.clone().next(line.to_string()));
subject.clone().complete();

// 消费结果
rx.recv().unwrap_or_default()
```

**为什么用 mpsc 而非 Arc<Mutex>**:
- mpsc = 单次值转移, tx 发送后销毁, rx 消费后销毁
- Arc<Mutex> = GC 共享可变, 多处读写, 锁竞争

**为什么不用 tokio oneshot**:
- 管线流程完全同步 (push all → complete → terminal 已执行完毕)
- 不需要 tokio runtime 开销

---

## 3. 完整管线模式 (sasspile v6.0 — Shared 多线程)

```rust
pub fn compile_pipeline(input: &str) -> String {
    let subject = Shared::subject::<String, Infallible>();
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    subject.clone()
        // Phase 1: scan_map(CompileState) 就地 mutate
        .scan_map(CompileState::new(), dispatch_pass)
        // flat_map 有序组合展开 Vec<String>
        .flat_map(|v: Vec<String>| Shared::from_iter(v))
        // Phase 2: scan_map(CssBuilder) 构建 AST
        .scan_map(CssBuilder::new(), |builder: &mut CssBuilder, line: String| -> Vec<CssNode> {
            builder.feed(&line)
        })
        // flat_map 展开 Vec<CssNode>
        .flat_map(|v: Vec<CssNode>| Shared::from_iter(v))
        // 汇聚 CssNode
        .collect::<Vec<CssNode>>()
        .last()
        // 汇聚后处理 (@media 合并等)
        .map(|nodes| post_process(nodes))
        .flat_map(|nodes| Shared::from_iter(nodes))
        // Phase 3: 借用渲染 (&CssNode → String)
        .map(|node: CssNode| render_node(&node))
        // 汇聚 String
        .collect::<Vec<String>>()
        .last()
        // subscribe = 执行边界, tx 转移终态
        .subscribe(move |css_vec: Vec<String>| {
            let _ = tx.send(css_vec.join("\n"));
        });

    // 注入所有行
    input.lines().for_each(|line| subject.clone().next(line.to_string()));
    // complete → 触发 collect emit → terminal 执行 → tx.send 被调用
    subject.clone().complete();

    // 同步等待终态产物
    rx.recv().unwrap_or_default()
}
```

**所有权流转**:
```
input: &str (借用)
  → line.to_string() (source 阶段唯一 clone, Shared 需要 'static)
  → scan_map(CompileState)  (&mut 就地 mutate, 状态在算子内部)
  → flat_map(展开 Vec<String>)
  → scan_map(CssBuilder)   (&mut 就地 mutate, 状态在算子内部)
  → flat_map(展开 Vec<CssNode>)
  → map(render_node(&node)) (借用 &CssNode)
  → collect → last → subscribe(tx.send) (tx move 进闭包)
  → rx.recv() (消费 String, 返回调用者)
```

**零 Arc<Mutex>, 零外部共享状态 — scan_map 算子即状态机。**

---

## 4. 绝对禁止 (GC 思维)

| 反模式 | 为什么违反 Rust 所有权 | 正确做法 |
|---|---|---|
| `Rc<RefCell<T>>` + `borrow_mut()` | 模拟 GC 共享可变 | `scan_map(State::new(), reducer)` |
| `let mut v = Vec::new(); for x in items { v.push(f(x)) }` | 命令式累积 | `items.flat_map(f).collect()` |
| `obs.subscribe(\|x\| cell.borrow_mut().push(x))` | 手动偷取流值 | `obs.last().subscribe(\|r\| ...)` |
| `Arc<Mutex<String>>` + `lock().push_str()` | GC 共享可变 | `mpsc channel` + `tx.send()` |
| helper 函数内构造 Observable | helper 绕开 pipeline | helper 内用原语迭代器 |
| `unwrap()` 在 stage 内 | 传播 panic | `Err(CompileError)` 或 `Option` |
| 猜测 rxrust API | 幻觉方法 | **先读源码, 再写代码** |
| Shared 管线用 `&str` 试图绕过 'static | 违反 'static + Send 约束 | 接受 `String` 入口, 保证中间零 clone |

---

## 5. box_it 使用

```rust
// 只在最终出口 .box_it() 一次
let boxed: LocalBoxedObservable<'static, String, Infallible> = source
    .scan_map(...)
    .flat_map(...)
    .box_it();   // ← 最终出口擦除类型

// ❌ 禁止中间 box_it + Local::new (形成 nested LocalCtx)
```

---

## 6. 调试 span 约定

```rust
fn dispatch_pass(state: &mut CompileState, token: String) -> Vec<String> {
    let _span = info_span!("dispatch_pass", phase = ?state.phase, token = %token).entered();
    // ...
}

pub fn render_node(node: &CssNode) -> String {
    let _span = info_span!("render_node", variant = ?node_variant_name(node)).entered();
    // ...
}
```

字段命名约定: `stage`, `module`, `id`, `elapsed_ms`, `phase`, `token`, `variant`

---

## 7. 完整反应式指南 (20 章 + 源码定位)

详见 **`docs/rxrust-reactive-guide.md`** (约 39K 字符, 基于 rxrust 1.0.0-rc.5 源码全量慢读).

**核心新增洞察** (v6.0):

1. **Observer `error(self)` / `complete(self)` 的 move 语义** 是类型层面的终结约束, 不需要运行时检查.
2. **`for<'a>` HRTB** 保证闭包在任意 lifetime 下适用 — 这是算子能串联的根本原因.
3. **scan_map 的 acc 放在 Observer 内部** (而非外部), 实现 per-subscription 状态隔离.
4. **flat_map = Map + MergeAll** 组合, 用 `concurrent` 参数控制并发 vs 顺序.
5. **from_iter 内手写 for 是合法的** (源码已验证), 但**必须在循环顶加 `if observer.is_closed() { break; }`**.
6. **Subject re-entrant next 会 panic** — 用 `delay(0)` 创建 async boundary 才能反馈.
7. **Shared vs Local 选择**: Shared = `Arc<Mutex<Subscribers>>` 多线程广播; Local = `Rc<RefCell<Subscribers>>` 单线程. sasspile 用 Shared.

**7 条 AI 铁律** (详见指南第 16 章):
- 铁律 1: 禁止手写循环处理流数据
- 铁律 2: scan_map 是唯一持有可变状态的地方
- 铁律 3: subscribe 是终端, 不在 subscribe 内再 subscribe
- 铁律 4: 利用 Unsub 类型判断同步 vs 异步
- 铁律 5: scan_map 的 FnMut 签名必须精确为 `FnMut(&mut Acc, Item) -> Output`
- 铁律 6: flat_map 的 Inner 必须是 Observable (用 `Shared::from_iter(v)`)
- 铁律 7: error/complete 是终结信号, Observer move 后不可再用

**附录清单 (提交前必查)**: `docs/rxrust-reactive-guide.md` 附录 B

---

## 7b. 提交前自检

- [ ] **先读 rxrust 源码** — 不猜 API, 不造轮子
- [ ] **0 个 `Rc<RefCell>` / `Arc<Mutex>`** — 状态由 scan_map 管理
- [ ] **0 个命令式 `for + push`** — 用 flat_map + collect
- [ ] **render_node 借用 `&T`** — 不 clone
- [ ] **终端用 mpsc channel** — 不用 Arc<Mutex>
- [ ] **tracing 在 tap** — 不在 map 闭包
- [ ] **endpoint 仅用 collect/last + subscribe** — 不 subscribe_boxed
- [ ] **单文件 ≤ 500 行**
- [ ] **Shared 入口用 String** — 不试图用 &str 绕过 'static
- [ ] **对照附录 B 清单逐项检查**

---

## 8. 常见错误对照

| 错误 | 原因 | 正确 |
|---|---|---|
| `SourceWithDynamicSubs is not a Future` | collect/last 在 Shared 返回嵌套 Subscription | 管线是同步的, 不需要 block_on |
| `cannot move out of tx in FnMut` | tx 不能 move 出 FnMut | `move` 闭包 + 直接 send (mpsc) |
| `closure may outlive borrowed value` | 'static 闭包捕获局部借用 | 用 mpsc (tx move 进闭包) |
| `the trait bound ... is not satisfied` | 缺少 `Send + 'static` | 检查 Shared 上下文约束 |
| `cannot find macro debug in this scope` | 缺少 tracing:: 前缀 | `tracing::debug!` |
| `Local` 无法跨线程传递 | Local = Rc, 非 Send | 改用 Shared |

---

## 9. 验收标准

输出格式:
```
Pattern: <scan_map | flat_map | collect | mpsc | map | ...>
Ownership: <借用 | move | &mut 就地修改>
Before: <违反所有权的写法>
After: <正确写法>
SourceRef: <来自 rxrust src 哪一行>
```
