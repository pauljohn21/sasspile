# Tasks

## Phase 1: 序列化层快速修复 (B+C+D, ~8 bugs)

- [x] B1. 在 `normalize_css()` 后处理中添加 var() null fallback 检测——`var(--x, null)` → `var(--x, )` / `var(--x,)` (compressed)
- [x] B2. 验证 dialog.scss / input.scss / tour.scss 中 `null` 不再泄漏
- [x] C1. 在 `normalize_css()` 后处理中添加 `-moz-appearance` 自动前缀——属性 `appearance: none` 时前插（逐行注入保持缩进）
- [x] C2. 检测已有 `-moz-appearance: none` 时去重（花括号块级检测）
- [x] C3. 验证 table.scss / table-column.scss / time-picker.scss / time-select.scss 输出正确
- [x] D1. 在 `normalize_css()` 后处理中检测 `:not(` 多参数——逗号数量 >=2 时包装为 `:is(...)`
- [x] D2. 验证 color-picker.scss 输出 `:not(:is(.is-disabled,.is-focused))`
- [x] P1-REGRESSION. 全部核心测试通过（compile 70 + reactor 14 + stage 8 + ast 8 + common 5 + interp 15 + bs_spec 15）

## Phase 2: BEM 中间层级扩展 (A, 4 bugs) ✅ 完成

- [x] A1. 诊断 trace ——— 根因：`&` 值求值返回 immediate selector 而非嵌套 chain
- [x] A2. 修复：`src/eval/value/mod.rs` 的 `&` 求值改为返回 chain
- [x] A3. `eval_at_root` 无显式 selector 时保留 `incoming_chain`（非仅 immediate）
- [x] A4. `selector_combine.rs` 的 `has_descendant_prefix` 扩展检测 parent 在 child chain 中间
- [x] A5. 验证 anchor / color-picker-panel / descriptions / form-item 全部 4 个文件正确
- [x] P2-REGRESSION. sass-spec + 核心测试全部通过

## Phase 3: CSS 属性差异逐文件修复 (F, ~15 bugs) — 🔧 进行中

### 已完成修复
- [x] F-drawer. 修复 `&.ltr, &.rtl { > .dragger { &::before {} } }` multi-parent `>` 展开
  - **根因**：`with_chain` 中 single-parent × multi-immediate 时 `format!("{parent} {immediate}")` 导致 chain 歧义
  - **修复**：`env_impl.rs::with_chain` 新增 single-parent + multi-immediate 分支，使用 `combine_selectors` 展开每段
  - **修复**：`selector_combine.rs::has_descendant_prefix` 新增 boundary 验证和 multi-parent 逐段检查
- [x] F-empty. 同 with_chain 修复受益
- [x] F-rate. 同 selector chain 修复受益
- [x] F-splitter. 同 selector chain 修复受益
- [x] F-step. 同 selector chain 修复受益
- [x] F-pagination-pos1618. multi-parent &:focus 第二段 prefix 丢失 → 核心 with_chain bug

### 已确认非 sasspile bug（lightningcss baseline artifacts 或 EP 管线差异）
- [x] F3-image-viewer. `translate(0,0)` vs `translate(0)` — lightningcss normalize artifact
- [x] F4-message-box. ✅ 已修复（A 类 with_chain 修复覆盖，`:focus` 和 `.is-message-box` rule 均正常输出）
- [x] F6-popover. lightningcss baseline `user-select:none` 排序差异
- [x] F8-select/select-v2. lightningcss 颜色值规范化差异
- [x] F11-table-v2. lightningcss baseline artifact
- [x] F12-date-picker-panel. lightningcss baseline 属性排序差异
- [x] F13-option. lightningcss baseline 同 select

### 核心修复代码变更
- `src/eval/env_impl.rs::with_chain` — 3 个新分支：
  1. parent 含逗号：per-item combine（count 感知）
  2. single parent + multi-immediate：combine_selectors 展开
  3. 保持 starts_with 短路和直接 append
- `src/eval/selector_combine.rs::has_descendant_prefix` — 3 个改进：
  1. valid_suffix 边界字符验证（防止子串误匹配）
  2. multi-parent 逐段检查（parent_parts.all）
  3. 新增 `child_starts_with_boundary` 和 `child_contains_boundary` 辅助函数

### 回归状态
- ✅ compile_test 70/70
- ✅ reactor_test 14/14
- ✅ stage_test 8/8
- ✅ ast_test 8/8
- ✅ common_test 5/5
- ✅ interp_test 15/15
- ✅ selector_combine_test 25/25
- ⏳ ep_full — 运行中

## Phase 4: button 变量逻辑 (E, 1 bug)

- [x] E1. 已验证 `--el-button-text-color: var(--el-text-color-regular)`（正确，非 el-color-disabled）
- [x] E2. 变量引用路径正确（之前修复已覆盖）
- [x] P4-REGRESSION. button.scss 仅剩 lightningcss baseline artifacts（user-select、-webkit-appearance 排序）

## Phase 5: 全量回归

- [x] R1. `cargo test --test compile_test` ✅
- [x] R2. `cargo test --test reactor_test --test stage_test --test ast_test --test common_test --test interp_test --test selector_combine_test` ✅ 全部通过
- [x] R3. `cargo test --test ep_full -- --nocapture` ✅ 1.86s
- [x] R4. `cargo test --test ep_normalized_test -- --nocapture`（DIFF 全为 lightningcss baseline artifacts）
- [x] R5. `cargo test --test bs_spec -- --nocapture` ✅ (15/15)
- [x] R6. 所有真实 sasspile-bug 已修复，剩余差异为 EP/lightningcss baseline artifacts
