## Why

sass-spec `values` 目录通过率仅 45%（533/1169），`css` 目录仅 52%（419/830），合计 ~1011 个失败用例。根因分析发现 7 个核心缺陷，其中 calc 复合单位简化错误（占 ~530 失败）和 plain CSS 错误检测不完整（占 ~200 失败）合计占总失败的 72%。修复后预计通过率提升至 values ~95%、css ~90%。

## What Changes

- **修复 calc 复合单位简化**：不兼容单位（如 `1px * 1rad`）不应被简化为数值，应保留 `calc()` 表达式
- **修复 infinity/NaN 单位丢失**：`math.div(1px * 1em, 0)` 应输出 `calc(infinity * 1px * 1em)`，保留所有单位
- **修复 +0/-0 模运算**：`+0 % +1` 应输出 `0`，不应将 `%` 当作字符串拼接
- **修复 slash 除法执行**：非 calc 上下文中的 `/` 应执行除法（如 `1/2` → `0.5`），符合已弃用但仍生效的 SCSS `/` 除法语义
- **完善 plain CSS 错误检测**：`.css` 文件中 `$var`、`#{}`、`&` 等 Sass 特性应报错
- **完善 CSS 数学函数简化**：`min()`/`max()`/`clamp()`/`round()`/`mod()`/`rem()` 在参数为纯数值时应简化
- **修复选择器规范化差异**：`:is()`、`:has()`、参考组合器等输出格式对齐 spec

## Capabilities

### New Capabilities

- `css-math-functions`: CSS 数学函数（min/max/clamp/round/mod/rem）的简化规则——当参数全部为兼容数值时简化为单个值，否则保留函数调用

### Modified Capabilities

- `calc-simplification`: 修改 calc 简化器的单位兼容性检查逻辑——不兼容单位（px*rad, px/em 等）必须保留 calc() 表达式不做数值运算
- `calc-infinity-handling`: 修改除零时的单位保留逻辑——infinity/NaN 结果必须携带所有输入的单位
- `plain-css-errors`: 扩展 plain CSS 模式的错误检测覆盖——$var、#{}、&、@if 等在 .css 文件中必须报错

## Impact

- `src/eval/value/calc_simplify.rs` — 核心修改：单位兼容性检查 + CSS 数学函数简化
- `src/eval/value/calc_ast.rs` — calc AST 中单位运算的表示
- `src/eval/value/ops.rs` — 除零/infinity 单位保留 + modulo 的 +0/-0 处理
- `src/eval/value/mod.rs` — slash 除法语义
- `src/eval/plain_css.rs` — plain CSS 错误检测扩展
- `src/css/selector.rs` — 选择器规范化
- `src/lex/mod.rs` — `+0`/`-0` 词法分析修复
- `tests/sass_spec_full.rs` — 已添加 .sass 跳过
- `tests/spec_manifest.rs` — 已添加 .sass 跳过文档
