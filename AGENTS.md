---
description: sasspile 项目 AI 编码强制规则（Rust + rxrust 编译器）
---

# sasspile 项目 AI 编码指令

> 本文件是 CatPaw/Claude 在本项目中的**系统级行为约束**。每次会话启动自动加载。
> 核心原则：**Rust ownership = 响应式解答**。不是模仿 Flux，是所有权模型天然解决响应式问题。

---

## ⛔ 项目强制规则（违反 = 任务失败）

1. **禁止 Python**：不用 python3/pip；脚本用 `rust-script`；表达式用 `rust-script -e`；测试用 `#[test]`
2. **禁止 println!/eprintln!**：所有代码一律用 `tracing` 宏（info!/warn!/error!/debug!）
3. **必须使用 tracing span**：跨函数/跨阶段管道处理必须用 `tracing::span!`，字段命名遵循 `stage`, `module`, `id`, `elapsed_ms`
4. **禁止内联测试**：src/ 保持纯生产代码，所有测试放在 `tests/` 目录
5. **禁止 `unwrap()`/`expect()`** 在 `src/` 生产代码中：用 `?` 或 `Option` / `Result`；仅 `tests/` 允许 `expect("原因")`
6. **单文件 ≤ 500 行**：源码和测试分别计算，超出必须拆分
7. **使用 SSH 方式推送 GitHub**：`git push github main`
8. **只提交不推送**：commit 后必须等用户确认
9. **禁止低效手动工具**：grep、sed、逐行命令行分析、重复打印查询、bash 循环

---

## 🔨 响应式支柱：Rust Ownership 三态

### 元认知：为什么不用 Flux 思维

```
Reactor/Flux 的核心假设：
  一切数据被 GC 管理，任何 lambda 能自由捕获/共享/修改任意对象
  ⇒ 运行时混沌 (CME/数据竞争/隐式反馈循环)

Rust 的解答：
  闭包捕获是三选一 (move/&/&) — 编译期 X 光
  ⇒ 零运行时灾难 (&mut Acc = 零锁零 clone 的状态机)
```

**不是 rxrust 模仿 Flux，是 Rust ownership 天然实现了 Flux 想做的事。**

### 支柱 1：move — 所有权转移 (Terminal 闭包)

```rust
// subscribe 闭包：move 转移终态
.subscribe(move |v: Vec<String>| {
    let _ = tx.send(v.join("\n"));  // tx 被 move, 此后外部不可用
});
```

### 支柱 2：& — 借引用 (Transform 借用)

```rust
// 渲染函数借 &T 产生 String, 不需要 clone
.map(|node: CssNode| render_node(&node))  // &node 借用, 零 clone

// filter 借用判断
.filter(|x: &Token| x.is_valid())
```

### 支柱 3：&mut Acc — 消费自身 (scan_map 独有)

```rust
// scan_map 是唯一持有状态的地方 — 零外部共享可变
stream.scan_map(CompileState::new(), |state: &mut CompileState, line: String| -> Vec<String> {
    state.line_count += 1;      // &mut 排他修改
    state.dispatch(token)       // 返回 Vec<String>
});

// ❌ 反模式: 外部可变或被多个 lambda 共享
let mut state = CompileState::new();
stream.map(|x| { state.update(x); ... })  // borrow 冲突
```

### 支柱 4：响应式思想（chain 是声明，subscribe 是执行边界）

```rust
// chain 不执行，只是声明数据关系
let pipeline = source.map(|x| x * 2).filter(|x| x > 10);

// .subscribe() 才真正开始执行
pipeline.subscribe(|x| { ... });
```

---

## ⛔ 反应式编码禁令（见了就改）

| # | 反模式 (GC 思维) | 正确做法 (Ownership 思维) |
|---|-----------------|-------------------------|
| 1 | `for item in stream { state.push(x) }` | `scan_map` / `flat_map` + `collect` |
| 2 | `Rc<RefCell<T>>` + `borrow_mut()` | scan_map 的 `&mut Acc` 就地修改 |
| 3 | `Arc<Mutex<T>>` + `lock()` | `std::sync::mpsc::channel` 单次值转移 |
| 4 | subscribe 闭包内再 subscribe/触发 source | 改 `flat_map` + 子流 或 `delay(0)` |
| 5 | `.flat_map(\|x\| vec![x])`（非 Observable） | `.flat_map(\|x\| Shared::from_iter(vec![x]))` |
| 6 | 猜测 rxrust API | **先读源码** `~/.cargo/registry/src/.../rxrust-1.0.0-rc.5/` |
| 7 | `from_iter` 循环无 is_closed 检查 | 循环顶加 `if observer.is_closed() { break; }` |
| 8 | Shared 管线用 `&str` 试图绕过 'static | 接受 Shared 需要 `String` |
| 9 | 用 Flux 思维设计然后"翻译"成 rxrust | 直接用 Rust ownership 三态思考数据流 |

---

## ✅ 算子速查（Shared 多线程上下文）

