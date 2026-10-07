# Proposal: Bootstrap Dist Compatibility

## Why

当前编译器将 Bootstrap 全量 SCSS 编译通过（不栈溢出，0.08s），但输出仅覆盖 dist 参考的 **~19%**（2324 行 vs 12048 行）。大量组件、工具类、CSS Custom Properties 完全缺失。响应式架构解决了"能不能跑完"，现在需要解决"能不能生成对的东西"——以 Bootstrap 官方 dist CSS 为**黄金参考**，系统性补全 SCSS 特性覆盖度。

## What Changes

- **新增能力**: Bootstrap dist 对照测试基础设施（自动化 diff 工具 + CI 门控）
- **补全内建函数**: `shade-color()`, `tint-color()`, `to-rgb()`, `map-keys()`, `map-values()`, `map-get()` 深路径, `str-replace()`, `list-separator()`
- **修复 @each/@for 迭代**: map 键值对展开、`$key, $value` 双变量迭代、range 生成
- **修复 CSS Custom Properties**: `--bs-*` 变量生成、`var()` fallback 链、RGB comma-separated 值
- **修复 Utility API**: `$utilities` map 迭代生成响应式工具类（spacing, display, flex, sizing）
- **修复颜色函数 + CSS var**: `rgba()` 接受 CSS var、颜色的 `red()/green()/blue()/alpha()` 通道函数
- **修复 @mixin 复杂参数**: 默认值表达式、keyword arguments、可变参数 `@content`
- **修复字符串处理**: escape、`url()` 内插值、SVG data-uri 引号保留
- **修复 @include + @content**: mixin body 内的 `@content` 占位符替换
- **修复 `@use`/`@forward`**: 模块系统命名空间访问、`@import` 向后兼容

## Capabilities

### New Capabilities

- `bootstrap-dist-alignment`: 与 Bootstrap 5.3.x dist CSS 的全量对齐能力。涵盖 CSS Custom Properties 生成、Utility API map 迭代、颜色 SHMIX 函数、@content 替换等。通过 `bootstrap.css`（280KB, 12048 行）逐行比对验证。

### Modified Capabilities

- `builtin-modules`: 补全 color (shade-color/tint-color/to-rgb/red/green/blue)、map (keys/values/deep-get)、list (separator/join/zip)、string (replace/index/length/slice)、math (clamp/abs/round) 函数
- `directive-ops`: 修复 `@each` map 双变量迭代、`@for` range、`@mixin` + `@content` 替换语义、`@include` keyword args

## Impact

- **代码**: `src/eval/` (内建函数 + 指令展开)、`src/runtime/` (EvalContext 能力)、`src/serialize/` (CSS var 序列化)
- **测试**: `tests/integration_test.rs` (新增对照测试)、`tests/bootstrap_test.rs` (从 ignored 变为可运行)
- **依赖**: 无新增外部依赖（纯特性补全）
- **行为**: 编译产物从"能通过简单 case"升级为"Bootstrap 5.3.x dist 逐行对照通过"
