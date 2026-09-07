## ADDED Requirements

### Requirement: 错误消息基本格式
系统 SHALL 在错误消息中包含文件位置、错误类型、上下文信息。

#### Scenario: 未定义变量
- **WHEN** 使用未定义变量 `$undefined`
- **THEN** 报错 `Undefined variable: $undefined` 并附行号

#### Scenario: 未定义函数
- **WHEN** 调用未定义函数 `unknown()`
- **THEN** 报错 "Undefined function" 或 CSS 透传

#### Scenario: 类型错误
- **WHEN** 对非数值执行数值运算
- **THEN** 报类型不匹配错误，包含操作数信息

### Requirement: 模块系统错误
系统 SHALL 对模块系统错误提供明确的错误消息。

#### Scenario: 模块未找到
- **WHEN** @use 不存在的模块
- **THEN** 报 "Can't find stylesheet to import: <path>" 并提供搜索路径

#### Scenario: 命名空间冲突
- **WHEN** 重复 @use 同一模块无 as
- **THEN** 报 "There's already a module with namespace"

#### Scenario: 成员未找到
- **WHEN** 通过命名空间访问不存在的成员
- **THEN** 报未找到错误，列出可用成员

### Requirement: 语法错误消息
系统 SHALL 对解析错误提供精确位置指示。

#### Scenario: 期望 token 缺失
- **WHEN** 缺少 `:`、`;`、`}` 等 token
- **THEN** 报 "expected X, found Y" 并附上下文

#### Scenario: 括号不匹配
- **WHEN** 缺少 `)`、`]`
- **THEN** 报 "expected )" 指示位置

### Requirement: 运行时错误位置
系统 SHALL 在错误消息中指向源码位置（文件名:行号）。

#### Scenario: 顶层错误
- **WHEN** 评估时出错
- **THEN** 错误包含 `filename.scss:line`

#### Scenario: 嵌套错误
- **WHEN** @include mixin 内部出错
- **THEN** 包含 mixin 定义位置和调用位置的链式指示

### Requirement: @extend 错误消息
系统 SHALL 对 @extend 各种违规情况提供对应的错误消息。

#### Scenario: 复杂选择器 extend
- **WHEN** `@extend a b`
- **THEN** 报 "complex selectors may not be extended"

#### Scenario: 复合选择器 extend
- **WHEN** `@extend a:hover`（非纯伪类）
- **THEN** 报 "compound selectors may no longer be extended" 并附建议

#### Scenario: 不存在的目标 extend
- **WHEN** `@extend .nonexistent`
- **THEN** 报 "The target selector was not found" 或静默（取决于 optional 标记）
