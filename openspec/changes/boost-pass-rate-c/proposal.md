## Why

sass-spec 通过率从 63.4% 因 `fmt_color_fn` 回归下降至 48.86%（缺少 CSS 颜色函数输出的闭合括号 `)`，影响 1770 个 case）。修复回归后，需要系统化、分阶段提升通过率，目标从 63.4% 逐步推进至 72-75%+。

## What Changes

- **紧急修复**: `display_color_spaces.rs` `fmt_color_fn` 函数格式字符串补上遗漏的 `)`，恢复 1770 个颜色相关 case
- **Phase 1 — 媒体查询/支持条件格式化**: 修复 `not(a)` → `not (a)`、`or(b)` → `or (b)` 等操作符后空格问题
- **Phase 2 — CSS 函数名规范化**: 修复 `TYPE(0)` → `type(0)`、`-a-calc( c)` vs `-a-calc(c)` 等 vendor 前缀函数和大小写问题
- **Phase 3 — calc() 简化**: 常量折叠（`calc(3px/2+1%)` → `calc(1.5px+1%)`）、嵌套展开（`calc((calc(...)))`）、一元负号规范化
- **Phase 4 — 选择器 + Meta 函数**: selector unify/extend/is_superselector 规范对齐；meta 内省函数（function_exists, variable_exists, module_mixins 等）
- **Phase 5 — CSS 嵌套展开**: 通过 `@use`/`@import` 模块的嵌套规则 `&` 展开、声明重排序
- **Phase 6 — 颜色系统深度修复**: adjust/change/scale 通道计算、色域映射精度、命名色舍入、NaN 传播

## Capabilities

### New Capabilities

- `media-query-formatting`: 媒体查询逻辑操作符（not/and/or）序列化时空格规范化
- `css-function-normalization`: CSS 函数名大小写规范化 + vendor 前缀函数注释空格处理
- `calc-simplification`: calc() 表达式常量折叠、嵌套展开、一元负号化简

### Modified Capabilities

- `color-serialization`: 修复 `fmt_color_fn` 输出格式字符串（补充缺失的闭合括号）
- `selector-operations`: selector unify/extend/is_superselector 算法规范对齐
- `meta-introspection`: meta 内省函数的正确行为（function_exists, variable_exists, module_functions 等）
- `css-nesting`: CSS 嵌套规则在模块间的展开和声明重排序

## Impact

- **主要影响文件**: `src/parse/ast/display_color_spaces.rs`, `src/css/media.rs`, `src/css/supports.rs`, `src/css/serializer.rs`, `src/eval/builtin/selector.rs`, `src/eval/builtin/selector_ops.rs`, `src/eval/builtin/meta.rs`, `src/eval/calc.rs`
- **测试影响**: 核心测试 202/202 必须维持通过；sass-spec 通过率提升
- **风险**: 低 — CSS 格式修复主要影响序列化层，不涉及解析/求值核心逻辑
