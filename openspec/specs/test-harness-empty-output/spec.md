## ADDED Requirements

### Requirement: 空输出 case 不再跳过
测试框架 SHALL 将"期望输出为空 + 非错误"的 case 视为正常评估对象，实际编译并验证输出为空字符串，而非跳过。

#### Scenario: spec_store 不跳过空输出 case
- **WHEN** `specstore::runner::run_case` 处理期望输出为空且非错误的 case
- **THEN** 执行编译并比较 `actual.trim() == expected.trim()`，返回 Pass/Fail 而非 Skip

#### Scenario: sass_spec_full 不跳过空输出 case
- **WHEN** `sass_spec_full::run_spec_dir` 遍历 case
- **THEN** 空输出 case 被计入 `cases` 并通过 `run_case` 实际评估，不增加 `skip` 计数

#### Scenario: hrx_support run_case 实际评估空输出
- **WHEN** `hrx_support::run_case` 接收空输出非错误 case
- **THEN** 执行编译并返回比较结果，不直接返回 true

### Requirement: 空输出验证方式
空输出 case 的验证 SHALL 采用 trim 后比较，允许编译产物含空白字符（换行符等）。

#### Scenario: 编译产物含尾部换行
- **WHEN** 编译产生 `"\n"` 或 `"  "` 而期望输出为 `""`
- **THEN** `actual.trim() == expected.trim()` 为 true，判定 PASS
