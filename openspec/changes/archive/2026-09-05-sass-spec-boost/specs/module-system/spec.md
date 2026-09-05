## MODIFIED Requirements

### Requirement: @import 嵌套导入解析
系统 SHALL 在嵌套上下文中正确解析 `@import` 语句，支持从被导入文件导入其他文件的链式导入。

#### Scenario: 嵌套导入链
- **WHEN** `_a.scss` 导入 `_b.scss`，`_b.scss` 导入 `_c.scss`
- **THEN** `_a.scss` 中可以使用 `_c.scss` 定义的变量和函数

#### Scenario: 导入路径回退
- **WHEN** 直接使用部分文件名导入时
- **THEN** 系统遵循 Sass 文件解析优先级（partial > 同名无 partial > 同目录 index）

### Requirement: @function 返回值类型
系统 SHALL `@function` 返回值正确处理列表、map、null 等类型，确保类型转换符合规范。

#### Scenario: 函数返回 null
- **WHEN** `@function` 返回 `null`
- **THEN** 调用处正确接收到空值

#### Scenario: 函数返回空列表
- **WHEN** `@function` 返回 `()`
- **THEN** 调用处正确接收到空列表
