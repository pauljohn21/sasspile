# Proposal

## Why

Element Plus 剩余 ~28 个编译失败中，约 14 个源于同一根因：`sasspile` 的选择器嵌套层级丢失。EP 的 `#{& + '-selfdefine'}` 模式生成的嵌套规则（应输出 `.el-popper .el-popper-selfdefine`）被错误输出为平坦的 `.el-popper-selfdefine`，丢失了父选择器前缀。根因位于 `has_descendant_prefix` / `starts_with_compound_prefix` 中单 `-` 被误判为 BEM compound 分隔符。

## What Changes

- 修改 `starts_with_compound_prefix`：仅 `--` 和 `__` 视为 BEM compound 分隔符，单 `-` 和单 `_` 不再触发 compound 判定
- 新增 `eval_rule` 插桩跟踪：在 RuleBuilder 接收 child Rule 时输出 `selector_combine` 决策 trace
- 修复 EP 14 个选择器嵌套失败文件：dropdown、menu、message、table、tag、timeline、transfer、tree、upload、popover、switch、form-item、message-box、overlay

## Capabilities

### New Capabilities

- `scss-eval/selector-nesting-chain`: SCSS 嵌套规则选择器组合行为——mixin 输出在父上下文中的正确 descendant combine、interpolation `#{& + '-suffix'}` 的正确嵌套、BEM compound 前缀检测的边界修正

### Modified Capabilities

（无现有 spec 修改——bug fix 使行为符合已有 spec）

## Impact

- 受影响代码：`src/eval/selector_combine.rs`（`starts_with_compound_prefix`）
- 受影响测试：`tests/ep_classify_test.rs`（EP 分类诊断）
- 预计 EP 失败：28 → 14（修复 14 个选择器嵌套文件）
- 风险：sass-spec 含 `--` modifier 的测试需确保回归不发生
