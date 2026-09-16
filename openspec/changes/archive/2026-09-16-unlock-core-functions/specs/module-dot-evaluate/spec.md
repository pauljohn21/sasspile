## ADDED Requirements

### Requirement: substitute_vars 识别点分函数调用路径

`substitute_vars` SHALL 在 alphanumeric 标识符扫描中将 `.` 视作路径分隔符的一部分，形成完整的点分标识符（如 `math.abs`、`color.alpha`）。

#### Scenario: math.abs 值替换后输出正确值
- **WHEN** 输入为 `@use "sass:math"; a {b: math.abs(-5px)}`
- **THEN** 输出包含 `a {b: 5px}` 且不含 `math.` 字面量泄漏

#### Scenario: color.alpha 点分调用
- **WHEN** 输入为 `@use "sass:color"; a {b: color.alpha(red)}`
- **THEN** 输出包含 `a {b: 1}` 且不含 `color.` 字面量泄漏

### Requirement: 非函数调用的点分标识符 fallback 保持不变

`substitute_vars` SHALL 在 absorb 点分路径后、下一个字符不是 `(` 时，将完整标识符（含 `.`）原样输出。

#### Scenario: CSS 类选择器不触发函数 dispatch
- **WHEN** substitute_vars 处理 selector `.container-fluid`
- **THEN** 输出仍保留 `.container-fluid` 不变

#### Scenario: 属性值中的点分非函数字符串
- **WHEN** 输入为 `a {background: url(foo.bar.png)}`
- **THEN** 输出中 `url(foo.bar.png)` 被正确保留

#### Scenario: 数值中的小数点
- **WHEN** 输入为 `a {width: 1.5px}`
- **THEN** 输出保留 `1.5px` 不变

### Requirement: 后缀路径提取正确

当点分标识符后接 `(` 时，SHALL 从最后一个 `.` 之后提取函数名 dispatch 到 `eval_builtin`。

#### Scenario: map-get 类双段点分
- **WHEN** 输入为 `$map: (a: 1); a {b: map.get($map, a)}`
- **THEN** 输出包含 `a {b: 1}`（此场景在没有 absorb 的情况下若 ident="map" 的情况不适用；但应支持自定义函数的双段点分调用）

#### Scenario: 单段函数调用不变
- **WHEN** 输入为 `a {b: abs(-5px)}`
- **THEN** 输出包含 `a {b: 5px}`（已有行为不受本次修复影响）
