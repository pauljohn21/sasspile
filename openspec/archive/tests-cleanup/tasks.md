## 1. 删除临时/废弃文件

- [x] 1.1 删除 extend_debug.rs（未跟踪临时文件）
- [x] 1.2 删除 trace_table.rs（硬编码个人路径，非共享资产）
- [x] 1.3 删除 failures_inspect.rs（被 failures_json.rs 替代）
- [x] 1.4 删除 failures_analyze.rs（被 failures_json.rs + sass_spec_stats.rs 替代）

## 2. 合并诊断工具 — 拆分为 3 文件（500 行约束）

- [x] 2.1 创建 tests/diag_helper.rs，定义共享 HrxCase 结构体 + parse_cases() + compile_case() + collect_hrx() + prelude 模块
- [x] 2.2 创建 tests/diagnostic_runner.rs（≤ 300 行），包含 core_functions / directives / css / expr / use 测试函数
- [x] 2.3 创建 tests/diag_color.rs（≤ 300 行），包含 color change/scale/hsl/hwb 诊断 + HSL 统计 + values diag_output_mismatch
- [x] 2.4 从 cfs_diag.rs 和 cfs_units.rs 迁移 change/scale/hsl 诊断测试 → diag_color.rs
- [x] 2.5 从 css_diag.rs 迁移 css_fail_details 测试 → diagnostic_runner.rs
- [x] 2.6 从 expr_diag.rs 迁移 expr_fail_details 测试 → diagnostic_runner.rs
- [x] 2.7 从 use_diag.rs 迁移 use 子目录统计测试 → diagnostic_runner.rs
- [x] 2.8 从 diag_detail.rs 迁移 diag_output_mismatch 测试 → diag_color.rs
- [x] 2.9 从 diag_directives.rs 迁移 diag_forward_extend 测试 → diagnostic_runner.rs
- [x] 2.10 从 diag_hsl2.rs 和 diag_hsl3.rs 迁移 HSL 相关诊断 → diag_color.rs

## 3. 删除已合并的源文件

- [x] 3.1 删除 cf_diag.rs（已合并入 diagnostic_runner.rs）
- [x] 3.2 删除 cfs_diag.rs（已合并入 diag_color.rs）
- [x] 3.3 删除 cfs_diag2.rs（逻辑与 cfs_diag.rs 重复）
- [x] 3.4 删除 cfs_units.rs（已合并入 diag_color.rs）
- [x] 3.5 删除 css_diag.rs（已合并入 diagnostic_runner.rs）
- [x] 3.6 删除 diag_detail.rs（已合并入 diag_color.rs）
- [x] 3.7 删除 diag_directives.rs（已合并入 diagnostic_runner.rs）
- [x] 3.8 删除 diag_hsl2.rs（已合并入 diag_color.rs）
- [x] 3.9 删除 diag_hsl3.rs（已合并入 diag_color.rs）
- [x] 3.10 删除 expr_diag.rs（已合并入 diagnostic_runner.rs）
- [x] 3.11 删除 use_diag.rs（已合并入 diagnostic_runner.rs）

## 4. 拆分 compile_test.rs

- [x] 4.1 创建 tests/compile_color_test.rs，将 14 个 test_compile_color_* 测试从 compile_test.rs 移出
- [x] 4.2 将原 hwb_spec_diag.rs 的 3 个测试函数合并入 compile_test.rs
- [x] 4.3 将原 file_size_check.rs 的 2 个测试函数合并入 compile_test.rs
- [x] 4.4 删除 hwb_spec_diag.rs
- [x] 4.5 删除 file_size_check.rs

## 5. 验证

- [x] 5.1 运行 `cargo test --test compile_test` → 通过 48 tests, 0 failed
- [x] 5.2 运行 `cargo test --test compile_color_test` → 通过 14 tests, 0 failed
- [x] 5.3 运行 `cargo test --test diagnostic_runner` → 编译并通过
- [x] 5.4 运行 `cargo test --test stage_test(8) && ast_test(8) && common_test(5)` → 全部通过
- [x] 5.5 运行 `cargo test --test bs_spec` → 15/15 通过
- [x] 5.6 确认无文件超 500 行（最大 hrx_support.rs 499 行，compile_test.rs ~480 行）
