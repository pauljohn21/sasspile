## ADDED Requirements

### Requirement: 管线最终阶段是 Observable 到终值的约聚
编译管线的最终结果 MUST 通过 rxrust 的终结算子（.reduce() / .last() / .collect()）将 Observable 收敛为单个 Result 值。这是"output 是结果"的最终实现点。

#### Scenario: 单文件编译终值
- **WHEN** 输入一个完整的 input.scss
- **THEN** .last() / .reduce() MUST 产出 `Result<String, CompileError>`，唯一值写入 subscribe 闭包

#### Scenario: 空输入
- **WHEN** 输入为空 Observable（如空文件）
- **THEN** .reduce(initial, acc) MUST 返回初始值（空字符串或空 Result）；不能 panic

### Requirement: 管线可以落入同步或异步 Sink
Sink 的实现 MUST 支持同步订阅（Local context），SHOULD 支持未来扩展到异步 / future 订阅（Shared + Tokio），但当前阶段先落地同步版本。

#### Scenario: 同步订阅打印 CSS
- **WHEN** 订阅者订阅完整的编译管道
- **THEN** subscribe 闭包 MUST 仅在最后一个元素约聚后同步执行一次输出

#### Scenario: 未来扩展
- **WHEN** 用户希望并行处理多个文件
- **THEN** Sink 层 MUST 能演进到 `Shared::from_iter(...).merge_all(n).reduce(...).await`

### Requirement: Sink 不得提前订阅导致泄漏
Observable 是 lazy 的：如果没有 subscribe 被调用，pipeline 构造不产生任何计算。MUST 确保 subscribe 是整个计算的唯一入口。

#### Scenario: 只构造管道未订阅
- **WHEN** `let p = compile_pipeline(input);` 但不调用 .subscribe()
- **THEN** MUST 不执行任何 tokenize/parse/evaluate/serialize 逻辑；不分配扫描缓冲区

#### Scenario: 同一管道多次订阅
- **WHEN** 对同一编译管线多次调用 .subscribe()
- **THEN** 每次订阅 MUST 触发一次完整的 re-computation（cold observable rxrust 语义）

### Requirement: Result 携带编译元数据
最终的 Result MUST 携带编译元数据，至少包括：源哈希 / 输入大小、编译耗时（elapsed_ms）、命中哪种 stage。便于用户或上层工具计测编译性能。

#### Scenario: 记录编译耗时
- **WHEN** 管线完成 .last()
- **THEN** subscribe 闭包 MUST 收到包含 `elapsed_ms` 字段的 Result 记录，或能从最近的 tracing span 中读取

#### Scenario: 日志格式
- **WHEN** 打印到 tracing
- **THEN** MUST 输出阶段耗时分项：`{ tokenize: 0.2ms, parse: 0.5ms, evaluate: 1.1ms, serialize: 0.3ms }`
