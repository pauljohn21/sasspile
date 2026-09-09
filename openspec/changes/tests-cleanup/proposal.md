## Why

tests/ 目录有 54 个文件、8750 行代码，其中 12 个诊断文件共享 ~90% 重复的 HRX parse/compile 逻辑。临时调试文件（extend_debug.rs、trace_table.rs）和被替代工具（failures_inspect.rs 被 failures_json.rs 替代）增加了维护负担。compile_test.rs（551 行）超限需拆分。本次重组减少文件数量、消除重复、使所有文件 ≤500 行。

## What Changes

- **删除 7 个临时/废弃文件**：extend_debug.rs、trace_table.rs、diag_hsl2.rs、diag_hsl3.rs、cfs_diag2.rs、failures_inspect.rs、failures_analyze.rs
- **合并 12 个诊断文件为 1 个** `diagnostic_runner.rs`：统一 parse/compile/collect 逻辑，保留所有统计和诊断测试入口
- **拆分 compile_test.rs**：将 CSS Color Level 4 色彩空间测试（14个）拆出为 `compile_color_test.rs`
- **整合 hwb_spec_diag.rs**：3 个边界测试移入 compile_test.rs 作为永久回归测试，原文件删除
- **file_size_check.rs 合入 compile_test.rs**：1个检查测试合并

## Capabilities

### New Capabilities
- （无新增能力）

### Modified Capabilities
- `test-infrastructure`: 诊断工具从 12 个分散文件合并为 1 个统一接口，测试函数名称和行为保持不变

## Impact

- **文件变化**: 54 → ~37 个文件（-17），总行数 8750 → ~6200（-28%）
- **API 影响**: 无公共 API 变更，仅内部测试文件重组
- **测试影响**: 所有现有测试函数保留，不丢失任何测试覆盖；cargo test 验证清单不变
- **诊断工具影响**: `cargo test --test diagnostic_runner --name diag_xxx` 替代原来 12 个文件的独立调用
