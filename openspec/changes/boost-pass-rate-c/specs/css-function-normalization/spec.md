## ADDED Requirements

### Requirement: 内置 CSS 函数名须输出 lower-case
Sass 编译器 SHALL 将内置 CSS 函数序列化为 lowercase 名称。

#### Scenario: uppercase 输入
- **WHEN** CSS 输入包含 `TYPE(0)` 函数调用
- **THEN** 输出 SHALL 为 `type(0)`（lowercase）

#### Scenario: vendor 前缀函数保留
- **WHEN** CSS 输入包含 `-webkit-calc()`、`-moz-element()` 等 vendor 前缀函数
- **THEN** 输出 SHALL 保留 vendor 前缀和正确的括号内间距

### Requirement: vendor 前缀函数注释间距规范化
Sass 编译器 SHALL 正确处理 vendor 前缀函数调用中的注释空格。

#### Scenario: calc 注释在开括号后
- **WHEN** 序列化 `/*comment*/calc(expr)` 形式
- **THEN** 输出 SHALL 正确处理注释空格，匹配 Sass 规范期望
