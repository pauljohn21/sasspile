## Why

sass-spec 测试框架将"期望输出为空 + 非错误"的 case 标记为 SKIP，导致 `variables/` 目录 11/14 个 case 被跳过（0% 而非 100%），全局 2620 个 case 被跳过。这些 case 实际验证"变量声明/@use/@mixin 等不产生 CSS"的正确行为，跳过它们意味着失去回归防护。

## What Changes

- **移除三处 empty-output skip 逻辑**：`tests/specstore/runner.rs`、`tests/hrx_support.rs`、`tests/sass_spec_full.rs` 中的 `if expected_output.is_empty() && !expect_error { skip }` 分支
- 空输出 case 改为实际编译并验证输出为空字符串
- 不修改编译器行为，仅改变测试覆盖范围

## Capabilities

### New Capabilities

- `test-harness-empty-output`: 测试框架对空输出 case 的处理行为——不再跳过，实际编译验证

### Modified Capabilities

（无编译器能力变更）

## Impact

- **受影响文件**：`tests/specstore/runner.rs`、`tests/hrx_support.rs`、`tests/sass_spec_full.rs`
- **测试影响**：全局新增 ~2600 个评估 case（从 9511 → 12131）
- **运行时影响**：`SPEC_STORE_CMD=run` 预计 +20-30 秒
- **通过率影响**：预期新增 ~2500 pass、~50-150 fail（需验证）
- **无 API 变更、无编译器行为变更**
