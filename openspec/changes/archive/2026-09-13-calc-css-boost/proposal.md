## Why

`sass-spec` 当前通过率 7647/12131 (63%)，其中 `values/calculation` 区域以 **39% 通过率（596 个失败）** 成为单一最大失败池。诊断发现三个明确的根因：(1) `MATH_NAMES` 注册表缺失 5 个 CSS 全局数学函数条目导致函数不被分派求值；(2) `calc()` 表达式简化逻辑缺少乘除/符号/常量折叠；(3) `max()`/`min()` 单位转换存在 unitless/unknown-unit 边界错误。此变更是将 7647 → ~7710 (+63) 的关键路径。

## What Changes

- **修复 1**：在 `dispatch.rs` 的 `MATH_NAMES` 中补全 `exp`、`sign`、`hypot`、`atan2`、`log` 条目的 CSS 全局名映射（使其作为 CSS 函数可以直接全局调用并求值）
- **修复 2**：扩展 `calc_simplify.rs` 的 `simplify_binary` 支持乘法/除法常量折叠（`3px * 2` → `6px`）、符号反转（`1% - -1px` → `1% + 1px`）、特殊常量吸收（`infinity * n` → `infinity`）、数学常量求值（`e * 2` → 数字）
- **修复 3**：修正 `math.rs` 中 `min`/`max` 的单位转换逻辑：当 unitless 为最值时返回 unitless、对 unknown unit 编译期求值而非直接透传

## Capabilities

### New Capabilities

- `css-math-exp-sign`: exp() 和 sign() 函数作为 CSS 全局函数可求值
- `css-math-hypot-atan2-log`: hypot()、atan2()、log() 函数作为 CSS 全局函数可求值
- `calc-simplify-enhanced`: calc() 表达式支持乘除常量折叠、符号反转、特殊常量吸收、数学常量求值
- `min-max-unitless-fix`: min()/max() 正确处理 unitless 最值和 unknown unit 编译期求值

### Modified Capabilities

- 无（以上均为新建能力，不修改已有 spec）

## Impact

- **受影响文件**：`src/eval/builtin/dispatch.rs`（注册表）、`src/eval/value/calc_simplify.rs`（简化器）、`src/eval/builtin/math.rs`（min/max 逻辑）
- **API/ABI**：不影响公开 API，纯内部求值增强
- **性能影响**：calc 简化略有增加但常数级
- **测试影响**：预估 +63 cases (7647 → ~7710)，`values/calculation` 从 39% → ~50%
