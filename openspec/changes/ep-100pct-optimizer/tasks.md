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

## 阶段 3 — 诊断 & 管线特性标记 IN PROGRESS (2026-10-02)

- [x] 3.1 对所有 38 DIFF 文件进行逐文件 span 插桩诊断
- [x] 3.2 分类为 sasspile bug vs EP 管线特性
- [x] 3.3 确认 33 files 为管线特性（不可在 normalize 测试中修复）
- [x] 3.4 确认 5 files 为 sasspile bug：descriptions, 等

## 阶段 4 — Sasspile bug 修复（剩余）

- [ ] 4.1 修复 descriptions.scss: `e(title)` 在 `m($size)` 内 @at-root 上下文丢失
- [ ] 4.2 验证 descriptions + ep_normalized_test 效果
- [ ] 4.3 修复 rgba()/rgb() var() fallback 在 input-number 等文件中的展开
- [ ] 4.4 运行 `SPEC_STORE_CMD=run` 确认 sass-spec ≥ 7927 无回归
- [ ] 4.5 归档 openspec 到 `openspec/changes/archive/`

## 阶段 5 — 收敛 & 验证（最终）

- [ ] 5.1 针对所有已知管线差异文件，在 normalize post-process 中标记为 KNOWN_DIFF
- [ ] 5.2 软验证：actionable patch 全部应用后 EP 一致率达到可接受基线
- [ ] 5.3 全量 `ep_normalized_test` + `ep_full` 121 tests 通过
- [ ] 5.4 `SPEC_STORE_CMD=run` 全量 sass-spec 对比确认无回归
- [ ] 5.5 codegraph sync
