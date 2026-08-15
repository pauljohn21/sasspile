# Decision 001: 管道架构使用 Tokio spawn + mpsc channel

## Status
Accepted

## Context
每个编译阶段需要独立并行执行，同时需要自然的背压机制和错误传播。

## Decision
每个编译阶段作为独立 `tokio::spawn` 任务，通过 `mpsc::channel` 连接管道。

## Alternatives Considered
- `rayon` 数据并行 → 不适合 IO 密集型 + 流式场景
- `async fn` 链式调用 → 无法利用多核并行
- `Generator` / `coroutine` → 不稳定特性

## Consequences
- ✅ 充分利用多核
- ✅ mpsc 提供自然背压（channel 满时自动阻塞上游）
- ✅ 各阶段可独立测试、替换
- ⚠️ 异步管道调试困难（需 tracing span）

## Related
- architecture.md
- pipeline.md
- tasks.md Phase 9
