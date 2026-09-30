## Why

EP（Element Plus）一致性基线为 74/121（61.2%），有待从 LightningCSS 规范化对比中暴露的语义差异逐一修复，使 EP 组件库编译输出与官方输出 100% 一致。现有 sass-spec 核心能力已完备（202/202 通过），剩余 DIFF 集中于少量特定模式：keyframe 空块保留、模块变量嵌套解析、extend 跨模块传播范围、伪元素格式规范化、GetCssVar/map.get 等函数求值。**当前是集中攻克这些语义差异的最佳时机**——工具链（lightningcss 规范化、diagnostic_runner）已就绪，各根因已通过探索分析定位。

## What Changes

- **keyframe-empty-preservation**: 修复 LightningCSS 规范化后空 `0%`/`100%` keyframe 块被移除的问题。当前 ep_normalized_test 运行 lightningcss 后空块消失导致与 EP 官方输出不一致。
  - 方案：在 normalize 阶段检测并保留仅含 `0%`/`100%` 选择器但内容为空的 keyframe 块（通过 padding 占位或跳过规范化），确保 EP 官方输出中存在的空块被保留。
- **module-var-nested-resolution**: 修复模块变量在嵌套 `@include` 中的可见性问题。当前 `@use` 引入的变量通过 `@include` 传递到深层嵌套规则时可能丢失绑定。
  - 方案：检查 Env 的 scope chain 构建逻辑，确保模块跨越 `@include` 调用时变量查找正确穿透。
- **extend-across-modules-scope**: 修复 `@extend` 在跨模块场景下传播范围不一致的问题。当前 extend 的选择器注入在 `@use` 边界的行为与 EP 编译结果不完全匹配。
  - 方案：审视 extend 的 selector injection 逻辑，确认 `@use` 引入的占位符和选择在 extend 时正确注入到目标规则集。
- **pseudo-element-colon-normalize**: 修复伪元素（`::before`/`::after`/`::placeholder`）在序列化时单双冒号格式规范化。当前部分输出保留了单冒号（CSS2 格式），而 EP 官方使用双冒号（CSS3 格式）。
  - 方案：在序列化阶段对已知伪元素名强制输出双冒号格式。
- **function-eval-bugs**: 修复 `getCssVar()`、`map.get()` 等函数在特定上下文未被正确求值的 bug。
  - 方案：逐一排查这些函数的实现，通过 tracing 插桩确认求值路径中的引用/解引用缺失。

## Capabilities

### New Capabilities

_无新增能力——本次变更集中于修复现有实现的语义偏差，不引入新功能。_

### Modified Capabilities

- `extend-across-modules`: 修改跨模块 extend 的传播范围规则，使 `@use` 引入的选择器在 extend 时正确注入到目标规则。
- `module-functions`: 修改模块变量在嵌套 `@include` 调用中的可见性规则。
- `css-serializer`: 修改序列化阶段对伪元素冒号的规范化规则。

## Impact

- **代码范围**: `src/css/serializer.rs`（伪元素格式化）、`src/directive/ops.rs`（extend 传播）、`src/eval/`（函数求值）、`tests/ep_normalized_test.rs`（规范化逻辑）
- **测试影响**: ep_normalized_test 目标从 74/121 → 121/121
- **API 影响**: 无外部 API 变更，纯内部语义修正
- **依赖影响**: 无新增依赖
