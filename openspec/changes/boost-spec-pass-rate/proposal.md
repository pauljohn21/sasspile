## Why

sass-spec 当前通过率仅 0.4%（37/10274），主要瓶颈集中在三大块：(1) Mixin 变量替换完全失效导致所有依赖 mixin 的 case 失败；(2) core_functions（占总量 76%）大量内置函数虽有框架注册但实现空漏；(3) args 解析存在单/多参数歧义（如 `"1 2"` 未拆分）。本 change 将一次性修复 mixin 展开 + 补全 color/list/string/map 内置函数 + 修 args 拆分，预计将 sass-spec 通过率从 0.4% 提升到 10%+。

## What Changes

- **修 mixin 变量替换**：`evaluate_node_with_locals` 将 mixin 参数注入 `global_variables` 的方式导致同全局污染与递归展开问题。改为独立的参数作用域链，展开时局部变量优先于全局变量
- **修 args 解析**：`MixinCall` 的 args 字段 `"1 2"` 未按空格/逗号拆分为多参数，需要在前置 collect_args 阶段做正确 split（尊重括号嵌套与引号）
- **补全 color 内置**：`darken`/`lighten`/`mix`/`rgb`/`rgba`/`hsl`/`hsla`/`invert`/`grayscale`/`alpha` — 已有 parse_hex_color 基础，补齐 HSL 转换通道 + rgba 输出格式
- **补全 list/string/map 内置**：`len`/`nth`/`append`/`join`/`index`/`quote`/`unquote`/`str-length`/`str-index`/`map-get`/`map-has-key`/`map-keys`/`map-values`/`map-merge` 已有骨架，完善边界条件（1-based 索引、空 list 处理、嵌套括号）
- **roserr/custom 项目**：`getCssVar`/`getCssVarName` 不变，保持企业验收不退化

## Capabilities

### New Capabilities
- `mixin-variable-scoping`: Mixin 参数通过独立局部作用域链求值，不影响全局变量，支持嵌套展开与默认值
- `color-functions`: 完整色彩操作通道（darken/lighten/mix/rgba/hsl/hsla 等），保持现有 rgb_to_string 输出约定
- `builtin-polish`: List/String/Map 内置函数的参数歧义消除与边界处理

### Modified Capabilities
无（纯功能补全，不修改现有 spec 约束）。

## Impact

- **受影响代码**: `src/evaluate_dst/eval_ctx.rs`（mixin 展开）、`src/evaluate_dst/mod.rs`（substitute_vars + split_args）、`src/evaluate_dst/builtins.rs`（color 函数）
- **不破坏**: Bootstrap/Element Plus 已通过路径不退化（tap 观测 + tracker 验证）
- **测试**: `tests/builtins_basic.rs` 增加用例，`tests/sass_spec.rs` 预计通过率提升
- **单文件约束**: 保持 ≤ 500 行，builtins.rs 若因 color 代码膨胀需拆分为 color.rs
