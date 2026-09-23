# sasspile-rx Architecture

> rxrust 1.0.0-rc.5 驱动的 SCSS 编译器 —— 管线 = 算子链, 算子即状态机, 零 GC 模式。

---

## 0. 核心铁律 (Agent 必读)

```
╔══════════════════════════════════════════════════════════════════╗
║  0.1 思考问题前, 先读 rxrust 源码                                 ║
║  0.2 从框架内部找答案, 不自己造轮子                               ║
║  0.3 rxrust IS Rust 所有权 — scan_map 消费自身, flat_map 转移自身  ║
║  0.4 禁止 GC 模式: Rc<RefCell>/Arc<Mutex>/for+push/subscribe+push ║
╚══════════════════════════════════════════════════════════════════╝
```

### 0.1 读源码优先

写代码前, **必须先读 rxrust 源码** (`~/.cargo/registry/src/.../rxrust-1.0.0-rc.5/`):

| 问题 | 读哪里 |
|------|--------|
| 算子签名 / 返回类型 | `src/observable.rs` (map/scan_map/flat_map/collect/last) |
| 创建 Observable | `src/factory.rs` (of/from_iter/from_stream/empty) |
| Shared vs Local 调度 | `src/rc.rs`, `src/scheduler.rs` |
| 订阅返回类型 | `src/observable.rs` 141 行 `subscribe` 签名 |
| 类型擦除 | `src/observable.rs` 1944 行 `box_it` |
| Subscription 包装 | `src/subscription/source_with_dynamic.rs` |

**原则**: 框架已经有的, 不手写。框架禁止的不绕过。

### 0.2 所有权思维 vs GC 思维

| GC 思维 (禁止) | Rust 所有权 (正确) |
|---|---|
| `Arc<Mutex<T>>` 共享可变 | `oneshot channel` 单次值转移 |
| `Rc<RefCell<T>>` + `borrow_mut()` | `scan_map(State::new(), reducer)` |
| `let mut v = vec![]; for x in items { v.push(x) }` | `items.flat_map(f).collect()` |
| `obs.subscribe(\|x\| cell.borrow_mut().push(x))` | `obs.last().subscribe(\|r\| ...)` |
| `std::mem::take(&mut *lock)` 读共享 | `rx.blocking_recv()` 消费 channel |

---

## 1. 管线总览 (实际实现)

```
   ┌──────────────────────────────────────────────────────────────────┐
   │              compile_pipeline(input: &str) -> String               │
   └───────────────────────────────┬──────────────────────────────────┘
                                   │
          ┌────────────────────────▼────────────────────────┐
          │  Source: Vec<String> → Shared::from_stream      │
          │  (input.lines() → to_string 是唯一的 clone,      │
          │   因为 Shared 需要 Send + 'static)                │
          └────────────────────────┬─────────────────────────┘
                                   │ String (每行)
          ┌────────────────────────▼────────────────────────┐
          │  Phase 1: 指令展开                               │
          │  scan_map(CompileState::new(), dispatch_pass)   │
          │  ─ &mut CompileState 就地 mutate                 │
          │  ─ 产出 Vec<String> (展开后的 style lines)       │
          └────────────────────────┬─────────────────────────┘
                                   │ Vec<String>
          ┌────────────────────────▼────────────────────────┐
          │  flat_map(Shared::from_stream(from_iter))       │
          │  ─ 有序组合 (composition, 非 merge)             │
          │  ─ Vec 展开为独立 String 事件                    │
          └────────────────────────┬─────────────────────────┘
                                   │ String (style line)
          ┌────────────────────────▼────────────────────────┐
          │  Phase 2: CSS AST 构建                          │
          │  scan_map(CssBuilder::new(), feed)              │
          │  ─ &mut CssBuilder 就地 mutate                   │
          │  ─ 产出 Vec<CssNode>                             │
          └────────────────────────┬─────────────────────────┘
                                   │ Vec<CssNode>
          ┌────────────────────────▼────────────────────────┐
          │  flat_map(Shared::from_stream(from_iter))       │
          │  ─ Vec<CssNode> 展开为独立 CssNode 事件          │
          └────────────────────────┬─────────────────────────┘
                                   │ CssNode
          ┌────────────────────────▼────────────────────────┐
          │  Phase 3: 渲染                                  │
          │  map(\|node\| render_node(&node))                │
          │  ─ 借用 &CssNode, 不 clone                       │
          │  ─ 产出 String (单行 CSS)                        │
          └────────────────────────┬─────────────────────────┘
                                   │ String
          ┌────────────────────────▼────────────────────────┐
          │  收集 + 订阅                                     │
          │  collect::<Vec<String>>().last()                │
          │  ─ 汇聚为 Option<Vec<String>>                   │
          │  .subscribe(move \|css_vec\| {                   │
          │      tx_opt.take().map(\|t\| t.send(...))         │
          │  })                                             │
          │  ─ oneshot channel 单次值转移                     │
          └────────────────────────┬─────────────────────────┘
                                   │
          ┌────────────────────────▼────────────────────────┐
          │  block_in_place + block_on(handle.source.source) │
          │  rx.blocking_recv() → String                    │
          │  ─ 零 Arc, 零 Mutex, 零 clone                   │
          └─────────────────────────────────────────────────┘
```

