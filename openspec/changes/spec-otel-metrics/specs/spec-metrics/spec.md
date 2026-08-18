## ADDED Requirements

### Requirement: OTel Metrics 仪表盘记录 spec 测试通过率

系统 SHALL 在 spec 测试执行过程中通过 OTel Metrics（Counter/Gauge/Histogram）按域统计通过率、耗时分布，产出 `otel-metrics-spec-{label}.jsonl`。

#### Scenario: Counter 按域+结果累计测试用例计数

- **WHEN** 一个 spec 测试用例执行完成（pass/fail/skip）
- **THEN** 系统 MUST 调用 `Counter.add(1, [domain, result])` 累加计数
- **AND** `domain` 标签取值为 `SASS_SPEC_MAP.md` 中定义的 17 个域之一
- **AND** `result` 标签取值为 `"pass"` / `"fail"` / `"skip"`

#### Scenario: Histogram 记录单用例编译耗时

- **WHEN** 一个 spec 测试用例编译完成
- **THEN** 系统 MUST 调用 `Histogram.record(elapsed_ms, [domain])` 记录耗时
- **AND** 单位为毫秒
- **AND** 标签 `domain` 标识该用例所属域

#### Scenario: ObservableGauge 在所有测试跑完后报告通过率

- **WHEN** 所有 spec 测试用例执行完毕，调用 `finalize()`
- **THEN** 系统 MUST 注册 ObservableGauge 回调报告通过率
- **AND** Gauge 值为 `passed / total`（0.0-1.0）
- **AND** 标签 `domain` 取各域名 + `"OVERALL"`

#### Scenario: Metrics 初始化与关闭

- **WHEN** 测试文件调用 `init_metrics(label)`
- **THEN** 系统 MUST 创建 `SdkMeterProvider`，注册到 `global::set_meter_provider`
- **AND** 使用 `Once` 保证只初始化一次
- **WHEN** 测试文件调用 `shutdown_metrics()`
- **THEN** 系统 MUST 调用 `force_flush()` + `shutdown()` 确保所有 metrics 写入文件

#### Scenario: Metrics 与 Trace 共存

- **WHEN** 同一测试文件中同时初始化 OTel Trace 和 Metrics
- **THEN** 两者 MUST 独立运行互不干扰
- **AND** shutdown 顺序为：先 metrics（flush），后 trace（flush）

### Requirement: Metrics 产出文件格式

系统 SHALL 产出 `otel-metrics-spec-{label}.jsonl` 文件，每行一个 JSON 对象，包含指标名、标签和值。

#### Scenario: Counter 产出格式

- **WHEN** MetricExporter flush Counter 数据
- **THEN** 产出 JSONL 中每行包含 `"name": "spec_tests_total"`, `"attrs": {"domain": "...", "result": "..."}`, `"value": N`
- **AND** 每个 domain+result 组合对应一行

#### Scenario: Gauge 产出格式

- **WHEN** MetricExporter flush ObservableGauge 数据
- **THEN** 产出 JSONL 中每行包含 `"name": "spec_pass_rate"`, `"attrs": {"domain": "..."}`, `"value": 0.XXX`

#### Scenario: Histogram 产出格式

- **WHEN** MetricExporter flush Histogram 数据
- **THEN** 产出 JSONL 中每行包含 `"name": "spec_test_duration_ms"`, `"attrs": {"domain": "..."}`, 以及统计分布（count/sum/p50/p95/p99）
