## ADDED Requirements

### Requirement: 带参数伪类匹配
系统 SHALL 支持带参数伪类（如 `:nth-child()`, `:nth-last-child()`）的 extend 匹配。

#### Scenario: :nth-child() 匹配
- **WHEN** 选择器包含 `:nth-child(2n+1)` 且 extendee 匹配该伪类
- **THEN** 系统 SHALL 识别为有效匹配

#### Scenario: :nth-last-child() 匹配
- **WHEN** 选择器包含 `:nth-last-child(odd)` 且 extendee 匹配该伪类
- **THEN** 系统 SHALL 识别为有效匹配

#### Scenario: :nth-child() 参数化扩展
- **WHEN** extend(":nth-child(2n+1)", ":nth-child(2n+1)", ".foo")
- **THEN** 结果追加 `.foo` 到 compound
