## Why

sasspile 当前 sass-spec 通过率为 2822/5362 = 52%。上一次 spec-pass-rate-boost 变更（已归档）完成了参数验证修复、meta 模块功能、error 检测和 values/css 深度修复。本轮聚焦剩余高频失败模式：命名空间函数映射缺失（~150 失败）、plain CSS 错误检测不完整（~120 失败）、@forward 冲突检测缺失（~60 失败）、expected_error_but_ok 模式（~200 失败）和 math 函数参数验证增强（~150 失败）。预计可提升至 62-65%。

## What Changes

- 补全 `module_dispatch.rs` 中缺失的命名空间函数映射（`map.map-*`, `string.str-*`, `color.lighten/darken/ie-hex-str`, `selector.selector-*`, `math.unitless` 等）
- 完善 plain CSS 模式错误检测：`@use`/`@forward`/`@include`/`@function`/`@mixin` 等 at-rule 在 `.css` 文件中应报错；interpolation、operators、parent selector suffix 在 plain CSS 中应报错
- 实现 `@forward` 冲突检测：两个 forwarded 模块定义同名 variable/function/mixin 时应报错
- 增强 expected_error 检测覆盖：对参数数量/类型错误的函数调用，在 plain CSS 模式下也正确报错
- 补全 math 函数参数验证：`clamp`/`min`/`max`/`hypot`/`pow`/`log` 等的参数数量、类型、单位检查
- 实现 `color.ie-hex-str` 函数
- 实现 `meta.load-css` mixin 和 `meta.apply` 函数（基础版）

## Capabilities

### New Capabilities

- `namespace-dispatch`: 命名空间函数映射补全——覆盖所有 sass 内建模块的 `module.function` 形式调用
- `plain-css-errors`: plain CSS 模式错误检测——`.css` 文件中的 Sass 语法应正确报错
- `forward-conflict-detection`: @forward 冲突检测——多个 forwarded 模块定义同名成员时报错
- `math-param-validation`: math 函数参数验证增强——clamp/min/max/hypot/pow/log 参数数量、类型、单位检查

### Modified Capabilities

- `param-validation-fix`: 增强参数验证覆盖——补全更多函数的参数数量和类型检查
- `error-detection-coverage`: 扩展 expected_error 检测——覆盖 plain CSS 模式下的错误场景
- `meta-module-functions`: 新增 meta.load-css mixin 和 meta.apply 函数基础实现

## Impact

- `src/eval/module_dispatch.rs`: 新增 ~30 行命名空间映射
- `src/eval/builtin.rs`: 新增 `color.ie-hex-str`、`meta.load-css`、`meta.apply` 分支
- `src/eval/builtin/math.rs`: 增强参数验证逻辑
- `src/eval/mod.rs` / `src/eval/module.rs`: @forward 冲突检测逻辑
- `src/eval/import.rs` 或新文件: plain CSS 错误检测
- `tests/`: 无新增测试文件（通过 sass-spec 验证）
