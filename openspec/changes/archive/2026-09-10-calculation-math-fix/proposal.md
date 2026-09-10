## Why

`sass-spec` 中 `values/calculation` 目录有 553 个失败 cases（通过率仅 44%），是当前最大的非 color 失败集中区域。诊断发现这些失败集中在 5 个系统性根因，修复后预期可通过 +90~150 cases，将通过率从 61% 提升至 62%+。

## What Changes

- **calc() 特殊常量格式化**：规范化 `infinity`/`NaN`/`MinusInfinity` 的 calc 输出格式（大小写、`1/0`→`infinity` 简化、`calc(infinity*N)`→`calc(infinity)`）
- **extract_unitless 扩展**：接受 `infinity`/`-infinity`/`NaN` 作为合法数值输入，使所有 math 函数能处理特殊浮点值
- **calc-size() 保留逻辑修正**：修复输出格式（`80pxsize` → 正确表达式），正确处理 `add()` 包装
- **atan/asin/acos/sass_script**：当参数含 Sass 变量时保留函数形式而不编译时求值
- **type-of(calc(special)) 修正**：`calc(infinity)` 的 type-of 应为 `number` 而非 `calculation`

## Capabilities

### New Capabilities

- `calc-special-constants`: calc() 表达式中特殊常量（infinity、NaN、-E∞）的语法规范化与简化规则
- `calc-size-implementation`: calc-size() 函数的参数解析、验证和输出格式化
- `math-special-float`: math 函数（pow/sqrt/log/asin/acos/atan/round 等）接受 infinity/NaN 特殊值的行为

### Modified Capabilities

- `math-functions`: `extract_unitless` 函数的行为扩展，原仅接受有限浮点数，现需接受特殊浮点常量

## Impact

- 受影响文件：`src/eval/builtin/math.rs`、`src/eval/builtin/math_trig.rs`、`src/eval/value/calc.rs`、`src/eval/value/calc_simplify.rs`、`src/eval/plain_css.rs`
- 无 API 变更，无破坏性修改
- 可能间接修复部分 `core_functions/math`（72 fails）和 `values/numbers`（41 fails）的关联 case
