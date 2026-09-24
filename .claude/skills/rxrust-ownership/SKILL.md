---
name: rxrust-ownership
description: Rust 所有权 + rxrust 算子驱动 SCSS 编译器开发.当写 Rust 代码、实现编译器功能、修复 spec 失败时激活.核心原则:rxrust IS Rust 所有权 — scan_map 算子即状态机, flat_map 有序组合, mpsc channel 终端单次值转移.禁止 Rc<RefCell>/Arc<Mutex>/for 循环/subscribe+push 等 GC 模式。
allowed-tools: Read, Write, Edit, MultiEdit, ListDir
license: MIT
metadata:
  author: sasspile-rx
  version: "7.0"
---

# v7.0 — "Rust Ownership = Reactive Answer" 范式

本 SKILL 基于 sasspile 管线实际编译通过代码 + rxrust 1.0.0-rc.5 源码全量慢读。

## 元认知 (必读)

```
Reactor/Flux 的 GC 假设：
  一切数据被 GC 管理，任何 lambda 能自由捕获/共享/修改任意对象
  ⇒ 运行时混沌的温床 (CME / 数据竞争 / 隐式反馈循环)

Rust 的解答：
  闭包捕获是三选一 (move/&/&) — 编译期 X 光
  &mut Acc = 零锁零 clone 的状态机，Flux 做不到

结论：
  NOT "rxrust 模仿 Flux"
  BUT "Rust ownership 自然解决 Flux 想解决但解决不好的问题"
```

---

## ⚠️ 第零铁律: 用 Ownership 三态思考, 不翻译 Flux

### 三态设计表 (任何管线问题的出发点)

| 你要做什么? | 闭包类型 | rxrust 算子 | 示例 |
|-------------|---------|------------|------|
| 消费 T → 产出 U | `FnMut(T) -> U` | `map` | `.map(\|n\| render_node(&n))` |
| 借用 T 判断 bool | `FnMut(&T) -> bool` | `filter` | `.filter(\|t\| t.is_valid())` |
| &mut 持有状态 | `FnMut(&mut Acc, T) -> Out` | **`scan_map`** | `.scan_map(CompileState::new(), reducer)` |
| T → 1:N 子流 | `FnMut(T) -> Inner` (Observable) | `flat_map` | `.flat_map(\|v\| Shared::from_iter(v))` |
| 副作用不改流 | `FnMut(&T)` | `tap` | `.tap(\|v\| tracing::info!(%v))` |
| 终态转移 (Terminal) | `move` closure | `subscribe` | `.subscribe(move \|v\| tx.send(v))` |
| 汇聚成集合 | — | `collect::<Vec<T>>()` | `.collect::<Vec<_>>()` |

**任何管线问题: 先把数据流用上面的表过一遍, 不要想 "Flux 怎么做"。**

---

## ⚠️ 源码优先: 不猜 API, 不造轮子

源码路径: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rxrust-1.0.0-rc.5/`

| 问题 | 读哪里 |
|------|--------|
| Observer trait (error/complete 的 move 语义) | `src/observer.rs` |
| Subject re-entrant panic / broadcast | `src/subject/subject_core.rs` |
| scan_map 签名 (`&mut Acc`) | `src/ops/scan_map.rs` |
| flat_map = MergeAll + concurrent | `src/ops/flat_map.rs`, `src/ops/merge_all.rs` |
| collect (Extend trait 汇聚) | `src/ops/collect.rs` |
| from_iter + is_closed 守卫 | `src/observable/from_iter.rs` |
| Connectable (publish/connect) | `src/observable/connectable.rs` |
| Local vs Shared (`MutRc`/`MutArc`) | `src/rc.rs` |

---

## 1. scan_map — 唯一状态栖息地

```rust
// 闭包签名: FnMut(&mut Acc, Item) -> Output
source.scan_map(CompileState::new(), |state: &mut CompileState, line: String| -> Vec<String> {
    state.line_count += 1;    // &mut 排他修改
    state.dispatch(line)      // 产出 Vec<String>
})
.flat_map(|v: Vec<String>| Shared::from_iter(v))  // 展开 Vec
```

**为什么 scan_map (不是 scan)**:
- `scan(init, f)`: Flux 风格, 每次 clone acc, 发射中间结果
- `scan_map(init, f)`: Rust 风格, `&mut Acc` 就地修改, 零 clone, 返回 Vec<T> 用于 1:N 展平

---

## 2. flat_map + 子流 — 1:N 展平

```rust
// flat_map 闭包返回 Observable (不是裸 vec)
.flat_map(|v: Vec<String>| Shared::from_iter(v))  // ✅ Inner: Observable
.flat_map(|v| v)                                   // ❌ Vec 不是 Observable, 编译失败
```

---

## 3. 终端模式: mpsc channel

```rust
let (tx, rx) = std::sync::mpsc::channel::<String>();

// 管线链...
    .subscribe(move |v: Vec<String>| {
        let _ = tx.send(v.join("\n"));  // tx move 进闭包
    });

