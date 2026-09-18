## Design: Custom Directive Operators (CDDOO)

### 架构总览 — 自定义算子取代通用 scan_map+State 模式

```
char stream (Local::from_iter)
   │
   ▼
[Built-in Ops]   filter_map / scan_map / flat_map  ← tokenize 阶段
   │
   ▼
[Custom Directive Ops]  .use_() .mixin() .include() .if_() .for_() .each()
   │                      ↑ 每个指令是独立算子文件,逻辑在 next() 方法
   ▼
[Built-in Ops]   map / collect::<String>() / last()  ← serialize + 收集者
   │
   ▼
subscribe → channel → String
```

### 自定义算子四件套 (以 Use 为例,见 `src/directive/use_.rs`)

```rust
// 1. 指令标记 (PhantomData)
pub struct Use;

// 2. 算子壳子
pub struct UseOp<S> {
    pub source: S,
    pub _instruction: PhantomData<fn() -> Use>,
}

// 3. Observer 包装 — 逻辑写在这里
pub struct UseObserver<O> {
    pub observer: O,
    pub _instruction: PhantomData<fn() -> Use>,
}
impl<O, Item, Err> Observer<Item, Err> for UseObserver<O>
where O: Observer<Item, Err> {
    fn next(&mut self, value: Item) {
        // TODO: @use 逻辑
        self.observer.next(value);
    }
    fn error(self, err: Err) { self.observer.error(err); }
    fn complete(self) { self.observer.complete(); }
    fn is_closed(&self) -> bool { self.observer.is_closed() }
}

// 4. CoreObservable — 订阅时包装下游 Observer
impl<S, C> CoreObservable<C> for UseOp<S>
where C: Context,
      S: CoreObservable<C::With<UseObserver<C::Inner>>> {
    type Unsub = S::Unsub;
    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped = context.transform(|observer| UseObserver {
            observer, _instruction: PhantomData
        });
        self.source.subscribe(wrapped)
    }
}
```

### DirectiveOps Trait (src/directive/mod.rs)

```rust
pub trait DirectiveOps: Observable
where Self::Inner: ObservableType {
    fn use_(self) -> Self::With<UseOp<Self::Inner>> { ... }
    fn mixin(self) -> Self::With<MixinOp<Self::Inner>> { ... }
    fn include(self) -> Self::With<IncludeOp<Self::Inner>> { ... }
    fn if_(self) -> Self::With<IfOp<Self::Inner>> { ... }
    fn for_(self) -> Self::With<ForOp<Self::Inner>> { ... }
    fn each(self) -> Self::With<EachOp<Self::Inner>> { ... }
}
impl<T> DirectiveOps for T where T: Observable, T::Inner: ObservableType {}
```

### 文件结构 (≤ 500 LOC/file)

```
src/
├── lib.rs                   (~115 LOC) — compile / compile_parallel + 测试
└── directive/
    ├── mod.rs               (~65 LOC)  — DirectiveOps trait + re-exports
    ├── use_.rs              (~66 LOC)  — @use 算子
    ├── mixin.rs             (~64 LOC)  — @mixin 算子
    ├── include.rs           (~64 LOC)  — @include 算子
    ├── if_.rs               (~64 LOC)  — @if 算子
    ├── for_.rs              (~64 LOC)  — @for 算子
    └── each.rs              (~64 LOC)  — @each 算子
```

### 所有权模型 — 零 clone / 零 Rc<RefCell>

- char 是 Copy — 按值移动,不是 clone
- scan_map reducer 用 `buf.drain(..)` 取出已累积的所有权,String 不复制
- 每个指令算子的 Observer 消费 self (error/complete 走所有权转移)
- DirectiveOps 链式传递:Observable::transform 把 inner 类型映射为 Self::With<Op>,不复制数据

### Non-Goals

- 不做 dart-sass 兼容
- 不做 sass-spec 100% pass 门控
- 不做多 crate 依赖
- 不做中间 box_it — 单链直到底部 collect
- 不做完整 Sass 模块系统 — 只覆盖 Element Plus / Bootstrap 子集
