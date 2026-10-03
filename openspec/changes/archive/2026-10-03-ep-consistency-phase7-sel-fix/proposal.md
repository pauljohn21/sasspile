## Why

EP (Element Plus) CSS 一致性基线当前为 83/121 IDENTICAL (68.6%)。剩余的 38 个 DIFF 文件中，22 个属于 EP 管线产物（autoprefixer / lightningcss），无法通过修复 sasspile 解决。**16 个文件存在真正的 sasspile bug**，集中在选择器展开（`&` 引用）、重复 keyframes 声明、BEM 嵌套链缺失等类别。通过精确修复这些 bug，EP 一致性预期可提升至 99/121 (81.8%) 或 100/121 (82.6%)。

选择器类 bug 是最高价值的修复目标——它们影响 8 个文件，且根因清晰（`&` 展开时机与上下文传播不一致）。

## What Changes

按优先级分阶段修复 3 类选择器 bug：

1. **`&` 展开失败** (step, rate)：`@at-root` 内部 mixin 生成的 AtRootDirect 节点在特定上下文中未展开字面 `&`，导致输出 CSS 包含非法字符或选择器重复
2. **选择器重复嵌套** (anchor, popover, table-v2, rate)：`combine_selectors` 或 `nest_rule_in_children` 在嵌套 `&` 引用时，未检测子选择器已包含父选择器前缀，导致 `.el-anchor .el-anchor.el-anchor--vertical` 之类的重复
3. **BEM 嵌套链中间层缺失** (descriptions, color-picker-panel)：`e()` mixin 在 `m()` 或伪类内嵌套时，缺少一级父选择器层级

同时可能需要：
- 新建辅助函数 `selector_already_contains_prefix(parent, child)` 检测前缀包含关系
- 修改 `combine_selectors` 内的去重逻辑

不改动项：
- EP 管线产物（22 files）保持差异，作为 KNOWN_DIFF 接受
- 颜色系统（color_tests）保持跳过
- sass-spec 验证为门控条件

## Capabilities

### New Capabilities

-none-（本变更不引入新能力，仅修复既有实现的正确性）

### Modified Capabilities

- `eval-rule`: `combine_selectors` / `nest_rule_in_children` 添加前缀去重逻辑
- `eval-mixin`: `exec_mixin` 出口处 AtRootDirect 包装时确保 `&` 字面量被展开
- `eval-rule`: `push_atroot_direct` 中 `&` 字面量强制展开为父选择器

## Impact

- `src/eval/rule.rs`：`combine_selectors`、`nest_rule_in_children`、`push_atroot_direct` 三处修改
- `src/eval/mixin.rs`：`exec_mixin` 出口 AtRootDirect 包装时展开 `&`
- 影响范围：EP 的 16 files bug；不影响 sass-spec 基线 (3097/5362)
- 门控测试：`ep_normalized_test` 必须通过 + 核心测试 202/202
- 文件行数预估：rule.rs +20 行、mixin.rs +10 行
