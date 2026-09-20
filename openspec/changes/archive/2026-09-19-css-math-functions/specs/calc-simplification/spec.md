# calc-simplification Delta Specification

## Purpose
修改 calc 简化管线的触发条件——识别 CSS 原生 math 函数并跳过简化，保持 CSS 语义。

## MODIFIED Requirements

### Requirement: calc simplification scope
calc simplification SHALL 仅对 Sass 表达式数学运算（由 Sass 操作符 `+`、`-`、`*`、`/` 触发）生效；对 CSS 原生函数 calc()/round()/clamp()/rem()/mod()/min()/max()/var() 内部的子表达式 SHALL 跳过简化。

#### Scenario: CSS round 内部不简化
- **WHEN** 表达式 `round(117, 25)` 进入 eval
- **THEN** 不触发 calc simplification，保持原样

#### Scenario: var fallback 不简化
- **WHEN** 表达式 `var(--c, 1 + 2)` 进入 AST
- **THEN** fallback `1 + 2` 不折叠为 `3`

#### Scenario: CSS calc 内部不简化
- **WHEN** 表达式 `calc(calc(1px))` 进入 eval
- **THEN** 内层 calc(1px) 保持，外层可能根据上下文简化

#### Scenario: Sass math.round 仍简化
- **WHEN** 表达式 `math.round(2.3)` 进入 eval
- **THEN** 触发 Sass 简化，结果为 `2`
