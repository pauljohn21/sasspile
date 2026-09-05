## MODIFIED Requirements

### Requirement: @forward show/hide 边界
MUST 正确处理 `@forward "url" show $name` 和 `@forward "url" hide $name` 的成员可见性控制。

#### Scenario: show 仅导出指定成员
- **WHEN** 使用 `show` 限定仅导出部分成员
- **THEN** 未列出的成员不可通过该模块访问

#### Scenario: hide 排除指定成员
- **WHEN** 使用 `hide` 排除部分成员
- **THEN** 被 hide 的成员不可通过该模块访问

### Requirement: @use with() 配置边界
MUST 正确处理 `@use "url" with ($var: value)` 覆盖模块默认变量。

#### Scenario: 多变量覆盖
- **WHEN** with() 中覆盖多个 !default 变量
- **THEN** 所有覆盖值生效
