# sasspile — Claude Code 项目级指令

本文档定义 Claude Code (CLI) 在本项目中的**强制行为规则**。每次会话启动自动加载。

## 项目概览

sasspile 是一个用 Rust 实现的纯函数式 SCSS 编译器，使用 rxrust 1.0.0-rc.5 响应式框架构建多线程编译管线。

**核心认知**：rxrust 不是模仿 Flux/Reactor，是 Rust 所有权模型（move/&/&mut）对响应式问题的自然解答。

---

## ⛔ 强制规则（违反 = 任务失败）

1. **禁止 Python**：不使用 python3/pip；脚本用 `rust-script`；表达式用 `rust-script -e`；测试用 `#[test]`
2. **禁止 println!/eprintln!**：所有代码（含 src/ 和 tests/）一律用 `tracing` 宏（info!/warn!/error!/debug!）
3. **必须使用 tracing span**：跨函数/跨阶段管道必须用 `tracing::span!`，字段命名：stage, module, id, elapsed_ms
4. **禁止内联测试**：src/ 保持纯生产代码，所有测试放在 `tests/` 目录
5. **禁止 `unwrap()`/`expect()`** 在 `src/` 生产代码中：用 `?` 或 `Option`/`Result`；仅 `tests/` 允许 `expect("原因")`
6. **单文件 ≤ 500 行**：源码和测试分别计算，超出必须拆分
7. **使用 SSH 方式推送 GitHub**：`git push github main`
8. **只提交不推送**：commit 后必须等用户确认
9. **禁止低效手动工具**：grep、sed、逐行命令行分析、重复打印查询、bash 循环

---

## 🔨 响应式支柱：Rust Ownership 三态

### 元认知

```
Flux 假设 GC 自由捕获 → 运行时混沌（CME/数据竞争/隐式反馈循环）
Rust 闭包是三选一   → 编译期 X 光（move/&/&mut）
```

**不是 rxrust 模仿 Flux，是 Rust ownership 天然实现 Flux 想做的事。**

| 支柱 | 语义 | 正确做法 |
|------|------|---------|
| **move** | 所有权转移 | subscribe 闭包 move tx 转移终态 |
| **&** | 不可变借用 | render 函数 `&T` 借用，零 clone |
| **&mut Acc** | 排他修改 | scan_map 唯一状态栖息地，零锁零 clone |

---

## ⛔ 反应式编码禁令

| ❌ 反模式 (GC 思维) | ✅ 正确做法 (Ownership 思维) |
|-------------------|-------------------------|
| `for item in stream { state.push(x) }` | `scan_map` / `flat_map` + `collect` |
| `Rc<RefCell<T>>` + `borrow_mut()` | scan_map 的 `&mut Acc` 就地修改 |
| `Arc<Mutex<T>>` + `lock()` | `std::sync::mpsc::channel` 单次值转移 |
| subscribe 闭包内再 subscribe/触发 source | 改 `flat_map` + 子流 或 `delay(0)` |
| `.flat_map(\|x\| vec![x])` | `.flat_map(\|x\| Shared::from_iter(vec![x]))` |
| 猜测 rxrust API | **先读源码** `~/.cargo/registry/src/.../rxrust-1.0.0-rc.5/` |
| `from_iter` 循环无 is_closed 检查 | 循环顶加 `if observer.is_closed() { break; }` |
| Shared 用 `&str` 绕过 'static | 接受 Shared 需要 `String` |
| 用 Flux 思维设计然后"翻译"成 rxrust | 直接用 Rust ownership 三态思考数据流 |

---

## ✅ 算子速查（Shared 多线程上下文）

| 算子 | 闭包签名 | 所有权语义 |
|------|---------|----------|
| `map` | `FnMut(T) -> U` | 消费 T, 产出 U |
| `filter` | `FnMut(&T) -> bool` | 借用判断, T 继续传 |
| `scan_map` | `FnMut(&mut Acc, T) -> Out` | Acc 就地修改 (零 clone 零锁) |
| `flat_map` | `FnMut(T) -> Inner: Observable` | T 消费, Inner 子流展平 |
| `tap` | `FnMut(&T)` | 副作用不改流 |
| `collect<C>` | `C: Extend<T>` | 汇聚, complete 时发射 |
| `last` | — | 只发射最后一次产生 |
| `finalize` | `FnOnce()` | 终止清理 (complete/error/unsubs) |

---

## ✅ sasspile 编译管线模式

