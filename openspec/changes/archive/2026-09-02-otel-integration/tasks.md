# Tasks — otel-integration

## 1. Cargo.toml 配置

- [x] 1.1 添加 `opentelemetry` 0.32 optional 依赖
- [x] 1.2 添加 `opentelemetry_sdk` 0.32 optional 依赖
- [x] 1.3 添加 `opentelemetry-stdout` 0.32 optional 依赖
- [x] 1.4 添加 `tracing-opentelemetry` 0.33 optional 依赖
- [x] 1.5 添加 `otel` feature

## 2. 实现 init_tracing_otel()

- [x] 2.1 创建 stdout `SpanExporter`
- [x] 2.2 构建 `SdkTracerProvider`（with_simple_exporter + Resource service_name）
- [x] 2.3 创建 `OpenTelemetryLayer` + `fmt` layer 双 layer 订阅
- [x] 2.4 `#[cfg(feature = "otel")]` 条件编译
- [x] 2.5 `#[cfg(not(feature = "otel"))]` 回退到 `init_tracing()`
- [x] 2.6 `OTEL_SERVICE_NAME` 环境变量支持

## 3. sass_spec_full.rs 改用 OTel

- [x] 3.1 `test_import_use_forward` → `init_tracing_otel()`
- [x] 3.2 `test_directives_subdirs` → `init_tracing_otel()`
- [x] 3.3 `test_sass_spec_full_stats` → `init_tracing_otel()`
- [x] 3.4 `test_core_functions_subdirs` → `init_tracing_otel()`

## 4. 验证

- [x] 4.1 `cargo check --features otel` 编译通过
- [x] 4.2 `cargo test --features otel --test sass_spec_full test_import_use_forward -- --nocapture` — OTel span 输出正常
- [x] 4.3 `cargo test --features otel --test sass_spec_full test_sass_spec_full_stats -- --nocapture` — 3216/5624 = 57%
- [x] 4.4 验证 span 层级（TraceId/ParentSpanId）正确
- [x] 4.5 验证 `busy_ns`/`idle_ns` 精确耗时记录

## 5. 更新文档

- [x] 5.1 AGENTS.md — 添加 OTel 追踪架构章节
- [x] 5.2 AGENTS.md — 调试协议 Step 2 添加 OTel 命令
- [x] 5.3 AGENTS.md — 常用命令添加 OTel 追踪
- [x] 5.4 AGENTS.md — 自检清单添加 OTel 检查项
- [x] 5.5 AGENTS.md — 归档列表添加 hrx-auditor-removal + otel-integration
- [x] 5.6 skill.md — 调试命令添加 OTel 追踪
- [x] 5.7 skill.md — 测试命令更新 sass-spec 基线
- [x] 5.8 .claude/skills/tracing-debug/SKILL.md — 步骤 2 添加 OTel 追踪
- [x] 5.9 .claude/skills/tracing-debug/SKILL.md — 步骤 5 添加 OTel 验证
- [x] 5.10 docs/CODE_INDEX.md — lib.rs 行数和职责更新
- [x] 5.11 README.md — 更新 sass-spec 基线 + 添加 OTel 说明
- [x] 5.12 src/lib.rs — 更新注释中的 sass-spec 基线
