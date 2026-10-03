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

## 阶段 3 — at-root 上下文：已回退 ⚠️ REVERTED (2026-10-03)

- Phase 3 方案 (at_root_top flag) 净效应为负：修复 descriptions.css 但破坏 12 其他文件
- 8dcd5bb (Phase 3 commit) + 206752c (revert) 都已保留在 history
- True baseline 回退到 83/121 (8dcd5bb 之前的状态)
- descriptions.css 的修复需要更精确的方案
- 3.1-3.11 在 commit 8dcd5bb 中完成，commit 206752c 中回退

## 阶段 4 — 精确 DIFF 分类（已完成） ✅ (2026-10-03)

- [x] 4.1 创建 ep_diff_analyzer，精确分类 38 个 DIFF 文件的根因
- [x] 4.2 确认 EP 管线产物 22 files (autoprefixer/lightningcss) — 不可修复
- [x] 4.3 识别 16 files 真正的 sasspile bug，按以下子分类：
  - sel-doubling (3): anchor, popover, table-v2 — `&` 引用被重复嵌套
  - bem-nesting / sel-missing (2): descriptions, color-picker-panel — 缺少中间层
  - step-ampersand (1): step — `&` 在 @at-root 上下文中未展开为字面选择器
  - dup-keyframes (3): dialog, drawer, message-box — keyframes 重复声明
  - select-grouping (2): input-number, pagination — comma-grouping 未拆分
  - rate-fv (1): rate — `.el-rate .el-rate:focus-visible` 重复
  - not-vs-is (1): color-picker — `:not()` vs `:is()` 格式
  - svg-replace (3): option, select, select-v2 — EP dist 用 border-trick 替代 SVG mask
- [x] 4.4 建立 DIFF 分类决策矩阵（EP管线产物 22 + sasspile bug 16 = 38 DIFF）

## 阶段 5 — 精确 Bug 修复（按优先级）

- [ ] 5.1 sel-doubling 修复：anchor, popover, table-v2
  - 根因：` nest_rule_in_children` 或 `push_atroot_direct` 中 propagate 父选择器时重复
  - fix 方向：检测 child 是否已以 parent 开头，不重复嵌套
- [ ] 5.2 step-ampersand 修复：step
  - `&` 在 `pseudo()` mixin 内被 @at-root 包装为 AtRootDirect 时未展开
  - fix 方向：push_atroot_direct 中 combine 时强制展开 `&`
- [ ] 5.3 rate-fv 修复：rate
  - 根因类似 sel-doubling：focus-visible 内 e(item) 嵌套时 `&` 被重复传播
- [ ] 5.4 bem-nesting 修复：descriptions, color-picker-panel
  - 需要精确方案避免 Phase 3 的 10-file 回归
- [ ] 5.5 dup-keyframes 修复：dialog, drawer, message-box
  - keyframes 去重或避免重复 @extend 传播
- [ ] 5.6 每次修复后跑 `ep_normalized_test` + 核心测试 202/202
- [ ] 5.7 每次修复后跑 `SPEC_STORE_CMD=run` 确认 sass-spec 无回归
- [ ] 5.8 归档到 `openspec/changes/archive/`
