## 1. 移除 spec_store skip 逻辑

- [x] 1.1 修改 `tests/specstore/runner.rs:42` — 删除 `if expected_css.is_empty() && !expect_error { return Skip; }` 分支，让空输出 case 继续执行编译流程

## 2. 移除 hrx_support skip 逻辑

- [x] 2.1 修改 `tests/hrx_support.rs:378-380` — 删除 `if case.expected_output.is_empty() && !case.expect_error { return true; }` 分支

## 3. 移除 sass_spec_full skip 逻辑

- [x] 3.1 修改 `tests/sass_spec_full.rs:91-94` — 删除空输出 skip 分支，让空输出 case 计入 cases 并通过 run_case 评估

## 4. 验证与统计

- [x] 4.1 运行 `cargo test --test compile_test --test stage_test --test ast_test --test common_test --test bs_spec --test ep_full` 确认核心测试全通过
- [x] 4.2 运行 `SPEC_STORE_CMD=run` + `SPEC_STORE_CMD=stats` 获取新增评估 case 分布
- [x] 4.3 确认 `variables/` 目录通过率达到 100%（3 SCSS 子目录）
- [x] 4.4 分析新增 FAIL case — 如有编译器行为问题则修复（优先）或记录为已知限制

## 5. CHANGELOG 更新

- [x] 5.1 在 CHANGELOG.md 记录测试框架变更及通过率 delta
