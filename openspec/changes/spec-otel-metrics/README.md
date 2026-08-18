# spec-otel-metrics

OTel Metrics + Trace 驱动的 sass-spec 测试基础设施 + 独立 spec 数据集

## 产出物

### 独立 spec 数据集
- `spec_dataset.json` (16MB) — 从 sass-spec HRX 提取的纯数据集，20504 条测试用例
- `scripts/gen_spec_dataset.rs` — 数据集生成器（rust-script）
- 完全独立于任何编译器实现，可作参照数据

### 独立对照工具
- `scripts/spec_check.rs` — 给定编译器命令 + 数据集，用 tracing span 记录证据链
- `catch_unwind` 兜住编译器 panic
- 产出 `spec_check_*.json` 报告

### 内嵌 OTel metrics 测试框架
- `tests/spec_otel_runner.rs` — Counter/Histogram/ObservableGauge
- `tests/spec_baseline.rs` — 全量基线（#[ignore]）
- `tests/hrx_vfs.rs` — 内存 VFS 支持多文件测试
- 17 个 `spec_*.rs` 域测试文件

### CLI 入口
- `src/main.rs` — sasspile CLI（compile_file）

### 全量基线结果
| 指标 | 值 |
|------|-----|
| Total cases | 20,504 |
| Passed | 2,346 |
| Failed | 18,158 |
| Pass rate | 11.44% |
| HRX files | 2,177 |
