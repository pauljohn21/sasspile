# rxrust-scss Skill

## 核心原则

**使用 rxrust 内置功能,绝不手工造轮子。**

## 数据源

```rust
// ✅ 正确: 多线程 Shared + from_stream
Shared::from_stream(futures::stream::iter(items))

// ❌ 错误: 单线程 Local + from_iter
Local::from_iter(items)

// ❌ 错误: 手写 mpsc channel
let (tx, rx) = std::sync::mpsc::channel();
```

## 聚合结果

```rust
// ✅ 正确: 内置算子链 + subscribe 返回 TaskHandle
let handle = source
    .collect::<String>()
    .last()
    .subscribe(|s| { /* ... */ });
handle.await;  // 或 runtime.block_on(handle);

// ❌ 错误: 手写 channel + Arc<Mutex> 聚合
```

## 自定义算子风格

遵循 rxrust 内置算子(如 `Filter`/`FilterMap`)的原风格:

```rust
#[derive(Clone)]
pub struct MixinOp<S> {
    pub source: S,
}

#[derive(Clone)]
pub struct MixinObserver<O> {
    observer: O,
    // 算子特有状态字段
}

impl<S> ObservableType for MixinOp<S> where S: ObservableType {
    type Item<'a> = S::Item<'a> where Self: 'a;
    type Err = S::Err;
}

impl<O> Observer<Item, Err> for MixinObserver<O> where O: Observer<Item, Err> {
    fn next(&mut self, value: Item) { /* 算子逻辑 */ }
    fn error(self, err: Err) { self.observer.error(err); }
    fn complete(self) { self.observer.complete(); }
    fn is_closed(&self) -> bool { self.observer.is_closed() }
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

## 关键禁令

1. **禁止 PhantomData** — 不需要 `PhantomData<fn() -> T>` 或 `PhantomData<()>`,`#[derive(Clone)]` 自动处理
2. **禁止手动 Clone impl** — 用 `#[derive(Clone)]`
3. **禁止手写 mpsc/channel** — 用 `collect().last()` + `subscribe()`
4. **禁止手写 OnceLock<Runtime>** — `subscribe()` 返回 `TaskHandle` 直接 `.await`
5. **禁止 `box_it()`** — 内置算子链在 `Shared` 下自动处理类型擦除
6. **禁止 `Local::from_iter`** — 多线程项目用 `Shared::from_stream`

## 多线程调度

`Shared::from_stream()` 内部使用 `SharedScheduler`,自动将任务分发到 tokio 线程池。不需要:
- 手写 tokio runtime
- 手动 `observe_on(SharedScheduler)`
- 额外的线程池配置

## 调试

如果遇到 `Send` 约束失败:
1. 检查是否用了 `PhantomData<fn() -> T>` (改为 `#[derive(Clone)]`)
2. 检查是否用了 `Rc`/`RefCell` (改为 `Arc`/`Mutex`)
3. 检查 observer 是否实现了 `Clone`

## 参考

### 完整架构指南
- **文档路径**: `docs/rxrust_architecture_guide.md`
- **远程记忆**: 知识库 `rxrust-architecture` (ID: `base6ca99fcb-622e-4cd8-ac53-3521783d0bfe`)
- **覆盖内容**: 核心 Trait、Context 系统、算子四部曲、调度器、Subscription、管道示例、设计规则

### 内置算子源码模板
`~/.cargo/registry/src/index.crates.io-*/rxrust-1.0.0-rc.5/src/ops/`
- `filter.rs` — 最简单的算子模板
- `filter_map.rs` — 带状态转换的模板
- `scan_map.rs` — 带累积状态的模板
- `collect.rs` — 终结操作(收集)
- `last.rs` — 终结操作(最后值)
- `flat_map.rs` — 展开子流(使用 `MergeAll<Map<...>>`)
- `from_stream.rs` — 流式数据源(内部已有 loop)
- `from_iter.rs` — 同步迭代器源