---

## 2. 算子签名速查 (来自 rxrust 源码)

### 2.1 scan_map — 状态机核心

```rust
// src/observable.rs 402 行
fn scan_map<Acc, Output, F>(self, initial: Acc, f: F) -> Self::With<ScanMap<Self::Inner, F, Acc>>
where
    F: for<'a> FnMut(&mut Acc, Self::Item<'a>) -> Output,
```

- `&mut Acc` — 就地修改累加器, 不 clone
- `Output` — 本次 emit 到下游的值
- 调 N 次 → 产出 N 个 Output

### 2.2 flat_map — 有序组合

```rust
// src/observable.rs 2400 行
fn flat_map<F, Inner>(self, f: F) -> Self::With<FlatMap<Self::Inner, F, Inner>>
where
    F: for<'a> FnMut(Self::Item<'a>) -> Inner,
    Inner: Context<Inner: ObservableType<Err = Self::Err>>,
```

- `Shared::from_stream(futures::stream::iter(v))` — 多线程有序组合
- `Local::from_iter(v)` — 单线程有序组合

### 2.3 collect + last — 终端汇聚

```rust
// src/observable.rs 1515 行
fn collect<C>(self) -> Self::With<Collect<Self::Inner, C>> where C: Default

// src/observable.rs 573 行
fn last(self) -> Self::With<Last<Self::Inner>>
```

- `collect::<Vec<String>>()` — 汇聚 N 个 String 为 1 个 Vec<String>
- `.last()` — 取最后一个 (对 collect 来说就是唯一那个)

### 2.4 subscribe — 唯一终点

```rust
// src/observable.rs 141 行
fn subscribe<F, U>(self, f: F) -> U
where
    F: for<'a> FnMut(Self::Item<'a>),
    Self::Inner: CoreObservable<Self::With<FnMutObserver<F>>, Unsub = U>,
```

- 返回 `U` = `Self::Unsub`
- Shared 上下文: `U` = `SourceWithDynamicSubs<SourceWithDynamicSubs<TaskHandle, ...>, ...>`
- 需要 `.source.source` 深入拿到 `TaskHandle` 才能 `block_on`

---

## 3. 终端模式: oneshot channel (非 GC 共享)

```rust
// ✅ 正确: oneshot = 单次值转移
let (tx, rx) = tokio::sync::oneshot::channel::<String>();
let mut tx_opt = Some(tx);

// ... 管线 ...
    .subscribe(move |css_vec: Vec<String>| {
        if let Some(tx) = tx_opt.take() {  // take() = 一次性 move
            let _ = tx.send(css_vec.join("\n"));
        }
    });

// 驱动管线
tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(handle.source.source);
});

// 消费结果
rx.blocking_recv().unwrap_or_default()
```

**为什么不是 Arc<Mutex>**:
- `Arc<Mutex<T>>` = GC 共享可变状态, 多处读写, 锁竞争
- `oneshot` = 单次值转移, 发送后 tx 销毁, rx 消费后销毁, 零竞争

