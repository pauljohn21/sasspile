## Why

EP Element Plus 一致性验证中，40/121 文件存在真实 DIFF，其中 ~20 个文件的根因是同一类 bug：当 `e()`（element）mixin 在复合选择器（`&.block--mod` 或 `when(state)`）内部调用时，`@at-root { & { __elem { @content } } }` 中的 `&` 引用未正确展开为完整父选择器链，导致**选择器重复**（`.a--mod .a--mod .a__elem`）或**选择器缺失**（`.a__elem`，完全丢失 `.a--mod` 前缀）。

现有 `descriptions.scss` 修复（wrapper-skip）仅处理了 `e()` 直接子节点是包装器的场景，但未根本解决 `&` 引用链传播问题，导致 `checkbox`、`dialog`、`alert` 等 20 个文件仍然存在同一类 bug。

## What Changes

- **修复 `eval_at_root` 中 `&` 引用传播路径**：当 mixin 展开 `@at-root { #{$selector} { ... } }` 时，`$selector` 中的 `&` 必须展开为 mixin 调用时刻的完整父选择器链（含嵌套层累积的 modifier/state 类）
- **防止 AtRoot 输出被外层规则重复嵌套**：当 `e()` mixin 已生成包含完整父链的 AtRoot 输出时，外层 `nest_rule_in_children` 不应再次累加父前缀
- **修复影响范围**（EP normalized 一致性从 78/121 提升至目标 93+/121）：
  - selector doubling: alert, button-group, check-tag, checkbox, input-otp, input(textarea), pagination, popover, radio-button, rate, table-v2, message-box
  - selector missing: dialog, carousel, color-picker-panel, form-item, input-number, page-header, step, timeline-item
  - property 顺序（同根因）: empty, image-viewer, drawer

## Capabilities

### New Capabilities
- `bem-atroot-chain-resolution`: 当 `@at-root` 与 BEM e/m mixin 嵌套使用时，`&` 引用正确展开为完整父选择器链（包含 block + modifier + state 类的复合）

### Modified Capabilities
- 无（此 bug 修复未改变 sass-spec 已有行为，仅修复 EP BEM 模式兼容性）

## Impact

**受影响代码**：
- `src/eval/mixin.rs` — `eval_at_root` 方法，核心的 `&` 解析逻辑
- `src/eval/rule.rs` — `nest_rule_in_children` / `combine_selectors`（double-prefix 守卫）
- `src/eval/env.rs` / `env_impl.rs` — `selector_chain` 传播（如果需增强）

**受影响的 EP 文件**（一致性提升 +15 预期）：
- alert, button-group, carousel, checkbox, check-tag, color-picker-panel, dialog, drawer, empty, form-item, image-viewer, input, input-number, input-otp, message-box, page-header, pagination, popover, radio-button, rate, step, table-v2, timeline-item

**sass-spec 预期**：无回归（只修复 @at-root 语义，不影响 sass-spec 已有通过测试）

**风险**：中等。`eval_at_root` 是 @at-root 指令核心，修改需确保不破坏 `@at-root` 无 `@use` 的普通场景。
