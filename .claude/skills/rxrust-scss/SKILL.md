# rxrust-scss Skill

## 核心原则

**使用 rxrust 内置功能,绝不手工造轮子。先读源码,再写代码。**

---

## 第零步: 读源码

写代码前, 先读 rxrust 源码:

```
~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rxrust-1.0.0-rc.5/src/
```

| 问题 | 读哪里 |
|------|--------|
| 算子签名 | `src/observable.rs` |
| 创建 Observable | `src/factory.rs` |
| 订阅返回 | `src/observable.rs` 141 行 |
| Subscription 嵌套 | `src/subscription/source_with_dynamic.rs` |

---

## 数据源

```rust
// ✅ 正确: 多线程 Shared + from_stream (跨线程需要 Send+'static)
Shared::from_stream(futures::stream::iter(items))

// ✅ 正确: flat_map 内有序组合
.flat_map(|v| Shared::from_stream(futures::stream::iter(v)))

// ❌ 错误: 手写 mpsc channel
let (tx, rx) = std::sync::mpsc::channel();
```

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

```rust
// ✅ 正确: collect/last + oneshot + subscribe
let (tx, rx) = tokio::sync::oneshot::channel::<String>();
let mut tx_opt = Some(tx);

let handle = source
    .collect::<Vec<String>>()
    .last()
    .subscribe(move |css_vec| {
        if let Some(tx) = tx_opt.take() {
            let _ = tx.send(css_vec.join("\n"));
        }
    });

// 驱动 Shared 管线 (collect/last 返回嵌套 Subscription)
tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(handle.source.source);
});

// 消费结果
rx.blocking_recv().unwrap_or_default()
```

---

## 自定义算子风格

遵循 rxrust 内置算子(如 `Filter`/`FilterMap`)的原风格:

```rust
#[derive(Clone)]
pub struct MixinOp<S> { pub source: S }

#[derive(Clone)]
pub struct MixinObserver<O> { observer: O }

impl<S> ObservableType for MixinOp<S> where S: ObservableType {
    type Item<'a> = S::Item<'a> where Self: 'a;
    type Err = S::Err;
}

impl<O, Item, Err> Observer<Item, Err> for MixinObserver<O> where O: Observer<Item, Err> {
    fn next(&mut self, value: Item) { /* 算子逻辑 */ self.observer.next(value); }
    fn error(self, err: Err) { self.observer.error(err); }
    fn complete(self) { self.observer.complete(); }
    fn is_closed(&self) -> bool { self.observer.is_closed(); }
}

impl<S, C> CoreObservable<C> for MixinOp<S>
where C: Context, S: CoreObservable<C::With<MixinObserver<C::Inner>>> {
    type Unsub = S::Unsub;
    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped = context.transform(|observer| MixinObserver { observer });
        self.source.subscribe(wrapped)
    }
}
```

---

## 关键禁令

1. **禁止 Arc<Mutex> 共享状态** — 用 scan_map 算子 + oneshot 终端
2. **禁止 Rc<RefCell>** — 用 scan_map 状态机
3. **禁止手写 mpsc/channel** — 用 oneshot + collect/last
4. **禁止手写 OnceLock<Runtime>** — Shared 内建 tokio runtime
5. **禁止命令式 for+push** — 用 flat_map + collect
6. **禁止猜测 API** — 先读 rxrust 源码
7. **禁止 subscribe_boxed** — 这个方法不存在

---

## 多线程调度

`Shared::from_stream()` 内部使用 `SharedScheduler`,自动将任务分发到 tokio 线程池 (2 workers)。不需要:
- 手写 tokio runtime
- 手动 `observe_on(SharedScheduler)`
- 额外的线程池配置

---

## 调试

如果遇到 `Send` 约束失败:
1. 检查是否用了 `Rc`/`RefCell` (改为 scan_map 状态)
2. 检查 observer 是否实现了 `Clone`
3. 检查闭包是否捕获了非 `'static` 借用 (改为 move + oneshot)

如果遇到 `SourceWithDynamicSubs is not a Future`:
- collect/last 在 Shared 返回嵌套 Subscription, 需要 `.source.source` 深入拿 TaskHandle

---

## 内置算子源码模板

`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rxrust-1.0.0-rc.5/src/ops/`
- `filter.rs` — 最简单的算子模板
- `filter_map.rs` — 带状态转换的模板
- `scan_map.rs` — 带累积状态的模板
- `collect.rs` — 终结操作(收集)
- `last.rs` — 终结操作(最后值)
- `flat_map.rs` — 展开子流(使用 `MergeAll<Map<...>>`)
- `from_stream.rs` — 流式数据源(内部已有 loop)
- `from_iter.rs` — 同步迭代器源
