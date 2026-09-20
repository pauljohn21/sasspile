# css-math-dispatch Specification

## Purpose
定义 CSS 原生数学函数（round/clamp/rem/mod/min/max）在 eval 阶段的独立分派行为——这些函数不应进入 Sass 简化管线，而应保持原样传递到 serializer。

## ADDED Requirements

### Requirement: CSS 原生 math 函数识别
系统 SHALL 在 eval 分派时识别以下 CSS 原生数学函数并标记为 `CssNativeFunc`：`round`, `clamp`, `rem`, `mod`, `min`, `max`, `var`, `calc`, `element`, `expression`。

#### Scenario: CSS round 识别
- **WHEN** eval 遇到 `round(117, 25)` 调用
- **THEN** 标记为 CssNativeFunc，不进入 Sass math.round 分派

#### Scenario: CSS clamp 识别
- **WHEN** eval 遇到 `clamp(1px, 2px, 3px)` 调用
- **THEN** 标记为 CssNativeFunc，跳过 Sass 简化

### Requirement: CSS math 函数绕过简化
被标记为 `CssNativeFunc` 的函数调用 SHALL 跳过 calc_simplification 管线的以下步骤：常量折叠、运算符简化、单位换算。

#### Scenario: var fallback 保留
- **WHEN** 表达式 `var(--c, 1 + 2)` 进入 eval
- **THEN** fallback `1 + 2` 保留不简化为 `3`

#### Scenario: clamp 单位混合报错
- **WHEN** 表达式 `clamp(1px, 2em, 3vw)` 包含不兼容单位
- **THEN** eval 阶段不报错（保留到 CSS 输出），serializer 原样输出

### Requirement: rem/mod CSS 语义
系统 SHALL 实现 CSS `rem(a, b)` 和 `mod(a, b)` 函数，遵循 CSS 值与单位 Level 4 规范。

#### Scenario: rem 向零取余
- **WHEN** 计算 `rem(-7, 7)`
- **THEN** 结果为 `0`（-7 - 7 * trunc(-1) = -7 + 7 = 0）

#### Scenario: mod 向负无穷取余
- **WHEN** 计算 `mod(-7, 7)`
- **THEN** 结果为 `0`（-7 - 7 * floor(-1) = -7 + 7 = 0）

#### Scenario: rem 无穷特殊行为
- **WHEN** 计算 `rem(5, infinity)`
- **THEN** 结果为 `5`

#### Scenario: rem 负数无穷
- **WHEN** 计算 `rem(5, -infinity)`
- **THEN** 结果为 `5`

#### Scenario: rem 零被除数倒数
- **WHEN** 计算 `math.div(1, rem(-7, 7))`（rem(-7,7) = 0）
- **THEN** 结果为 `calc(infinity)` 或 `calc(-infinity)` 取决于符号

### Requirement: min/max CSS 输出
系统 SHALL 支持 CSS `min()` 和 `max()` 函数的 3+ 参数形式，输出保留 CSS 原生函数形式。

#### Scenario: max 多参数
- **WHEN** 表达式 `max(1, 2, 3)` 在 CSS 上下文中
- **THEN** 输出 `max(1, 2, 3)` 而非简化为 `3`

#### Scenario: min 多参数
- **WHEN** 表达式 `min(1, 2, 3)` 在 CSS 上下文中
- **THEN** 输出 `min(1, 2, 3)` 而非简化为 `1`
