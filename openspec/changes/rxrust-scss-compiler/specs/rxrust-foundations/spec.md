## ADDED Requirements

### Requirement: 编译器设计必须以 rxrust 类型为唯一抽象
编译器 MUST 使用 rxrust 定义的类型与算子作为 SCSS 编译过程的核心抽象，不得自行定义响应式原语或套壳包装 rxrust 语义。

#### Observable 类型体系
编译器 MUST 使用以下 rxrust 核心 trait：

| rxrust 抽象 | 编译器用途 | 核心方法 |
|-------------|-----------|----------|
| `Observable` | 管道 stage 出/入 | `.pipe()`, `.scan()`, `.flat_map()`, `.filter()` |
| `Observer<Item, Err>` | subscribe 端消费 | `next(&mut self)`, `error(self)`, `complete(self)` |
| `CoreObservable<O>` | stage 内部实现 | `subscribe(observer) -> Unsub` |
| `Subscription` | 取消订阅 / 生命周期管理 | `unsubscribe()` |

#### Scenario: stage 签名遵循 Observable trait 约定
- **WHEN** 定义 tokenize/parse/evaluate/serialize 四个 stage
- **THEN** MUST 实现为 `fn(input: LocalBoxedObservable<In, Infallible>) -> LocalBoxedObservable<Out, Infallible>`
- **AND** MUST 通过 `.pipe(stage)` 链式组合

#### Scenario: 不使用自定义 Publisher/Stream 包装
- **WHEN** 需要响应式原语（单值、流、共享流）
- **THEN** MUST 使用 rxrust 内置 `Local::from_iter()`, `Local::of()`, `Local::subject()` 等
- **AND** MUST 使用 `ops/` 下提供的算子而非自实现

### Requirement: 编译器必须遵循 rxrust 的两层算子架构
rxrust 算子遵循 **CoreObservable（纯逻辑）** + **Observable（用户 API）** 双层模型。编译器内部 stage 实现 MUST 与该架构对齐。

#### 算子结构模式
```
MapObserver<O, F>         ← Observer wrapper (内部)
  observer: O              ← 下游 Observer
  func: F                  ← 变换函数

Map<S, F>                ← 用户-facing struct
  source: S
  func: F

impl CoreObservable for Map  ← subscribe 入口，把 observer 包成 MapObserver
impl ObservableType for Map  ← 声明 Item/Err 类型
```

#### Scenario: tokenize stage 算子实现
- **WHEN** 实现 tokenize 算子
- **THEN** MUST 包含 `TokenizeObserver<O, State>` wrapper
- **AND** MUST 包含 `Tokenizer<S, State>` 用户-facing struct
- **AND** `Tokenizer::subscribe(context)` MUST 将 `TokenizeObserver` 注入 context

### Requirement: 管道必须使用 rxrust 的 `pipe` 组合语义
管道串联 MUST 使用 rxrust 的 `.pipe()` 方法或等价的 `.map/.scan/.flat_map`，不得使用循环迭代或回调嵌套。

#### Scenario: 主管道通过 pipe 组合
- **WHEN** 构建完整编译管线
- **THEN** MUST 类似：
  ```rust
  Local::<str, Infallible>::from_iter(src)
    .pipe(tokenize)
    .pipe(parse)
    .pipe(evaluate)
    .pipe(serialize)
    .last()
    .subscribe(|result| { ... })
  ```

#### Scenario: 不得使用命令式循环
- **WHEN** 需要遍历 Token 流构建 AST
- **THEN** MUST 使用 `.scan(initial, reducer)` 或 `.flat_map(|tokens| expand(tokens))`
- **AND** MUST NOT 使用 `for token in tokens { ... }` 或 `while let Some(token) = iter.next()`

### Requirement: 必须使用 rxrust 的全部算子类别
编译器实现 MUST 在适当位置使用以下 rxrust 算子类别，每个类别至少使用一种：

| 类别 | 算子 | 编译器用途 |
|------|------|-----------|
| **Creation** | `Local::from_iter()`, `Local::of()`, `Local::subject()` | 创建 char 流、单值流、共享流 |
| **Transformation** | `map`, `scan`, `flat_map`, `scan_map` | Token 变换、状态累积、嵌套展开 |
| **Filtering** | `filter`, `distinct`, `distinct_until_changed` | 去重 Token、过滤空白、消除相同连续选择器 |
| **Combination** | `merge`, `merge_all`, `zip`, `with_latest_from`, `combine_latest` | 多文件并行编译、sourcemap 合并 |
| **Utility** | `tap`, `finalize`, `delay` | 副作用 logging、资源清理 |
| **Conditional** | `take_while`, `skip_while`, `take_until` | 条件截断 |
| **Aggregation** | `reduce`, `last`, `collect`, `sum`, `average` | 收敛为单值 Result（最终 CSS 字符串） |
| **Connectable** | `multicast`, `publish`, `ref_count` | 多订阅者共享模块缓存 |

#### Scenario: 至少使用 map + scan + flat_map + filter + reduce + tap
- **WHEN** 实现编译管线
- **THEN** MUST 在不同 stage 分别使用上述算子
- **AND** 代码审查 MUST 验证每个类别的代表算子至少出现一次

### Requirement: 必须遵循 rxrust 的 Observer 包装模式
自定义 Observer（如 TokenizeObserver、ParseObserver）MUST 遵循 rxrust 的包装器模式：持有下游 observer + 实现变换逻辑。

