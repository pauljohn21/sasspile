# Proposal: Full Reactive Pipeline

## Why

当前管线在 Parser 入口、Evaluator 入口和 Serializer 出口处存在 `collect_boxed()` 断裂点——每个阶段先完整收集上游输出为 `Vec`，再命令式处理或重新包装为 Observable。这违背了"以 rxrust 为核心的纯响应式"架构原则，阻止了真正的流式编译（Level 3+）。

AI 在长上下文中容易渐进式地回归老路（保留 `for + push` 循环、保留中间 `Vec`），因此本次以**全面一次性重构**为目标，不留渐进妥协空间。

## What Changes

- **Lexer**: 去除 `box_it()`，返回具体 `Shared<Create<...>>` 类型，让 Parser 层统一处理类型擦除
- **Parser**: 从 `collect_boxed → 命令式 while 循环 → Shared::from_iter` 重写为 `scan_map(ParserState) + flat_map(Shared::from_iter)` 响应式管线。每个 token 增量驱动解析器状态机，完成的 AST 节点即时 emit
- **Evaluator 入口**: 去除 `eval_stream` 中的 `collect_boxed`，直接消费上游 `AstStream`，使用 `expand` 算子替代 `flat_map` 做语义正确的递归展开
- **Serializer**: 从纯函数 `&[CssStmt] → String` 改造为管线末端 `Observable<CssStmt> → Observable<String>`，使用 `fold(SerializeState)` 累积后 render
- **错误通道**: 所有类型别名从 `Infallible` 改为 `CompileError`，错误通过 Observable 错误通道传播
- **类型别名**: 统一为 `SharedBoxedObservable<'static, T, CompileError>`
- **collect_boxed**: 仅在最终消费端（`from_string`/`from_path`）使用，管道中间零 collect

## Capabilities

### Modified Capabilities

- `reactive-pipeline`: 管道定义从"中间有断裂的级联"升级为"全链路无断裂 Observable 管线"。错误传播从 `Infallible` 改为 `CompileError`。各阶段不再完整收集上游输出再处理。流式输出能力从理论变为实际可用。

## Impact

- **受影响文件**: `src/lexer/mod.rs`, `src/parser/mod.rs`, `src/parser/state.rs`, `src/eval/mod.rs`, `src/eval/emit.rs`, `src/serialize/mod.rs`, `src/types.rs`, `src/pipeline.rs`, `src/lib.rs`, `src/observable_ext.rs`
- **公开 API**: `from_string`/`from_path`/`CompileBuilder` 签名不变，内部改为订阅纯响应式管线
- **测试**: 所有集成测试应仍通过（行为等价），但可能需要适配错误类型变化
- **性能**: 消除中间 Vec 分配；理论上首个 CSS 产出延迟降低（流式优势）
- **无新增依赖**: 仅 rxrust 内部算子重组
