## Why

`sass-spec` 当前基线 7658/12133 (63.3%)，其中 `values/calculation/calc/error/known_incompatible/` 独占 227 个失败案例 — 这是单一类别中最大的失败集群（排除颜色）。

根因：`calc()` 内不兼容单位相加（如 `calc(1vmin + 1deg)`）**应产生编译错误**，但 sasspile 的 `simplify_op()` 在遇到不兼容 Add/Sub 时走 partial-simplification 路径返回 `Ok(Op)` 而非 `Err`，错误被静默吞掉，最终输出不报错的 CSS。

冲突点：partial-simplification（calc-css-boost 新增）对 `calc(3px * 2 + 1%)` 正确简化 `Mul` 子表达式，但副作用是吞掉了简单二元不兼容应产生的错误。需要在不破坏 partial-simplification 的前提下，让不兼容单位相加正确报错。

## What Changes

- **新增**：`simplify_calc` 入口 pre-check — 检测顶层 Add/Sub 二元不兼容单位运算，提前返回错误
- **保留**：现有 partial-simplification 逻辑不变（嵌套运算的单位不兼容仍保留 Op 节点）
- **行为变化**：`calc(1vmin + 1deg)` 等 ~227 个案例从不报错变为正确报错
- **不破坏**：`calc(3px * 2 + 1%)` 的 Mul 简化仍然正确执行

## Capabilities

### New Capabilities

- `calc-unit-incompat-error`: calc 表达式顶层 Add/Sub 检测到不兼容单位时产生编译错误（符合 Sass spec 的 known_incompatible 语义）

### Modified Capabilities

- `calc-simplification`: `simplify_calc` 增加 pre-check 步骤，但 `simplify_op` partial-simplification 逻辑不变；行为补充：纯二元不兼容不再被吞，但嵌套运算的不兼容仍走原有保留路径

## Impact

- **代码**：`src/eval/value/calc.rs`（simplify_calc 入口 + pre-check）、`src/eval/value/calc_simplify.rs`（可选：simplify_op 错误传播调整）
- **行为**：~227 个 known_incompatible 案例从 DIFF(不报错) → PASS(正确报错)
- **API**：`simplify_calc` 返回类型可能从 `Value` 改为 `Result<Value>`（需传播错误到 `eval_value`）
- **风险**：低 — pre-check 仅影响顶层纯二元不兼容运算，不触及 Mul/Div/嵌套场景