| 算子 | 闭包签名 | 所有权语义 |
|------|---------|----------|
| `map` | `FnMut(T) -> U` | 消费 T, 产出 U (所有权转移) |
| `filter` | `FnMut(&T) -> bool` | 借用判断, T 继续传给下游 |
| `scan_map` | `FnMut(&mut Acc, T) -> Out` | Acc 就地修改 (零 clone 零锁) |
| `flat_map` | `FnMut(T) -> Inner: Observable` | T 消费, Inner 子流展平 |
| `tap` | `FnMut(&T)` | 副作用不改流 (借用) |
| `collect<C>` | `C: Extend<T>` | 汇聚,**complete 时发射** |
| `last` | — | 只发射最后一次产生 |
| `take(n)` | — | 取前 n 个,**自动关闭** |
| `finalize` | `FnOnce()` | 终止清理 (complete/error/unsubs) |
| `on_complete` | `FnOnce()` | complete 回调 |

---

## ✅ 完整管线模式（sasspile 编译验证模板）

```rust
pub fn compile_pipeline(input: &str) -> String {
    use rxrust::prelude::*;
    use std::convert::Infallible;

    // Shared Subject 入口 (String 因 Shared 需要 'static + Send)
    let subject = Shared::subject::<String, Infallible>();
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    subject.clone()
        // Phase 1: CompileState 消费自身 (&mut 就地修改)
        .scan_map(CompileState::new(), dispatch_pass)
        .flat_map(|v: Vec<String>| Shared::from_iter(v))
        // Phase 2: CssBuilder 消费自身 (&mut 就地修改)
        .scan_map(CssBuilder::new(), |builder: &mut CssBuilder, line: String| -> Vec<CssNode> {
            builder.feed(&line)  // &借用 line
        })
        .flat_map(|v: Vec<CssNode>| Shared::from_iter(v))
        .collect::<Vec<CssNode>>().last()
        // 汇聚后处理 (@media 合并等)
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

---

## 🔬 Shared vs Local 选择原则

**sasspile 用 Shared**，原因：
1. `Shared` = `Arc<Mutex<Subscribers>>` — 多线程广播 + 同步求值
2. `Local` = `Rc<RefCell<Subscribers>>` — 单线程，无法跨线程传递
3. Shared 需要 `'static + Send` → 入口必须 `String`
4. 管线流程实际是同步的（push all → complete → terminal 已全部执行）

---

## 🔬 调试协议（强制）

**核心原则**：禁止凭直觉猜测根因。所有 bug 修复必须基于 tracing trace 证据链。

### 4 步流程（不可跳过）

1. **SPAN 插桩**：疑似路径每个入口/出口加 span
2. **TRACE 采集**：`RUST_LOG=trace cargo test -- --nocapture` 收集证据
3. **根因定位**：必须引用具体 span + 字段值
4. **修复验证**：修复后清理临时 span

---

## 🔍 代码查询工具链

必须使用已建立的高效工具，禁止低效手动搜索：
1. `sass_spec_stats.rs` 生成 MD 报告 + 基线对比
2. `css_diag` / `expr_diag` / `cfs_diag` / `diag_directives` 定位失败
3. `codegraph callers/impact/node/explore` 代码查询
4. `RUST_LOG=trace --features otel` 链路追踪
5. `rust-script` 或 `Rust test` 处理数据/脚本

---

## 📦 rxrust 源码位置

路径：`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rxrust-1.0.0-rc.5/`

| 问题 | 读哪里 |
|------|--------|
| Observer trait (终结语义：error/complete 是 move) | `src/observer.rs` |
| Subject re-entrant panic / broadcast | `src/subject/subject_core.rs` |
| Connectable (publish/connect 等价) | `src/observable/connectable.rs` |
| scan_map (Acc 隔离实现) | `src/ops/scan_map.rs` |
| flat_map = MergeAll + concurrent 限流 | `src/ops/flat_map.rs`, `src/ops/merge_all.rs` |
| group_by + GroupedObservable | `src/ops/group_by.rs` |
| collect (Extend trait 汇聚) | `src/ops/collect.rs` |
| from_iter + is_closed 守卫 | `src/observable/from_iter.rs` |
| delay(0) 异步边界机制 | `src/ops/delay.rs` |
| finalize (终止保证一次) | `src/ops/finalize.rs` |
| tap (副作用不改流) | `src/ops/tap.rs` |
| distinct (去重 HashSet) | `src/ops/distinct.rs` |
| Local vs Shared (`MutRc`/`MutArc`) | `src/rc.rs` |
| Context 子系统 scheduler 集成 | `src/context.rs` |

---

## 📦 项目技术栈

- Rust edition 2024, toolchain 1.97
- rxrust 1.0.0-rc.5 响应式框架（sasspile 用 Shared 多线程上下文）
- SCSS 编译器项目（sasspile）
- 详细指南：`docs/rxrust-reactive-guide.md`

---

## 📋 提交前自查

- [ ] 所有权思维：move/&/&mut 是编译期决定, 不是运行时
- [ ] 无 `unwrap()`/`expect()` 在 src/ 中
- [ ] 无 `println!`/`eprintln!`
- [ ] tracing span 已插入跨函数路径
- [ ] 单文件 ≤ 500 行
- [ ] flat_map 返回 `Shared::from_iter`（非 Iterator）
- [ ] 入口 Subject 为 `Shared::subject::<String, Infallible>()`
- [ ] scan_map 的 Acc 是状态唯一栖息地
- [ ] `complete()` 在全部 `next()` 之后调用
- [ ] 不强行翻译 Flux，用 Rust 闭包所有权思考
- [ ] 临时调试 span 已清理
