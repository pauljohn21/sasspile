# rxrust-scss Skill

## 元认知

**使用 rxrust 内置功能,绝不手工造轮子。先读源码,再写代码。**

sasspile 当前管线模式: **Shared Subject + mpsc channel 终端** (非 from_stream)。
本文档补充 rxrust 算子级别的开发辅助规则。

---

## 第零步: 读源码

写代码前, 先读 rxrust 源码:

```
~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rxrust-1.0.0-rc.5/src/
```

| 问题 | 读哪里 |
|------|--------|
| 算子签名 | `src/observable.rs` |
| scan_map 实现 (Acc 隔离) | `src/ops/scan_map.rs` |
| flat_map = MergeAll | `src/ops/flat_map.rs` |
| collect (汇聚) | `src/ops/collect.rs` |
| Observer trait (move 终结) | `src/observer.rs` |
| re-entrant / broadcast | `src/subject/subject_core.rs` |

---

## 状态管理: scan_map 算子即状态机

```rust
// ✅ 正确: scan_map 内部 &mut 就地修改, 状态在算子内
.scan_map(CompileState::new(), dispatch_pass)
.scan_map(CssBuilder::new(), |builder, line| builder.feed(&line))

// ❌ 错误: 外部 Arc<Mutex> 共享状态
```

---

## 聚合结果

### 模式 A: Shared Subject + mpsc (svg 当前生产管线)

```rust
let subject = Shared::subject::<String, Infallible>();
let (tx, rx) = std::sync::mpsc::channel::<String>();

subject.clone()
    .scan_map(CompileState::new(), dispatch_pass)
    .flat_map(|v: Vec<String>| Shared::from_iter(v))
    ...
    .collect::<Vec<String>>()
    .last()
    .subscribe(move |v: Vec<String>| {
        let _ = tx.send(v.join("\n"));
    });

input.lines().for_each(|line| subject.clone().next(line.to_string()));
subject.clone().complete();
rx.recv().unwrap_or_default()
```

### 模式 B: from_stream + oneshot (异步流场景, sasspile 当前未用)

```rust
// 仅在真正需要异步生产时使用
Shared::from_stream(futures::stream::iter(items))
    ...
```

---

## 关键禁令

1. **禁止 Arc<Mutex> 共享状态** — 用 scan_map 算子
2. **禁止 Rc<RefCell>** — 用 scan_map 状态机
3. **禁止命令式 for+push** — 用 flat_map + collect
4. **禁止猜测 API** — 先读 rxrust 源码
5. **禁止 subscribe_boxed** — 不存在
6. **禁止 Flux→Rust "翻译"思维** — 直接用 Rust ownership 三态 (move/&/&) 思考

---

## 调试

如果遇到 `Send` 约束失败:
1. 检查是否用了 `Rc`/`RefCell` (改为 scan_map 状态)
2. 检查闭包是否捕获非 `'static` 借用 (改为 move)

如果遇到 `SourceWithDynamicSubs is not a Future`:
- collect/last 在 Shared 返回嵌套 Subscription, 管线是同步的, 不需要 block_on

---

## 内置算子源码模板

`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rxrust-1.0.0-rc.5/src/ops/`
- `filter.rs` — 最简单的算子模板
- `filter_map.rs` — 带状态转换的模板
- `scan_map.rs` — 带累积状态的模板
- `collect.rs` — 终结操作(收集)
- `last.rs` — 终结操作(最后值)
- `flat_map.rs` — 展开子流(使用 `MergeAll<Map<...>>`)
- `from_iter.rs` — 同步迭代器源
