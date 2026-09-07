## ADDED Requirements

### Requirement: 数字解析边界
系统 SHALL 解析 SCSS 中各种数字格式——整数、浮点、科学计数法、负数、带单位。

#### Scenario: 科学计数法
- **WHEN** 解析 `1e3` 或 `2.5e-2`
- **THEN** 作为数值正确求值

#### Scenario: 负零
- **WHEN** 解析 `-0` 或 `-0.0`
- **THEN** 正确处理为 0（不保留负号）

#### Scenario: 前导小数点
- **WHEN** 解析 `.5`（无前导 0）
- **THEN** 正确解析为 `0.5`

### Requirement: 字符串解析和序列化
系统 SHALL 处理带引号/不带引号字符串的解析和序列化边界。

#### Scenario: 空字符串序列化
- **WHEN** 值 `""` 被序列化
- **THEN** 输出 `""`（带双引号）

#### Scenario: 含特殊字符
- **WHEN** 字符串含 `#{}` 插值残留、引号、反斜杠
- **THEN** 正确转义输出

### Requirement: Null 值处理
系统 SHALL 正确处理 SCSS `null` 值——作为 falsy，序列化为 `null`，在列表中过滤。

#### Scenario: Null 布尔判断
- **WHEN** `@if null { ... }`
- **THEN** 条件为 false，走 else 分支

#### Scenario: Null 在列表中
- **WHEN** 构建列表 `1, null, 2`
- **THEN** null 保留在列表中（不自动过滤）

#### Scenario: Null 序列化
- **WHEN** 序列化变量 `$x: null`（非 plain CSS）
- **THEN** 该声明不输出（null 抑制）

### Requirement: Boolean 字面量
系统 SHALL 正确解析 `true`、`false`，区分于字符串。

#### Scenario: true 判断
- **WHEN** `@if true { ... }`
- **THEN** 条件为 true

#### Scenario: false 判断
- **WHEN** `@if false { ... }`
- **THEN** 条件为 false

### Requirement: 列表/Map 边界
系统 SHALL 处理空列表、单元素列表、嵌套列表、空 Map 边界。

#### Scenario: 空括号列表
- **WHEN** 解析 `()`
- **THEN** 作为空列表（或单元素空组，取决于上下文）

#### Scenario: 单元素列表
- **WHEN** 解析 `(1)` 或 `(1,)`
- **THEN** 作为单元素列表

#### Scenario: 空 Map
- **WHEN** 解析 `()` 作为 map 上下文
- **THEN** 作为空 Map

### Requirement: 插值边界
系统 SHALL 处理各种插值场景——属性值、选择器、字符串、url() 内。

#### Scenario: url() 内插值
- **WHEN** `url(#{ $var }.png)`
- **THEN** 正确拼接路径

#### Scenario: 空插值
- **WHEN** `"#{}"`（空插值表达式）
- **THEN** 输出空字符串
