## Why

sasspile 已有 `tracing` feature 提供 span/event 追踪，但在深度链路分析时缺少：
- **TraceId/SpanId** — 无法追踪完整调用链层级
- **精确耗时** — `tracing` 只有时间戳，无 `busy_ns`/`idle_ns`
- **OpenTelemetry 生态兼容** — 无法接入 OTel Collector 或其他后端

sass-spec 全量测试约 5600 个用例，失败诊断需要精确的 span 层级和耗时数据。

## What Changes

1. **添加 `otel` feature**：`opentelemetry` 0.32 + `opentelemetry_sdk` + `opentelemetry-stdout` + `tracing-opentelemetry` 0.33
2. **实现 `init_tracing_otel()`**：stdout exporter 同步输出 span，无需 gRPC/tokio
3. **sass_spec_full.rs 所有测试改用 `init_tracing_otel()`**
4. **更新文档**：AGENTS.md、skill.md、tracing-debug SKILL.md 添加 OTel 使用方法

## Impact

- OTel span 输出包含：TraceId/SpanId/ParentSpanId/busy_ns/idle_ns + 业务字段
- 性能开销：约 41s vs 44s（无额外开销）
- `--features otel` 编译时启用，不影响默认构建
