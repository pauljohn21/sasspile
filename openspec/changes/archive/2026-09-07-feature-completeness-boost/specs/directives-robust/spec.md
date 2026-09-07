## ADDED Requirements

### Requirement: @for 边界行为
系统 SHALL 处理 @from/@to 的各种数值类型边界。

#### Scenario: 浮点步长
- **WHEN** `@for $i from 0.5 through 2.5`
- **THEN** 循环 $i = 0.5, 1.5, 2.5

#### Scenario: 反向循环
- **WHEN** `@for $i from 5 through 1`
- **THEN** 循环 $i = 5, 4, 3, 2, 1

#### Scenario: 单位一致性
- **WHEN** `@for $i from 1px through 5px`
- **THEN** 每次递增 1px

#### Scenario: 不兼容单位（错误）
- **WHEN** `@for $i from 1px through 5%`
- **THEN** 报 "incompatible units" 错误

### Requirement: @each 边界行为
系统 SHALL 处理 @each 的各种列表边界。

#### Scenario: 空列表
- **WHEN** `@each $x in ()`
- **THEN** 循环体不执行

#### Scenario: 多变量解构
- **WHEN** `@each $key, $value in $map`
- **THEN** 解构为键值对

#### Scenario: 嵌套列表
- **WHEN** `@each $item in (1 2, 3 4)`
- **THEN** 每个元素本身是列表

### Requirement: @while 边界行为
系统 SHALL 处理 @while 的边界条件。

#### Scenario: 初始 false
- **WHEN** `@while false`
- **THEN** 循环体不执行

#### Scenario: 复杂条件
- **WHEN** `@while $i > 0 and $j < 10`
- **THEN** 复合条件求值

### Requirement: @if 边界行为
系统 SHALL 处理 @if 的各种 falsy/truthy 边界。

#### Scenario: 空字符串为 truthy
- **WHEN** `@if ""`
- **THEN** 条件为 true（仅 null/false 是 falsy）

#### Scenario: 0 为 truthy
- **WHEN** `@if 0`
- **THEN** 条件为 true

#### Scenario: 空列表为 truthy
- **WHEN** `@if ()`
- **THEN** 条件为 true

### Requirement: @function 边界行为
系统 SHALL 处理 @function 的各种边界。

#### Scenario: 可选参数（带默认值）
- **WHEN** `@function foo($a, $b: 10)`
- **THEN** `$b` 为可选参数

#### Scenario: 可变参数
- **WHEN** `@function foo($args...)`
- **THEN** 收集剩余参数为列表

#### Scenario: @return 终止
- **WHEN** `@return $value` 在条件分支中
- **THEN** 立即返回，不执行后续代码

### Requirement: @mixin 边界行为
系统 SHALL 处理 @mixin 的边界情况。

#### Scenario: 空 mixin
- **WHEN** `@mixin empty { }`
- **THEN** include 时不输出任何内容

#### Scenario: 参数默认复杂表达式
- **WHEN** `@mixin foo($a: 1 + 2)`
- **THEN** 默认值在调用时求值