input.lines().for_each(|line| subject.clone().next(line.to_string()));
subject.clone().complete();
rx.recv().unwrap_or_default()
```

---

## 4. 三阶段反应式范式

```
1. 入口流      Shared::subject::<String, E>()     把外部数据变成 Observable
2. 中间算子    scan_map / flat_map / map / filter   消费自身 → 产出新值
3.  终端汇聚  collect/last → subscribe          展开子 Vec,消费整条流
```

---

## 5. 完整管线模式 (sasspile v7.0 — Shared 多线程)

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
        .scan_map(CssBuilder::new(), |b: &mut CssBuilder, line: String| -> Vec<CssNode> {
            b.feed(&line)  // &借用，零 clone
        })
        .flat_map(|v: Vec<CssNode>| Shared::from_iter(v))
        .collect::<Vec<CssNode>>().last()
        .map(|nodes| post_process(nodes))
        .flat_map(|nodes| Shared::from_iter(nodes))
        // Phase 3: 借用渲染 (&CssNode → String)
        .map(|node: CssNode| render_node(&node))
        .collect::<Vec<String>>().last()
        .subscribe(move |css_vec: Vec<String>| {
            let _ = tx.send(css_vec.join("\n"));
        });

    input.lines().for_each(|line| subject.clone().next(line.to_string()));
    subject.clone().complete();
    rx.recv().unwrap_or_default()
}
```

---

## 6. 绝对禁止 (GC 思维)

| 反模式 | 为什么违反 Rust 所有权 | 正确做法 |
|---|---|---|
| `Rc<RefCell<T>>` + `borrow_mut()` | 模拟 GC 共享可变 | `scan_map(State::new(), reducer)` |
| `let mut v = Vec::new(); for x in items { v.push(x) }` | 命令式累积 | `items.flat_map(f).collect()` |
| `obs.subscribe(\|x\| shared.lock().push(x))` | 共享可变跨线程 | `obs.last().subscribe(\|r\| ...)` |
| `Arc<Mutex<String>>` + `lock().push_str()` | GC 共享模式 | `mpsc channel` + `tx.send()` |
| 用 Flux 思维设计然后"翻译"成 rxrust | 绕了弯路 | **直接想: 这数据是 move/&/?** |
| 猜测 rxrust API | 幻觉方法 | **先读源码, 再写代码** |

---

## 7. Ownership vs GC 思维对照 (参考)

| GC 思维 (禁止) | Rust 所有权 (正确) |
|---|---|
| `Arc<Mutex<T>>` 共享可变 | `mpsc channel` 单次值转移 |
| `Rc<RefCell<T>>` + `borrow_mut()` | `scan_map(State::new(), reducer)` |
| `let mut v = vec![]; for x in items { v.push(x) }` | `items.flat_map(f).collect()` |
| `obs.subscribe(\|x\| cell.borrow_mut().push(x))` | `obs.last().subscribe(\|r\| ...)` |

---

## 8. 调试 span 约定

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

## 9. 完整反应式指南 (10 章 + 源码定位)

详见 **`docs/rxrust-reactive-guide.md`** (基于 rxrust 1.0.0-rc.5 源码全量慢读).

**核心洞察** (v7.0):
1. **Observer `error(self)` / `complete(self)` 的 move 语义** 是类型层面的终结约束
2. **scan_map 的 acc 放在 Observer 内部** 实现 per-subscription 状态隔离
3. **flat_map = Map + MergeAll** 组合, 用 `concurrent` 参数控制并发 vs 顺序
4. **from_iter 内 for 循环顶必须加 `if observer.is_closed() { break; }`**
5. **Subject re-entrant next 会 panic** — 用 `delay(0)` 创建 async boundary
6. **Shared = `Arc<Mutex<Subscribers>>` 多线程广播; Local = `Rc<RefCell<Subscribers>>` 单线程**

---

## 10. 提交前自检

- [ ] **用 Ownership 三态 (move/&/&) 思考数据流** — 不强行翻译 Flux
- [ ] **先读 rxrust 源码** — 不猜 API, 不造轮子
- [ ] **0 个 `Rc<RefCell>` / `Arc<Mutex>`** — 状态由 scan_map 管理
- [ ] **0 个命令式 `for + push`** — 用 flat_map + collect
- [ ] **render_node 借用 `&T`** — 不 clone
- [ ] **终端用 mpsc channel** — 不用 Arc<Mutex>
- [ ] **endpoint 仅用 collect/last + subscribe**
- [ ] **单文件 ≤ 500 行**
- [ ] **Shared 入口用 String** — 不试图用 &str 绕过 'static

---

## 11. 验收标准

输出格式:
```
Pattern: <scan_map | flat_map | collect | mpsc | map | filter | tap>
Ownership: <move | 借引用 & | 排他修改 &mut>
Before: <违反所有权的写法>
After: <正确写法>
SourceRef: <来自 rxrust src 哪一行>
```