---

## 4. Shared vs Local 调度

| 场景 | 调度器 | 说明 |
|------|--------|------|
| 多线程 source + 多线程处理 | `Shared::from_stream` | 全局 tokio runtime, 2 workers |
| 单线程有序组合 | `Local::from_iter` | 当前线程, 无调度开销 |
| 跨线程边界 | `Shared` | 数据需要 `Send + 'static` |
| 终端订阅 | `Shared` 返回 `TaskHandle` | 需要 `block_in_place + block_on` 驱动 |

**关键**: `flat_map + Shared::from_stream` = 多线程有序组合 (composition, 不是 merge)

---

## 5. 文件结构

```
src/
├── lib.rs                    — 入口, 模块声明
├── css/
│   ├── mod.rs                — CssNode + CssBuilder + render_node 导出
│   ├── node.rs               — CssNode 枚举 + render_node (AST → String)
│   └── builder.rs            — CssBuilder (scan_map reducer, 构建 AST)
└── directive/
    ├── mod.rs                — 模块声明
    ├── pipeline.rs           — compile_pipeline 入口 + dispatch_pass
    └── state.rs              — CompileState + Collecting 状态机
```

---

## 6. 状态机: CompileState

```rust
// src/directive/state.rs
pub struct CompileState {
    pub phase: Phase,
    pub collecting: Collecting,        // 4 态: None | For | Each | If | MixinDef
    pub selector_stack: Vec<String>,   // 嵌套选择器栈
    pub nesting_depth: usize,
    pub current_mixin_name: Option<String>,
    pub scope: Scope,                  // 变量 + mixin 定义
}

pub enum Collecting {
    None,
    For { var_name: String, values: Vec<String>, body: Vec<String> },
    Each { var_name: String, items: Vec<String>, body: Vec<String> },
    If { body: Vec<String>, branch_taken: bool },
    MixinDef { params: Vec<(String, Option<String>)> },
}
```

`dispatch_pass(state: &mut CompileState, token: String) -> Vec<String>`:
- `&mut CompileState` — 就地修改, 不 clone
- 返回 `Vec<String>` — 下游 flat_map 展开

---

## 7. 渲染: render_node

```rust
// src/css/node.rs
pub fn render_node(node: &CssNode) -> String  // 借用 &, 不 clone
```

递归渲染 CssNode AST 为 CSS 字符串, 支持:
- `Rule { selector, children }` — 嵌套规则
- `Declaration { property, value }` — 属性声明
- `AtRoot { children }` — @at-root 提升
- `AtRule { query, children }` — @media / @keyframes
- `Comment(String)` — 注释

---

## 8. 调试协议 (强制)

```
4步流程:
1. SPAN 插桩: 疑似路径每个入口/出口加 span
2. TRACE 采集: RUST_LOG=trace 收集证据
3. 根因定位: 必须引用具体 span + 字段值
4. 修复验证: 修复后清理临时 span
```

```rust
// 正确: span 边界追踪
fn dispatch_pass(state: &mut CompileState, token: String) -> Vec<String> {
    let _span = info_span!("dispatch_pass", phase = ?state.phase, token = %token).entered();
    // ...
}
```

---

## 9. 提交前自检

- [ ] **0 个 `Rc<RefCell>` / `Arc<Mutex>`** — 状态由 scan_map 管理
- [ ] **0 个命令式 `for + push`** — 用 flat_map + collect
- [ ] **render_node 借用 `&CssNode`** — 不 clone
- [ ] **终端用 oneshot** — 不用 Arc<Mutex>
- [ ] **tracing 在 tap** — 不在 map 闭包
- [ ] **先读 rxrust 源码** — 不猜 API, 不造轮子
- [ ] **单文件 ≤ 500 行** — 源码和测试分别计算

---

## 10. 工程约定

- **Rust edition**: 2024, toolchain 1.97
- **clippy pedantic**: `cargo clippy -- -W clippy::all -W clippy::pedantic`
- **RUST_LOG=trace**: span 调用链全可见
- **推送**: SSH 方式 `git push github main`
- **commit 后等用户确认** — 不自动推送
