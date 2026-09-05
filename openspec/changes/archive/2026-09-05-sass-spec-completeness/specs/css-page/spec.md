## ADDED Requirements

### Requirement: @page 规则解析
MUST 解析 `@page { ... }` 规则及其伪类选择器（`:left`、`:right`、`:first`）。

#### Scenario: 基本 page
- **WHEN** 输入包含 `@page { margin: 1cm; }`
- **THEN** 输出保留 @page 规则

#### Scenario: page 伪类
- **WHEN** 输入包含 `@page :first { margin: 2cm; }`
- **THEN** 正确解析 `:left` / `:right` / `:first` 伪类选择器
