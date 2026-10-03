## 阶段 1 — BEM Mixin 上下文修复 ✅ COMPLETED (2026-09-22)

- [x] 1.1 在 `eval_mixin` 入口 capture current_selector 到 mixin 执行上下文
- [x] 1.2 修复 mixin 体内 `$selector: &` 插值时的上下文解析
- [x] 1.3 修复 m() mixin 在 e() mixin 内嵌套时 modifier 分裂
- [x] 1.4 运行 `ep_normalized_test` 确认 +N IDENTICAL，核心测试 202/202
- [x] 1.5 运行 `SPEC_STORE_CMD=run` 确认 sass-spec 无负向变化

## 阶段 2 — 括号感知 + literal safety ✅ COMPLETED (2026-10-02)

- [x] 2.1 在 `rule.rs` 实现 `split_selectors_respecting_parens`，按括号嵌套深度分割选择器
- [x] 2.2 color-picker `:not(.is-disabled, .is-focused)` 语义正确输出
- [x] 2.3 修复 `parse_literal_arg` 1-char string panic (`src/eval/value/mod.rs`)
- [x] 2.4 运行 `ep_normalized_test` 确认 83/121
- [x] 2.5 运行 `SPEC_STORE_CMD=run` 确认 sass-spec +6 (7921 → 7927)
- [x] 2.6 所有核心测试 202/202 通过

## 阶段 3 — at-root 嵌套上下文修复 ✅ COMPLETED (2026-10-03)

- [x] 3.1 新增 `at_root_top: bool` 字段至 `Env` 结构（env.rs）
- [x] 3.2 新增 `with_at_root_top` 构造方法（env_impl.rs）
- [x] 3.3 `eval_at_root()` 入口设置 `env.with_at_root_top(true)`（mixin.rs）
- [x] 3.4 在 `eval_rule()` descenter 实现 compose-in-descender（depth > 0 + !at_root_top 时组合父链）
- [x] 3.5 `enter_scope()` 时 reset `at_root_top = false`（rule.rs）
- [x] 3.6 在 `RuleBuilder::push()` Rule arm 增加 `starts_with` 前缀检测避免双层膨胀
- [x] 3.7 修复 descriptions.scss：`e(title)` 在 `m($size)` 内正确输出完整链
- [x] 3.8 验证 descriptions.scss：`.el-descriptions--large .el-descriptions__header .el-descriptions__title` 完全匹配 EP dist
- [x] 3.9 核心测试 119/119 通过（含 extend 回归测试）
- [x] 3.10 sass-spec 全量验证无回归
- [x] 3.11 EP consistency DIFF 16→15

## 阶段 4 — Sasspile bug 修复（剩余）

- [ ] 4.1 修复 rgba()/rgb() var() fallback 在 input-number 等文件中的展开
- [ ] 4.2 运行 `SPEC_STORE_CMD=run` 确认 sass-spec ≥ 7927 无回归
- [ ] 4.3 归档 openspec 到 `openspec/changes/archive/`

## 阶段 5 — 收敛 & 验证（最终）

- [ ] 5.1 针对所有已知管线差异文件，在 normalize post-process 中标记为 KNOWN_DIFF
- [ ] 5.2 软验证：actionable patch 全部应用后 EP 一致率达到可接受基线
- [ ] 5.3 全量 `ep_normalized_test` + `ep_full` 121 tests 通过
- [ ] 5.4 `SPEC_STORE_CMD=run` 全量 sass-spec 对比确认无回归
- [ ] 5.5 codegraph sync
