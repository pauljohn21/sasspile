## Design: main → rx 能力迁移架构

### 0. 核心原则（不可违反）

> rx 不是 main 的移植，是以 rxrust 响应式约束重新实现。

```
铁律 (v5.0 — 基于编译通过的实际代码):
0. 先读 rxrust 源码, 从框架内部实现, 不猜 API 不造轮子
1. scan_map (CompileState / CssBuilder)  ← 算子即状态机, 零外部状态
2. flat_map(Shared::from_stream(iter))   ← 多线程有序组合 (composition)
3. collect/last + oneshot + subscribe    ← 唯一收集出口, 单次值转移
4. render_node(&node) 借用渲染           ← 不 clone
5. 零 Arc<Mutex>/Rc<RefCell>             ← 禁止 GC 共享可变
```

---

### 1. 当前 rx 管线（v5.0 — 编译通过）

```rust
// src/directive/pipeline.rs
pub fn compile_pipeline(input: &str) -> String {
    let lines: Vec<String> = input.lines().map(|l| l.to_string()).collect();
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let mut tx_opt = Some(tx);

    let handle = Shared::from_stream(futures::stream::iter(lines))
        // Phase 1: 指令展开 — scan_map(CompileState) 内部 &mut 就地修改
        .scan_map(CompileState::new(), dispatch_pass)
        // 多线程有序组合展开 Vec → 独立 String 事件
        .flat_map(|v| Shared::from_stream(futures::stream::iter(v)))
        // Phase 2: CSS AST — scan_map(CssBuilder) 内部 &mut 构建 AST
        .scan_map(CssBuilder::new(), |builder, line| builder.feed(&line))
        // 有序组合展开 Vec<CssNode> → 独立 CssNode
        .flat_map(|v| Shared::from_stream(futures::stream::iter(v)))
        // Phase 3: 借用渲染 (render_node(&node), 零 clone)
        .map(|node| render_node(&node))
        // 汇聚 + 终端: oneshot 单次值转移
        .collect::<Vec<String>>()
        .last()
        .subscribe(move |css_vec| {
            if let Some(tx) = tx_opt.take() {
                let _ = tx.send(css_vec.join("\n"));
            }
        });

    // 驱动 Shared 管线的 TaskHandle
    // collect/last 在 Shared 返回 SourceWithDynamicSubs<SourceWithDynamicSubs<TaskHandle>>
    // 需要 .source.source 深入拿到 Future
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(handle.source.source);
    });

    // 阻塞消费
    rx.blocking_recv().unwrap_or_default()
}
```

**所有权流转**:
```
input: &str (借用)
  → lines: Vec<String> (唯一 clone, 因 Shared 需 Send+'static)
  → scan_map(CompileState) — &mut 就地 mutate, 状态在算子内部
  → flat_map — Vec 展开为独立事件
  → scan_map(CssBuilder) — &mut 就地 mutate, 状态在算子内部
  → flat_map — CssNode 展开
  → map(render_node(&node)) — 借用, 零 clone
  → collect → last → subscribe(tx.send) — tx move 进闭包
  → block_on(handle.source.source) — 驱动 TaskHandle
  → rx.blocking_recv() — 消费, 返回 String
```

---

### 2. 目标 rx 管线（v6.0 — 下一步）

新增能力（在现有 v5.0 骨架上扩展）:

- **@media 内合并**: CssBuilder 跟踪 @media query, 相同 query 合并 children
- **@extend 选择器分组**: Phase 1.5 extend 暂存 + 规则闭合时应用
- **嵌套 @at-root**: CssBuilder 遇到 AtRoot 时 children 提升到顶层
- **@keyframes 特殊序列化**: 百分比节点格式化
- **函数求值**: dispatch_pass 内 try_eval_builtin, 替换字面量

所有扩展都在现有 scan_map(CompileState) 或 scan_map(CssBuilder) 内部完成, 不改变管线拓扑。

---

### 3. 能力 #1: css-ast-construction ✅ 已完成

#### 数据结构 (src/css/node.rs)

```rust
pub enum CssNode {
    Rule { selector: String, children: Vec<CssNode> },
    Declaration { property: String, value: String },
    AtRoot { children: Vec<CssNode> },
    AtRule { query: String, children: Vec<CssNode> },
    Comment(String),
}
```

#### scan_map builder (src/css/builder.rs)

```rust
pub struct CssBuilder {
    pub output: Vec<CssNode>,
    pub rule_stack: Vec<RuleFrame>,
}

impl CssBuilder {
    pub fn feed(&mut self, line: &str) -> Vec<CssNode> { ... }
}
```

---

### 4. 能力 #2: selector-nesting ✅ 已完成

`dispatch_pass` 中:
```rust
if !t.starts_with('@') && t.contains('{') {
    let parent = state.selector_stack.last().map(|s| s.as_str());
    let full_selector = match parent {
        Some(ref p) if selector_part.contains('&') => selector_part.replace('&', p),
        Some(ref p) => format!("{p} {selector_part}"),
        None => selector_part.to_string(),
    };
    state.selector_stack.push(full_selector.clone());
}
```

---

### 5. 能力 #3: for/each/mixin/include ✅ 已完成

见 `dispatch_pass` + `finalize_collecting` (pipeline.rs):
- `@for $i from 1 through 3 { ... }` — 单行/多行
- `@each $item in a, b, c { ... }` — 单行/多行
- `@mixin name($param: default) { ... }` — 多行收集
- `@include name($arg)` — fold 参数回退默认值

---

### 6. 能力 #4: zero-gc-pattern ✅ 已完成

| 之前 (GC 思维) | 现在 (Rust 所有权) |
|---|---|
| `Arc<Mutex<String>>` 共享 | `oneshot channel` 单次值转移 |
| `Rc<RefCell<T>>` + `borrow_mut()` | `scan_map(State, reducer)` |
| `std::mem::take(&mut *lock)` | `rx.blocking_recv()` |

---

### 7. 文件拆分 (实际)

```
src/css/
├── mod.rs           — 模块声明
├── node.rs          — CssNode + render_node (91 行)
└── builder.rs       — CssBuilder (187 行)

src/directive/
├── mod.rs           — 模块声明
├── pipeline.rs      — compile_pipeline + dispatch_pass + helper (376 行)
└── state.rs         — CompileState + Collecting (169 行)
```

所有文件 ≤ 500 行, 符合约束。

---

### 8. 源码参考清单 (开发必读)

| 开发问题 | 读哪里 |
|---------|--------|
| 算子签名 | `src/observable.rs` |
| 创建 Observable | `src/factory.rs` |
| 订阅返回类型 | `src/observable.rs` 141 行 |
| Subscription 嵌套 | `src/subscription/source_with_dynamic.rs` |
| 调度器 / TaskHandle | `src/scheduler.rs` |
| Shared 上下文 | `src/rc.rs` |

源码路径: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rxrust-1.0.0-rc.5/`
