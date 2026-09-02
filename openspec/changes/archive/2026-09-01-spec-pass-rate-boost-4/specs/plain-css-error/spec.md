## ADDED Requirements

### Requirement: plain CSS 中不允许的节点报错

plain CSS 模式下，SCSS 特有语法（@extend、变量、嵌套规则等）SHALL 报错，不 SHALL 静默通过。

#### Scenario: error/complex 期望错误

- **WHEN** 输入包含复杂 SCSS 语法到 plain CSS
- **THEN** 编译器报错，不输出内容

#### Scenario: error/compound 期望错误

- **WHEN** 输入包含复合 SCSS 语法到 plain CSS
- **THEN** 编译器报错

#### Scenario: error/no_selector 期望错误

- **WHEN** 输入包含无选择器的声明到 plain CSS
- **THEN** 编译器报错

### Requirement: @-moz-document 解析

`@-moz-document` 规则 SHALL 在 plain CSS 中正确解析，包括 `url-prefix`、`domain` 等参数。

#### Scenario: url-prefix 参数

- **WHEN** 输入 `@-moz-document url-prefix(...) { ... }`
- **THEN** 正确解析不报错

#### Scenario: unquoted url-prefix

- **WHEN** 输入 `@-moz-document unquoted url-prefix(...) { ... }`
- **THEN** 正确解析不报错

### Requirement: 错误消息对齐 sass-spec

错误消息 SHALL 使用英文，SHALL 匹配 sass-spec 期望的错误格式。

#### Scenario: map 类型错误

- **WHEN** 对数字调用 map 函数
- **THEN** 错误消息为 "X is not a map"（英文）

#### Scenario: 参数类型错误

- **WHEN** 对非字符串调用 string 函数
- **THEN** 错误消息为 "$string: X is not a string"（英文）