```rust
// 入口：Shared 多线程上下文
let subject = Shared::subject::<String, Infallible>();
let (tx, rx) = std::sync::mpsc::channel::<String>();

subject.clone()
    // Phase 1: CompileState 消费自身 (&mut Acc)
    .scan_map(CompileState::new(), dispatch_pass)
    .flat_map(|v: Vec<String>| Shared::from_iter(v))
    // Phase 2: CssBuilder 消费自身 (&mut Acc)
    .scan_map(CssBuilder::new(), |b: &mut CssBuilder, line: String| -> Vec<CssNode> {
        b.feed(&line)  // &借用，零 clone
    })
    .flat_map(|v: Vec<CssNode>| Shared::from_iter(v))
    .collect::<Vec<CssNode>>().last()
    .map(|nodes| merge_media_nodes(nodes))
    .flat_map(|nodes| Shared::from_iter(nodes))
    // Phase 3: 借用渲染 (&T → String)
    .map(|node: CssNode| render_node(&node))
    .collect::<Vec<String>>().last()
    // subscribe = 执行边界, move 转移终态
    .subscribe(move |css_vec: Vec<String>| {
        let _ = tx.send(css_vec.join("\n"));
    });

input.lines().for_each(|line| subject.clone().next(line.to_string()));
subject.clone().complete();
rx.recv().unwrap_or_default()
```

---

## 🔬 Shared vs Local 选择

**sasspile 用 Shared**：
- `Shared` = `Arc<Mutex<Subscribers>>` — 多线程广播
- `Local` = `Rc<RefCell<Subscribers>>` — 单线程
- Shared 需要 `'static + Send` → 入口必须 `String`

---

## 🔬 调试协议（强制）

**禁止凭直觉猜测根因**。所有 bug 修复必须基于 tracing trace 证据链：

1. **SPAN 插桩**：疑似路径入口/出口加 `info_span!`
2. **TRACE 采集**：`RUST_LOG=trace cargo test -- --nocapture`
3. **根因定位**：必须引用具体 span + 字段值
4. **修复验证**：修复后清理临时 span

---

## 🔍 代码查询工具

1. `sass_spec_stats.rs` 生成 MD 报告 + 基线对比
2. `css_diag` / `expr_diag` / `cfs_diag` / `diag_directives` 定位失败
3. `codegraph callers/impact/node/explore` 代码查询
4. `RUST_LOG=trace --features otel` 链路追踪
5. `rust-script` 处理数据/脚本

---

## 📦 关键源码路径

rxrust: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rxrust-1.0.0-rc.5/`

| 问题 | 读哪里 |
|------|--------|
| Observer trait（终结语义：move）| `src/observer.rs` |
| Subject re-entrant / broadcast | `src/subject/subject_core.rs` |
| Connectable (publish/connect) | `src/observable/connectable.rs` |
| scan_map 实现（&mut Acc 隔离）| `src/ops/scan_map.rs` |
| flat_map = map + merge_all | `src/ops/flat_map.rs`, `src/ops/merge_all.rs` |
| group_by + GroupedObservable | `src/ops/group_by.rs` |
| collect (Extend trait) | `src/ops/collect.rs` |
| from_iter + is_closed 守卫 | `src/observable/from_iter.rs` |
| delay(0) 异步边界 | `src/ops/delay.rs` |
| finalize (终止保证一次) | `src/ops/finalize.rs` |
| tap (副作用不改流) | `src/ops/tap.rs` |
| Local vs Shared (`MutRc`/`MutArc`)| `src/rc.rs` |

---

## 📦 技术栈

- Rust edition 2024, toolchain 1.97
- rxrust 1.0.0-rc.5（Shared 多线程上下文）
- SCSS 编译器（sasspile）
- 详细指南：`docs/rxrust-reactive-guide.md`
- Skill：`.claude/skills/rxrust-ownership/SKILL.md`（v6.0）

---

## 📋 提交前自查

- [ ] 用 Rust ownership 三态 (move/&/&) 思考数据流，不强行翻译 Flux
- [ ] 无 `unwrap()`/`expect()` 在 src/ 中
- [ ] 无 `println!`/`eprintln!`
- [ ] tracing span 已插入跨函数路径
- [ ] 单文件 ≤ 500 行
- [ ] flat_map 返回 `Shared::from_iter`（非 Iterator）
- [ ] 入口 Subject 为 `Shared::subject::<String, Infallible>()`
- [ ] scan_map 的 Acc 是状态唯一栖息地
- [ ] `complete()` 在全部 `next()` 之后调用
- [ ] 临时调试 span 已清理
