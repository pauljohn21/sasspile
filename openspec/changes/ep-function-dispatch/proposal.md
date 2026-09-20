## Why

EP（element-plus）官方构建全量对比发现 **111/121 (92%) 文件与官方 dist 不一致**。根因是 sasspile 无法正确调用项目中定义的 **自定义 `@function`**（如 `joinVarName()`、`getCssVar()`、`bem()`），导致输出保留原始函数调用形式而非计算结果。这是 sasspile 进入企业级项目的**系统级障碍**。

## What Changes

- **修复 @function 调用分派链**：确保自定义函数在 `@use` / `@import` 后正确注册、解析和执行
- **修复 list 类型参数传递**：`joinVarName(('color', 'primary'))` 中 list 参数需正确拆分为多值
- **修复函数返回值的字符串化**：函数返回的 string 在插值位置 `#{}` 正确展开
- **添加对比回归测试**：`tests/ep_dist_test.rs` 作为企业级项目编译正确性基准

## Capabilities

### New Capabilities
- `function-dispatch-fix`: 自定义 @function 调用的完整分派机制修复
- `ep-dist-regression-test`: 基于 EP 官方 dist 的编译正确性回归测试

### Modified Capabilities
- 无（这是 bug 修复，不改变现有已通过的 spec 行为）

## Impact

- **受影响代码**: `src/eval/` 目录（call_user_function, bind_params, 函数查找逻辑）
- **API 变更**: 无公开 API 变更
- **性能影响**: 函数调用缓存提升
- **风险**: 修复可能影响 sass-spec 中依赖当前行为的 case；需全量回归确认
