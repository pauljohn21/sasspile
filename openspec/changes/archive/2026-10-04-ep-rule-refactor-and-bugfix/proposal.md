# Proposal — EP Rule Refactor & Bugfix

## Why

`src/eval/rule.rs` 已膨胀至 **714 行**（超标 44%），且包含 **3 处重复的前缀检测+combine 逻辑**（`RuleBuilder::push`、`push_atroot_direct`、`nest_rule_in_children`）。该文件同时承载纯函数（`combine_selectors`、`nest_rule_in_children`）和状态机（`RuleBuilder`），违反单一职责原则。更关键的是，3 处重复逻辑中的 `starts_with_compound_prefix` 无法覆盖 descendant-prefix 场景（如 `.el-alert .el-alert__icon`），导致 **EP 有 6 个文件存在选择器加倍 bug**（`.el-alert .el-alert .el-alert__icon`）。

通过拆分重构 + 统一前缀检测逻辑，预期修复 EP Category A（6 files 选择器加倍）+ Category B-C（10 files 嵌套前缀/结构错误），EP 一致性从 **78/121 → 94+/121**，同时使每个源文件合规（≤ 350 行）。

## What Changes

### 重构（结构变化，行为等价）

1. **拆分纯函数到独立文件**：将 `combine_selectors`、`split_selectors_respecting_parens`、`starts_with_compound_prefix`、`resolve_ampersand_in_nodes`、`nest_rule_in_children` 提取为独立的纯函数集合文件
2. **分离状态机**：`RuleBuilder` 及所有方法保留在独立文件
3. **Evaluator 方法精简**：`eval_rule`、`selector_contains_ampersand` 保留，引用新模块的纯函数
4. **删除 2 处重复逻辑**：3 处 duplicate 调用统一到新 helper

### Bugfix（附带修复）

1. **新增统一前缀检测器** `has_descendant_prefix(parent, child)`：
   - 覆盖 `starts_with_compound_prefix` 的 compound-prefix 检测
   - **新增** descendant-prefix 检测（如 `.el-alert .el-alert__icon` vs `.el-alert .el-alert__icon` → 已包含前缀）
   - 替代 3 处独立的 `starts_with_compound_prefix()` 调用
2. **修复 `RuleBuilder::push` 内层 AtRootDirect 组合逻辑**：当前 `push_atroot_direct` 使用简化 combine，无法正确处理 `.el-button-group` 内的 `.el-button-group>.el-button` 结构

### 不修改项

- `eval_rule` 入口逻辑（仅调整引用）
- `build()` 中的 root_nodes hoisting 算法
- EP 管线产物（autoprefixer/lightningcss）处理策略
- sass-spec 预期通过率（确保不退化）

## Capabilities

无新 capability 或已有 spec 变更。本 change 是结构重构附带行为 bugfix，所修复的行为已在 `selector-ops-complete` 等已有 spec 中被描述为正确（如 parent reference 解析、compound selector combine）。设置 `skip_specs: true`。

## Impact

**受影响的源文件**：
| 文件 | 变化 | 最终行数 |
|------|------|----------|
| `src/eval/rule.rs` | 重命名/瘦身为 Evaluator 方法文件 | ~300 行 |
| `src/eval/rule_builder.rs` | 新建（RuleBuilder 状态机） | ~250 行 |
| `src/eval/selector_combine.rs` | 新建（纯函数集合） | ~180 行 |

**EP 预期影响（+16 files IDENTICAL）**：
- Category A（选择器加倍 6 files）：alert, button-group, input, input-otp, pagination, rate
- Category B-C（嵌套结构 10 files）：dialog, timeline-item, page-header, color-picker-panel, input-number, carousel, radio-button, step, popover, message-box

**sass-spec 风险**：低。纯函数拆分为等价变换，bugfix 仅影响 descendant-prefix 检测的边界情况。`selector_combine.rs` 将有单元测试覆盖。

**门控条件**：
- 核心测试 202/202 通过（compile/stage/ast/common/bs_spec/ep_full）
- sass-spec 全量通过率不退化（当前 ~8000/12133）
- EP classify_test DIFF 从 39 → ≤ 23（减少 ≥ 16 files）
