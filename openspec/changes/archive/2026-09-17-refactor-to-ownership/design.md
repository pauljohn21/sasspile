## Context

项目中所有业务逻辑函数违反 Rust 所有权模式,退回到 GC 思维:

| 文件 | 模式 | 出现次数 |
|---|---|---|
| `lib.rs` | `Rc<RefCell<Vec>>` + `subscribe(\|x\| push)` | 2 |
| `shared/context.rs` | `Rc<RefCell<HashMap>>` | 4 |
| `evaluate_dst/mod.rs` | `while i < chars_vec.len()` | 7 |
| `evaluate_dst/mod.rs` | `borrow_mut().insert` | 2 |
| `evaluate_dst/builtins.rs` | `while/for` + `Vec::push` | 5 |
| `evaluate_dst/builtins.rs` | `for ch in inner.chars()` | 1 |
| `evaluate_dst/eval_ctx.rs` | `for x in items { out.extend(f(x)?) }` | 5 |
| `evaluate_dst/eval_ctx.rs` | `borrow_mut().insert` | 3 |
| `parse_dst/declarations.rs` | `while i < tokens.len()` | 14 |
| `parse_dst/declarations.rs` | `for x in items { out.push(f(x)) }` | 3 |
| `parse_dst/directives.rs` | `while pos < tokens.len()` | 3 |
| `parse_dst/mod.rs` | `while i < tokens.len()` | 3 |
| `shared/module_cache.rs` | `Rc<RefCell<>>` | 1 |

---

## Decisions

### Decision 1: 函数内部不用 rxrust Observable,用原语迭代器

**原理**: rxrust 是 pipeline 阶段的组合子。pipeline 内部的纯函数 helper 应该用 Rust 原语迭代器（`scan`, `map`, `fold`, `collect`）,而不是每个 helper 都构造一个 `Local::from_iter` Observable。

```rust
// ✓ 正确: helper 内部用原语迭代器
fn split_args(s: &str) -> Vec<String> {
    s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

// 复杂嵌套跟踪用 scan
fn split_args_nested(s: &str) -> Vec<String> {
    s.chars()
        .scan(DepthTracker::new(), |st, ch| Some(st.feed(ch)))
        .filter_map(|opt| opt)
        .collect()
}
```

### Decision 2: 累积状态用 scan_map,终点用 last

**原理**: rxrust 的 `scan_map` 语义就是 reducer——消费旧 acc,返回新 acc。

```rust
// 状态传播
chars
    .scan_map(SubstState::new(), |state, ch| state.feed(ch))
    .flat_map(|results| Local::from_iter(results))
    .last()
    .subscribe(|final_result| { /* use final result */ })
```

### Decision 3: 副作用隔离到 .tap()

**原理**: reducer 闭包应该纯净（输入→输出,无副作用）。
```rust
// � 错误
nodes.flat_map(|n| { tracing!(?n); eval(n) })

// ✓ 正确
nodes
    .tap(|n| tracing::debug!(?n, stage = "eval"))
    .flat_map(eval_node)
```

### Decision 4: 跨节点展开用 flat_map

**原理**: 1 节点 → N CssNode 是 flat_map 的本职,不是 `for + extend`。
```rust
// 在 pipe 内部 match 分支返回 Observable 而不是 Vec
Node::For { var, from, to, body } => {
    expand_for_loop(var, from, to, body).into_local_obs().boxed()
}
```

### Decision 5: CompilerContext 的 Rc<RefCell> 保留查找路径,消除修改路径

**原理**:
- 模块缓存 / 全局变量**查找**是真正的共享只读语义 → `borrow()` OK
- 变量注册 / mixin 注册 / scope push-pop 是**写操作** → 不应该通过 `borrow_mut`
- 写操作应该由 reducer 内部通过返回新状态 / 调用 ctx 的不可变 API 来达成

渐进方案: 先把明显的 `for + extend` 替换成迭代器风格,`borrow_mut` 暂不动（整体架构大改属于后续 P2 的独立 change）。

---

## Migration Plan

Phase 1 (本 change 范围):
1. lib.rs 收集法 → `.last().subscribe()`
2. 所有 helper 函数内 `while/for + Vec::push` → 迭代器链
3. 副作用隔离到 `.tap()`

Phase 2 (后续 change):
- CompilerContext 状态管理 rxrust 化
- evaluate_for/if 完全 flat_map 化
- 测试订阅模式更新

---

## Verification

- `cargo test` 全量通过
- `cargo clippy -- -W clippy::all` 无新增 warning
- 统计 `grep -c "while.*<.*\.len"` / `borrow_mut` / `Rc.*RefCell` 在 src/ 中的数量,每个文件 ≤1（仅 CompilerContext 定义处）
