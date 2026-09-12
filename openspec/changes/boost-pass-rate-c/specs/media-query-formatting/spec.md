## ADDED Requirements

### Requirement: 媒体查询逻辑操作符须包含空格分隔
Sass 编译器 SHALL 在序列化媒体查询时，`not`、`and`、`or` 操作符与括号表达式之间输出空格。

#### Scenario: not 操作符
- **WHEN** 序列化 `@media not (condition)`
- **THEN** 输出 SHALL 包含 `not (condition)`（not 后有空格）

#### Scenario: or 操作符
- **WHEN** 序列化 `@media (a) or (b)`
- **THEN** 输出 SHALL 包含 `or` 两侧的空格

#### Scenario: and 操作符
- **WHEN** 序列化 `@media a and (b)`
- **THEN** 输出 SHALL 包含 `and` 两侧的空格

#### Scenario: 仅类型无操作符
- **WHEN** 序列化 `@media (min-width: 768px)`
- **THEN** 输出 SHALL 不包含多余空格