#### 模式规定
```rust
// 1. 定义 Wrapper
pub struct XxxObserver<O, F> {
    observer: O,    // 下游
    state: F,       // 本 stage 的局部状态
}

// 2. 实现 Observer
impl<O, F, Item, Out, Err> Observer<Item, Err> for XxxObserver<O, F>
where
    O: Observer<Out, Err>,
    F: FnMut(Item) -> Out,
{
    fn next(&mut self, value: Item) {
        let output = (self.state)(value);
        self.observer.next(output);
    }
    fn error(self, err: Err) { self.observer.error(err); }
    fn complete(self) { self.observer.complete(); }
    fn is_closed(&self) -> bool { self.observer.is_closed() }
}
```

#### Scenario: ParseObserver 包装下游 evaluate
- **WHEN** parse 将 Token 转为 Node
- **THEN** MUST 定义 `ParseObserver<O, State>` 包装 downstream evaluate
- **AND** `next()` MUST 累积 Token 并产出聚合的 Node

### Requirement: 必须遵循 rxrust 的 Context/Scope 抽象
编译器 MUST 明确区分 `Local`（单线程）与 `Shared`（跨线程）scope，初期使用 `Local` 但预留 `Shared` 扩展能力。

#### Scenario: 当前使用 Local scope
- **WHEN** 编译单文件
- **THEN** MUST 使用 `Local::<Item, Err>::from_iter(...)` / `LocalBoxedObservable`

#### Scenario: 类型签名体现 Scope
- **WHEN** stage 函数签名
- **THEN** MUST 使用 `LocalBoxedObservable<'static, T, Infallible>` 而非 `SharedBoxedObservable`

#### Scenario: Shared 作为 Non-Goal 预留
- **WHEN** 未来需要并行编译
- **THEN** MUST 可切换为 `Shared` scope + `observe_on(scheduler)` + `subscribe_on(scheduler)`

### Requirement: 必须在 tap 中实现副作用，保持流纯
所有副作用（logging、metrics、cache 写入）MUST 通过 `tap` 算子注入，流变换本身 MUST 保持纯函数语义。

#### Scenario: 使用 tap 记录 tracing
- **WHEN** 需要在流中打印调试信息
- **THEN** MUST 使用 `.tap(|token| debug!(?token))` 或 `.tap(|_| span.log())`
- **AND** MUST NOT 在 `map` / `scan` 内部直接写入 tracing

#### Scenario: 使用 tap 写入模块缓存
- **WHEN** 模块加载后需要写入缓存
- **THEN** MUST 使用 `.tap(|module| { cache.insert(path, module.clone()) })`

### Requirement: 必须使用 Infallible 错误类型作为管道不变量
管道 Observable 的 Err 类型 MUST 为 `std::convert::Infallible`，编译错误 MUST 封装为 `Result<T, CompileError>` 作为流的正常值。

#### Scenario: 管道签名验证
- **WHEN** stage 函数定义签名
- **THEN** MUST 包含 `Infallible` 错误类型
- **AND** MUST NOT 使用 `Box<dyn Error>` 或其他错误类型

#### Scenario: 错误不调用 on_error
- **WHEN** 输入产生编译错误
- **THEN** MUST NOT 调用 `observer.error()`
- **AND** MUST 调用 `observer.next(Result::Err(CompileError::...))`

### Requirement: 必须使用 Subscription 管理生命周期
`subscribe()` 返回的 `Subscription` handle MUST 被妥善管理，UBSCRIBE 端负责在无需要时调用 `.unsubscribe()` 终止计算。

#### Scenario: 编译完成后的资源清理
- **WHEN** `.last()` 产出了最终 `Result`
- **THEN** MUST 自动触发 unsubscribe，释放 stage 内部资源

#### Scenario: 多文件终止
- **WHEN** 需要在第一个错误后停止处理多个输入
- **THEN** MUST 使用 `.take_while(|item| item.is_ok())` 或 `.unsubscribe()` 语义

### Requirement: 必须使用 box_it() 进行类型擦除
每个 stage 的输出 MUST 通过 `.box_it()` 进行类型擦除，避免 pipe chain 的泛型嵌套膨胀。

#### Scenario: pipeline.rs 使用 box_it
- **WHEN** 在 pipeline.rs 中链式调用各 stage
- **THEN** MUST 在每个 stage 后调用 `.box_it()`：
  ```rust
  Local::from_iter(chars)
    .pipe(tokenize).box_it()
    .pipe(parse).box_it()
    .pipe(evaluate).box_it()
    .pipe(serialize).box_it()
    .last()
    .subscribe(|result| { ... })
  ```

### Requirement: 必须使用 Higher-Rank Trait Bounds 注解闭包
所有 stage 定义的闭包 MUST 使用 `for<'a>` 生命周期注解，确保闭包可以处理任意生命周期的输入。

#### Scenario: stage 函数签名
- **WHEN** tokenize 定义 char→Token 变换闭包
- **THEN** MUST 标注为 `F: for<'a> FnMut(char) -> Token` 形式

### Requirement: 必须使用 Subject 实现跨 stage 状态共享
当多个下游观测者需要共享同一份上游发射数据时，MUST 使用 `Local::subject()` + `.publish()` / `.multicast()` 模式。

#### Scenario: 模块缓存共享
- **WHEN** 多个 evaluate 订阅者需要共享同一模块
- **THEN** MUST 使用 `let shared_module = Local::subject();`
- **AND** MUST 通过 `.multicast(shared_module.into_inner())` 实现共享
- **AND** 通过 `.ref_count()` 自动管理连接生命周期
