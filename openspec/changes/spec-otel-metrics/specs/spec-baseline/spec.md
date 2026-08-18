## ADDED Requirements

### Requirement: 全量 sass-spec 基线测试

系统 SHALL 提供 `spec_baseline.rs` 测试文件，以 RecordOnly 模式跑完全部 1306 个 HRX，产出结构化 baseline JSON。

#### Scenario: 基线测试默认不运行

- **WHEN** 运行 `cargo test`
- **THEN** `test_baseline_all` MUST 标记 `#[ignore]`，默认不执行
- **AND** 只有 `cargo test --test spec_baseline -- --ignored` 才触发

#### Scenario: 基线测试跑完全部 HRX 不中断

- **WHEN** `test_baseline_all` 执行
- **THEN** 系统 MUST 遍历 `sass-spec/spec/` 下所有 `.hrx` 文件（排除 `libsass`、`non_conformant`）
- **AND** 失败用例 MUST 用 `tracing::error!` 记录，不 panic
- **AND** 全部 1306 个 HRX MUST 跑完

#### Scenario: 基线测试产出 JSON

- **WHEN** `test_baseline_all` 执行完毕
- **THEN** 系统 MUST 产出 `spec_baseline_{timestamp}.json` 文件
- **AND** JSON MUST 包含 `timestamp`, `version`, `total`, `passed`, `failed`, `skipped`, `pass_rate`
- **AND** JSON MUST 包含 `domains` 对象，每个域有 `{total, passed, failed, skipped}`
- **AND** JSON MUST 包含 `failed_tests` 数组，每条包含 `{domain, name, trace_span_id, mismatch}`

### Requirement: Baseline diff 工具

系统 SHALL 提供 `scripts/spec_diff.rs`（rust-script），对比两次 baseline JSON，输出新增通过/回归/跳过变化。

#### Scenario: diff 两个 baseline

- **WHEN** 运行 `rust-script scripts/spec_diff.rs --old old.json --new new.json`
- **THEN** 系统 MUST 输出 Markdown 格式报告
- **AND** 报告 MUST 包含：新增通过用例列表、新增失败用例列表、回归用例列表
- **AND** 报告 MUST 按 domain 分组

#### Scenario: 域列表与 SASS_SPEC_MAP.md 对应

- **WHEN** baseline 测试定义域列表
- **THEN** 域列表 MUST 与 `SASS_SPEC_MAP.md` 中定义的 17 个域对应
- **AND** 包括：css/plain, css/selector, css/media, css/supports, css/custom_properties, css/functions, css/moz_document, css/unicode_range, css/unknown_directive, directives, expressions, operators, parser, values, variables, callable, core_functions
