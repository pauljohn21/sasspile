---
description: sasspile 项目 AI 编码强制规则（Rust + rxrust 编译器）
---

# sasspile 项目 AI 编码指令

> 本文件是 CatPaw/Claude 在本项目中的**系统级行为约束**。每次会话启动自动加载。
> 四大支柱：借引用 · 消费自身 · 响应式思想 · 终点收集

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

## 🔨 rxrust 四大支柱（管线代码核心规则）

### 支柱 1：借引用（&T / &mut Acc）

scan_map 闭包内用 `&mut Acc` 就地修改，渲染函数用 `&T` 借用。

```rust
// ✅ 借引用：scan_map 内部 &mut 就地改，render 借用 &CssNode
.scan_map(CssBuilder::new(), |builder: &mut CssBuilder, line: String| -> Vec<CssNode> {
    builder.feed(&line)  // &line 借用传参
})
.map(|node: CssNode| render_node(&node))  // render_node 借 &CssNode 产生 String
```

### 支柱 2：消费自身（&mut Acc）

状态演化算子 `scan_map` 用 `&mut Acc` 就地修改，零外部共享可变。

```rust
// ✅ 消费自身：&mut 就地改，算子内部持有状态
stream.scan_map(CompileState::default(), |state: &mut _, token: String| {
    state.line_count += 1;      // 就地修改
    state.dispatch(token)       // 返回 Vec<String>
});

// ❌ 外部可变或被多个 lambda 共享
let mut state = CompileState::default();
stream.map(|x| { state.update(x); ... })  // borrow 冲突
```

### 支柱 3：响应式思想（chain 是声明，subscribe 是执行边界）

```rust
// chain 不执行，只是声明数据关系
let pipeline = source.map(|x| x * 2).filter(|x| x > 10);

// .subscribe() 才真正开始执行
pipeline.subscribe(|x| println!("{}", x));
```

### 支柱 4：终点收集（→ String）

中间步骤尽量借用，最终结果必须是 `String` / `Vec<T>`。

```rust
source
    .scan_map(CompileState::default(), dispatch)     // &mut 就地修改
    .flat_map(|v: Vec<String>| Shared::from_iter(v))  // 1:N 展平
    .scan_map(CssBuilder::default(), build)           // &mut 就地修改
    .flat_map(|v: Vec<CssNode>| Shared::from_iter(v)) // 1:N 展平
    .map(|node: CssNode| render_node(&node))          // 借用渲染（首次 owned String）
    .collect::<Vec<String>>()                         // 汇聚
    .last()                                           // 取终态
    .subscribe(|v: Vec<String>| v.join("\n"))         // 终点产物：String
```

---

## ⛔ 反应式编码禁令（见了就改）

| # | 反模式 | 正确做法 |
|---|--------|---------|
| 1 | `for item in stream { state.push(x) }` | `scan_map` / `flat_map` + `collect` |
| 2 | `Rc<RefCell<T>>` + `borrow_mut()` | scan_map 的 `&mut Acc` 就地修改 |
| 3 | `Arc<Mutex<T>>` + `lock()` | `std::sync::mpsc::channel` 单次值转移 |
| 4 | subscribe 闭包内再 subscribe | 改 `flat_map` + 子流 |
| 5 | `.flat_map(\|x\| vec![x])`（非 Observable） | `.flat_map(\|x\| Shared::from_iter(vec![x]))` |
| 6 | 猜测 rxrust API | **先读源码** `~/.cargo/registry/src/.../rxrust-1.0.0-rc.5/` |
| 7 | `from_iter` 循环无 is_closed 检查 | 循环顶加 `if observer.is_closed() { break; }` |
| 8 | Shared 管线用 `&str` 试图绕过 'static | 接受 Shared 需要 `String`，保证中间零 clone |

---

## ✅ 算子速查（sasspile 用 Shared 多线程上下文）

| 算子 | 签名 | 用途 |
|------|------|------|
| `map` | `FnMut(Item) -> Out` | 1:1 类型转换 |
| `filter` | `FnMut(&Item) -> bool` | 过滤，不改变类型 |
| `scan_map` | `FnMut(&mut Acc, Item) -> Output` | 状态机：就地累积 + 每步发射 |
| `flat_map` | `FnMut(Item) -> Inner`（Inner: Observable） | 1:N 展平 |
| `collect<C>` | `C: Extend<Item>` | 汇聚为集合，complete 时发射 |
| `last` | — | 只发射最后一次产生值 |
| `take(n)` | — | 取前 n 个，后续自动关闭 |
| `subscribe` | `FnMut(Item)` | **终端**，从这里开始真正消费数据 |

---

## ✅ 完整管线模式（实际编译验证模板）

```rust
pub fn compile_pipeline(input: &str) -> String {
    // Shared Subject 入口 (String 因 Shared 需要 'static + Send)
    let subject = Shared::subject::<String, Infallible>();
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    subject.clone()
        // Phase 1: CompileState 消费自身 (&mut 就地修改)
        .scan_map(CompileState::new(), dispatch_pass)
        // flat_map: Vec<String> → 逐个 String
        .flat_map(|v: Vec<String>| Shared::from_iter(v))
        // Phase 2: CssBuilder 消费自身 (&mut 就地修改)
        .scan_map(CssBuilder::new(), |builder: &mut CssBuilder, line: String| -> Vec<CssNode> {
            builder.feed(&line)
        })
        // flat_map: Vec<CssNode> → 逐个 CssNode
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

---

## 🔬 Shared vs Local 选择原则

**sasspile 用 Shared**，原因：
1. `Shared` = `Arc<Mutex<Subscribers>>` — 多线程广播 + 同步求值
2. `Local` = `Rc<RefCell<Subscribers>>` — 单线程，无法跨线程传递
3. Shared 需要 'static + Send → 入口必须 `String`
4. 管线流程实际是同步的（push all → complete → terminal 已全部执行）

Shared 多线程流程（基于源码）：
```
Subject<Arc<Mutex<Subscribers>>> → next() 加锁广播 → ScanMapObserver(&mut State)
→ flat_map(MergeAll) → 子流 FromIter 同步迭代 → collect → last → subscribe terminal
```

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
| 算子签名 / 返回类型 | `src/observable.rs` |
| Observer trait | `src/observer.rs` |
| scan_map 实现 | `src/ops/scan_map.rs` |
| flat_map 实现 | `src/ops/flat_map.rs` |
| FromIter | `src/observable/from_iter.rs` |
| Subject | `src/subject/` |
| Shared vs Local | `src/rc.rs` (MutRc / MutArc 对比) |
| Context 子系统 | `src/context.rs` |

---

## 📦 项目技术栈

- Rust edition 2024, toolchain 1.97
- rxrust 1.0.0-rc.5 反应式框架（sasspile 用 Shared 多线程上下文）
- SCSS 编译器项目（sasspile）
- 详细指南：`docs/rxrust-reactive-guide.md`（20 章，AI 零基础执行版）
