## Why

sass-spec 测试框架当前有三个致命问题：

1. **假通过**：12 个 spec 测试文件（spec_css、spec_directives 等）只统计 pass/fail 计数不 assert，`cargo test` 报告 "all passed" 但实际大量用例失败
2. **多文件黑洞**：`run_hrx_tests` 对多文件测试直接返回 `passed: true, message: "SKIPPED"`，~40% 的用例被静默跳过
3. **无量化追踪**：无法知道当前通过率，无法对比版本间进步/回归，无法定位失败用例的根因层

项目已有完整的 OTel trace 基础设施（`tracing_init.rs` + `otel_test_harness.rs`），但只用于 Bootstrap/Bulma 等真实项目编译追踪，未覆盖 spec 测试。

## What Changes

- 在 spec 测试 runner 中集成 OTel **Metrics**（Counter/Gauge/Histogram）+ **Trace**（error! 宏 + span 树），构建量化仪表盘 + 调试证据链
- Metrics 产出 `otel-metrics-spec.jsonl`：按域统计 pass/fail/skip 计数、通过率 Gauge、测试耗时 Histogram
- Trace 产出 `otel-trace-spec.jsonl`：失败用例的完整 span 调用链，`error!` 宏记录不 panic，flush 后统一 assert
- 实现 HRX VFS（内存文件系统），解锁多文件测试
- 新建 `spec_baseline.rs`：RecordOnly 模式跑完全部 1306 个 HRX，产出 baseline JSON
- 新建 `spec_diff.rs`（rust-script）：对比两次 baseline，输出新增通过/回归/跳过变化

## Non-goals

- 不修改编译器代码（src/ 下不改一行）
- 不实现新的 Sass 语言功能
- 不对接 Grafana/Prometheus（后续迭代）
- 不实现 .sass 缩进语法支持
- 不修改现有 otel_test_harness.rs（真实项目测试不变）

## Capabilities

### New Capabilities

- `spec-metrics`: OTel Metrics 仪表盘——Counter/Gauge/Histogram 按域统计 spec 通过率、耗时分布，产出 `otel-metrics-spec.jsonl`
- `spec-trace-evidence`: OTel Trace 证据链——失败用例用 `error!` 宏记录不 panic，完整 span 调用链写入 `otel-trace-spec.jsonl`，flush 后统一 assert
- `hrx-vfs`: HRX 内存文件系统——解析 HRX 中所有文件到 HashMap，通过 `VfsResolver` 实现 `ModuleResolver` trait，解锁多文件测试
- `spec-baseline`: 全量基线——RecordOnly 模式跑完全部 1306 个 HRX，产出结构化 baseline JSON，支持跨版本 diff

### Modified Capabilities

（无现有 spec 需要修改）

## Impact

- **`Cargo.toml`** — `opentelemetry-stdout` features 加 `"metrics"`
- **`tests/tracing_init.rs`** — 新增 `init_metrics()` / `shutdown_metrics()`，创建 `SdkMeterProvider`
- **新增 `tests/spec_otel_runner.rs`** — 基于 `spec_runner.rs` 改造，加 Metrics + Trace + error! 不 panic 模式
- **新增 `tests/hrx_vfs.rs`** — HRX VFS 解析 + `VfsResolver` 实现
- **新增 `tests/spec_baseline.rs`** — 全量 HRX 基线测试，`#[ignore]` 默认不跑
- **新增 `scripts/spec_diff.rs`** — rust-script，baseline diff 工具
- **改造 17 个 `spec_*.rs`** — 统一改用 `spec_otel_runner`，AssertAll 模式
