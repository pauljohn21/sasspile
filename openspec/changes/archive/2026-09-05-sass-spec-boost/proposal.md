## Why

sasspile 当前 sass-spec 通过率 6205/11824 = 52.5%，仍有 5619 个测试失败。颜色函数目录（core_functions/color）占全部失败的 61%（3419/5619），其中 `color/to_space`（1637 失败）、`color/scale`（238）、`color/change`（228）、`color/adjust`（201）是最大失败来源。本次变更系统性地修复颜色系统 + directives 高频失败区域，目标将从 52.5% 提升至 60%+。

## What Changes

- **color/to_space**：修复 CSS Color 4 色域转换精度（sRGB↔display-p3↔lab↔oklab↔lch↔oklch↔xyz），支持 `color()` 函数解析和序列化
- **color/scale**：修复 `color-scale()` 通道边界行为和 HSL/现代色彩空间逻辑
- **color/change**：修复 `color-change()` 参数校验、现代色彩空间通道设置
- **color/adjust**：修复 `color-adjust()` 增量调整的精度
- **color/mix**：修复 `mix()` 权重和 alpha 通道处理
- **color/hsl**：修复 HSL 空间函数的输出格式和精度
- **directives/import**：修复 `@import` 的源映射和嵌套导入行为
- **directives/function**：修复 `@function` 返回值类型和空值处理

## Capabilities

### New Capabilities
- `color-to-space`：CSS Color 4 色彩空间转换（color() 函数 + to-space 序列化）
- `color-scale-fix`：color-scale 通道边界与现代色彩空间逻辑
- `color-change-fix`：color-change 参数校验与现代空间通道
- `color-adjacent-fix`：color-adjust/mix/hsl 输出精度

### Modified Capabilities
- `color-system`：扩展 scale/change/adjust/mix 的现代色彩空间支持
- `module-system`：修复 @import 嵌套和源映射行为

## Impact

- **src/eval/color_space.rs**：色域转换矩阵和 clip 算法
- **src/eval/color_adjust.rs**：scale/change/adjust 核心逻辑
- **src/eval/builtin/color.rs**：内建函数分派
- **src/eval/builtin/color_space.rs**：color() 函数实现
- **src/css/serializer.rs**：color() 输出格式
- **src/parse/import.rs**：@import 解析
- **依赖**：color crate v0.3 色彩空间转换参考
