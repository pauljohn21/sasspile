## Context

sasspile 使用 `tracing` crate 提供 span/event 追踪，通过 `init_tracing()` 初始化 `tracing-subscriber` fmt layer。但在 sass-spec 全量测试（5600+ 用例）中，需要更精确的调用链分析能力。

## Goals / Non-Goals

**Goals:**
- 添加 `otel` feature，提供 OpenTelemetry stdout span 输出
- 无需 gRPC Collector 或 tokio runtime
- `init_tracing_otel()` 在 `otel` feature 未启用时回退到 `init_tracing()`
- sass_spec_full.rs 所有测试使用 OTel 追踪
- 文档中补充 OTel 使用方法

**Non-Goals:**
- 不引入 async runtime（tokio/async-std）
- 不用 gRPC OTLP exporter
- 不改 src/ 生产代码的 span 逻辑
- 不改默认 feature（`default = ["tracing"]`）

## Decisions

### D1: stdout exporter（非 gRPC）

```rust
let exporter = opentelemetry_stdout::SpanExporter::default();
let provider = SdkTracerProvider::builder()
    .with_simple_exporter(exporter)
    .with_resource(Resource::builder().with_service_name(service_name).build())
    .build();
```

stdout exporter 同步输出 span 到 stdout，无需 async runtime。适合编译器场景（非服务端长运行）。

### D2: 双模式初始化

| 函数 | feature | 输出 |
|------|---------|------|
| `init_tracing()` | `tracing` | tracing fmt 日志 |
| `init_tracing_otel()` | `otel` | tracing fmt + OTel stdout span |

`init_tracing_otel()` 在 `otel` feature 未启用时通过 `#[cfg(not(feature = "otel"))]` 回退到 `init_tracing()`，无需条件编译。

### D3: Cargo.toml feature 配置

```toml
[features]
default = ["tracing"]
tracing = ["dep:tracing", "dep:tracing-subscriber"]
otel = ["dep:opentelemetry", "dep:opentelemetry_sdk", "dep:opentelemetry-stdout", "dep:tracing-opentelemetry"]
```

`otel` feature 隐含 `tracing`（通过 `tracing-opentelemetry` 桥接 layer）。

### D4: OTel Span 业务字段

OTel 自动捕获 tracing span 的字段作为 span attributes：
- `stage`、`module` — 管道阶段和功能模块
- `dir`、`hrx` — sass-spec 目录和 HRX 文件
- `pass`、`fail`、`pct` — 统计数据
- `busy_ns`、`idle_ns` — OTel 自动耗时

### D5: sass_spec_full.rs 改用 init_tracing_otel()

所有 4 个测试函数从 `init_tracing()` 改为 `init_tracing_otel()`。在 `--features otel` 编译时输出 OTel span；不启用时回退到普通 tracing。

## Risks

- **stdout 输出量大** → 通过 `RUST_LOG` 过滤控制
- **性能开销** → 实测约 0 额外开销（41s vs 44s）
- **版本兼容** → `tracing-opentelemetry` 0.33 + `opentelemetry` 0.32 需匹配

## 改动范围

| 文件 | 改动 |
|------|------|
| `Cargo.toml` | 添加 otel feature + 4 个 optional 依赖 |
| `src/lib.rs` | 添加 `init_tracing_otel()` + `#[cfg]` 回退 |
| `tests/sass_spec_full.rs` | 4 个测试函数改用 `init_tracing_otel()` |
| `AGENTS.md` | 添加 OTel 架构说明 + 调试命令 + 自检清单 |
| `skill.md` | 添加 OTel 追踪命令 |
| `.claude/skills/tracing-debug/SKILL.md` | 添加 OTel 追踪命令 |
