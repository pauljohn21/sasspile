## ADDED Requirements

### Requirement: error! 宏不 panic 的测试执行模式

系统 SHALL 在 spec 测试失败时使用 `tracing::error!` 宏记录失败信息，不调用 `panic!`/`assert!`，确保所有测试用例跑完后 OTel trace 能完整 flush。

#### Scenario: 失败用例用 error! 记录不中断

- **WHEN** 一个 spec 测试用例的输出与 expected 不匹配
- **THEN** 系统 MUST 调用 `tracing::error!(domain, test_name, expected, actual, "FAIL")` 记录
- **AND** MUST NOT 调用 `panic!` 或 `assert!`
- **AND** MUST 继续执行下一个测试用例

#### Scenario: 编译错误的用例用 error! 记录

- **WHEN** spec 测试用例编译失败（`compile()` 返回 `Err`）
- **THEN** 系统 MUST 调用 `tracing::error!(domain, test_name, error=%e, "COMPILE_ERROR")` 记录
- **AND** MUST NOT panic
- **AND** MUST 继续执行下一个测试用例

#### Scenario: span 调用链自动桥接

- **WHEN** spec 测试用例执行时
- **THEN** 顶层 span `spec_test` MUST 记录 `domain` + `test_name` + `result` 字段
- **AND** `compile_pipeline` 内部的 span（tokenize/parse/evaluate/serialize）MUST 自动桥接为 OTel span
- **AND** 失败时 span 树 MUST 完整保留调用链用于根因定位

### Requirement: 统一 assert 在 trace flush 之后

系统 SHALL 在 `shutdown_otel()` + `shutdown_metrics()` 完成后执行统一 assert。

#### Scenario: 统一 assert 包含 trace 文件路径

- **WHEN** 所有测试用例执行完毕且 trace/metrics 已 flush
- **AND** 存在失败用例（`total_failed > 0`）
- **THEN** 系统 MUST 调用 `panic!` 报告失败
- **AND** panic 消息 MUST 包含 `otel-trace-spec-{label}.jsonl` 和 `otel-metrics-spec-{label}.jsonl` 文件路径
- **AND** panic 消息 MUST 包含 `total_failed / total` 计数

#### Scenario: 无失败用例时不 panic

- **WHEN** 所有测试用例执行完毕
- **AND** `total_failed == 0`
- **THEN** 系统 MUST NOT panic
- **AND** 测试 MUST 正常通过

### Requirement: Trace flush 保证

系统 SHALL 保证 OTel trace 在统一 assert 之前完整写入文件。

#### Scenario: shutdown_otel 在 assert 之前调用

- **WHEN** 所有测试用例执行完毕
- **THEN** 系统 MUST 先调用 `shutdown_otel()`（`force_flush` + `shutdown`）
- **AND** 然后调用 `shutdown_metrics()`（`force_flush` + `shutdown`）
- **AND** 最后才执行统一 assert
