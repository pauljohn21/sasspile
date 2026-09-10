## ADDED Requirements

### Requirement: :is/:where/:matches subselector 检测
当 extendee 是 `:is()`, `:where()`, `:matches()` 伪类的 subselector 时，系统 SHALL 将扩展视为 no-op。

#### Scenario: :is() subselector 检测
- **WHEN** extend(".c:is(d)", ":is(d)", "d.e")
- **THEN** 结果为 ".c:is(d)" (no-op)

#### Scenario: :where() subselector 检测
- **WHEN** extend(".c:where(d)", ":where(d)", "d.e")
- **THEN** 结果为 ".c:where(d)" (no-op)

#### Scenario: :matches() subselector 检测
- **WHEN** extend(".c:matches(d)", ":matches(d)", "d.e")
- **THEN** 结果为 ".c:matches(d)" (no-op)

### Requirement: :where() specificity_modification
当 extendee 是 `:where()` 的 subselector 且 extender 添加选择器时，系统 SHALL 将新选择器添加到 `:where()` 参数中。

#### Scenario: :where() 扩展添加选择器
- **WHEN** extend(":where(.x)", ".x", ".x .y")
- **THEN** 结果为 ":where(.x, .x .y)"
