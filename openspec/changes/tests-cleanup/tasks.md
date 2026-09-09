## 1. 删除临时/废弃文件

- [ ] 1.1 删除 extend_debug.rs（未跟踪临时文件）
- [ ] 1.2 删除 trace_table.rs（硬编码个人路径，非共享资产）
- [ ] 1.3 删除 failures_inspect.rs（被 failures_json.rs 替代）
- [ ] 1.4 删除 failures_analyze.rs（被 failures_json.rs + sass_spec_stats.rs 替代）

## 2. 合并诊断工具 — 核心共享逻辑

- [ ] 2.1 创建 tests/diagnostic_runner.rs，定义共享 HrxCase 结构体 + parse_case() + compile_case() + collect_hrx_files()
- [ ] 2.2 实现 mod core_functions 模块，包含所有 cf_diag.rs 中的 diag_* 和 stats_* 测试函数
- [ ] 2.3 从 cfs_diag.rs 和 cfs_units.rs 迁移 change/scale/hsl 诊断测试
- [ ] 2.4 从 css_diag.rs 迁移 css_fail_details 测试
- [ ] 2.5 从 expr_diag.rs 迁移 expr_fail_details 测试
- [ ] 2.6 从 use_diag.rs 迁移 use 子目录统计测试
- [ ] 2.7 从 diag_detail.rs 迁移 diag_output_mismatch 测试
- [ ] 2.8 从 diag_directives.rs 迁移 diag_forward_extend 测试
- [ ] 2.9 从 diag_hsl2.rs 和 diag_hsl3.rs 迁移 HSL 相关诊断（合并为 1-2 个测试函数）

## 3. 删除已合并的源文件

- [ ] 3.1 删除 cf_diag.rs（已合并入 diagnostic_runner.rs）
- [ ] 3.2 删除 cfs_diag.rs（已合并入 diagnostic_runner.rs）
- [ ] 3.3 删除 cfs_diag2.rs（逻辑与 cfs_diag.rs 重复）
- [ ] 3.4 删除 cfs_units.rs（已合并入 diagnostic_runner.rs）
- [ ] 3.5 删除 css_diag.rs（已合并入 diagnostic_runner.rs）
- [ ] 3.6 删除 diag_detail.rs（已合并入 diagnostic_runner.rs）
- [ ] 3.7 删除 diag_directives.rs（已合并入 diagnostic_runner.rs）
- [ ] 3.8 删除 diag_hsl2.rs（已合并入 diagnostic_runner.rs）
- [ ] 3.9 删除 diag_hsl3.rs（已合并入 diagnostic_runner.rs）
- [ ] 3.10 删除 expr_diag.rs（已合并入 diagnostic_runner.rs）
- [ ] 3.11 删除 use_diag.rs（已合并入 diagnostic_runner.rs）

## 4. 拆分 compile_test.rs

- [ ] 4.1 创建 tests/compile_color_test.rs，将 14 个 test_compile_color_* 测试从 compile_test.rs 移出
- [ ] 4.2 将原 hwb_spec_diag.rs 的 3 个测试函数合并入 compile_test.rs
- [ ] 4.3 将原 file_size_check.rs 的 2 个测试函数合并入 compile_test.rs
- [ ] 4.4 删除 hwb_spec_diag.rs
- [ ] 4.5 删除 file_size_check.rs

## 5. 验证

- [ ] 5.1 运行 `cargo test --test compile_test` 确认 43+ 测试全通过
- [ ] 5.2 运行 `cargo test --test compile_color_test` 确认 14+ 测试全通过
- [ ] 5.3 运行 `cargo test --test diagnostic_runner` 确认能编译并运行
- [ ] 5.4 运行 `cargo test --test stage_test && cargo test --test ast_test && cargo test --test common_test` 确认不受影响
- [ ] 5.5 运行 `cargo test --test bs_spec` 确认不受影响
- [ ] 5.6 确认无文件超 500 行（`wc -l tests/*.rs | awk '$1 > 500'` 输出为空）
