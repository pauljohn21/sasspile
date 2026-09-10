## ADDED Requirements

### Requirement: 目录统计报告
系统 SHALL 生成 sass-spec 各目录的 pass/fail/skip 聚合统计报告。

#### Scenario: Markdown 报告输出
- **WHEN** 运行 `spec_store stats --md`
- **THEN** 系统输出 Markdown 表格，含 dir、pass、fail、skip、total、pct 列

#### Scenario: 带基线对比
- **WHEN** 运行 `spec_store stats --md --compare <snapshot_id>`
- **THEN** 报告额外输出 delta 列（与基线对比的 pass 变化量），以及 regressions/improvements 摘要

#### Scenario: JSON 输出
- **WHEN** 运行 `spec_store stats --json`
- **THEN** 系统输出结构化 JSON，含 total 和 per-directory 的聚合数据

### Requirement: 总体统计
系统 SHALL 计算并输出全局通过率。

#### Scenario: 全局摘要
- **WHEN** 运行 `spec_store stats`
- **THEN** 报告包含 total_pass / total_fail / total_skip / total_cases / pass_pct
