# Spec: Reactor Core

## ADDED Requirements

### Requirement: Reactor 消费-返回模式
REACTOR SHALL 提供消费 `self` 并返回新 `Reactor` 实例的管线方法, 禁止 `&mut self` 方法。

#### Scenario: lex() 消费 Reactor
- **WHEN** 调用 `reactor.lex()`
- **THEN** 旧 `reactor` 被 move (不可再用)
- **AND** 返回 `Result<Reactor<Lexed>>` 包含 tokens

#### Scenario: 重复使用已消费的 Reactor 在编译期报错
- **WHEN** 尝试在 `reactor.lex()` 后再次使用 `reactor`
- **THEN** Rust 所有权系统产生编译错误: "use of moved value"

### Requirement: Reactor 类型状态机
REACTOR SHALL 通过泛型参数编码管线阶段, 保证无效操作在编译期被阻止。

#### Scenario: 未 lex 前不能 parse
- **WHEN** 尝试 `Reactor::new(src).parse()`
- **THEN** 编译错误: `Reactor<Raw>` 没有 `parse()` 方法

### Requirement: ReactorIO Trait
REACTOR SHALL 通过 `ReactorIO` trait 将文件 IO 抽象化, 使测试可 mock。

#### Scenario: 测试注入 Mock IO
- **WHEN** 构建 `Reactor::new(src).with_io(Arc::new(MockReactorIO::new(files)))`
- **THEN** 所有 `@import` 解析都从内存 HashMap 读取, 不触发文件系统 IO
- **AND** sass-spec CI 可在无网络环境下运行

### Requirement: 链式追踪 Span
REACTOR SHALL 为每个管线阶段创建 `tracing::Span` 或 OTel Span, 错误时自动携带上下文。

#### Scenario: 错误发生时 trace 携带 stage 信息
- **WHEN** evaluator 在 `@import "missing"` 失败
- **THEN** OTel span 包含字段:
  - `stage = "evaluate"`
  - `io.path = "missing.scss"`
  - `error = "File not found"`
  - `parentSpanId` 指向 evaluate 阶段 span
- **AND** 可还原从 lex 到失败点的完整调用链

#### Scenario: 测试可直接读取 Reactor 内置 trace
- **WHEN** `reactor.trace.snapshot()` 在单测中调用
- **THEN** 返回 `Vec<SpanSnapshot>` 包含完整 OTel 阶段树
- **AND** 可通过 `snapshot.iter().find(|s| s.name == "color.scale")` 定位特定 span

### Requirement: imbl 持久化数据结构
REACTOR SHALL 使用 `imbl` crate (而非废弃的 `im`) 提供 Env/Scope/ModuleCache 的持久化数据结构。

#### Scenario: Cargo.toml 依赖 imbl
- **WHEN** 查看 Cargo.toml
- **THEN** 包含 `imbl = "3"` (或最新版本)
- **AND** 不包含废弃的 `im` crate

#### Scenario: Reactor clone 为 O(log n) 结构共享
- **WHEN** 在作用域内执行 `env.bind("x", val)`
- **THEN** 返回新 `Env`, 旧 `Env` 不变
- **AND** clone 成本为 O(log n), 不作为 O(n) 全量拷贝
