## ADDED Requirements

### Requirement: @namespace 声明
MUST 解析 `@namespace` 声明，支持有前缀和无前缀形式。

#### Scenario: 带前缀 namespace
- **WHEN** 输入包含 `@namespace svg "http://www.w3.org/2000/svg";`
- **THEN** 输出保留 namespace 声明

#### Scenario: 默认 namespace
- **WHEN** 输入包含 `@namespace "http://www.w3.org/1999/xhtml";`
- **THEN** 输出保留默认 namespace 声明
