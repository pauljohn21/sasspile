## ADDED Requirements

### Requirement: 字符串拼接
系统 SHALL 处理 `+` 运算符对字符串的拼接行为——结果引号由左操作数决定。

#### Scenario: 未引号 + 未引号
- **WHEN** `"foo" + "bar"`（均为字符串值）或 `foo + bar`（标识符拼接）
- **THEN** 返回拼接结果

#### Scenario: 引号 + 未引号
- **WHEN** 左操作数为引号字符串，右操作数为未引号
- **THEN** 结果保留引号

### Requirement: 数值运算类型转换
系统 SHALL 处理数值的类型转换——to number、to string。

#### Scenario: 字符串转数值
- **WHEN** `"123"` 参与数值运算
- **THEN** 根据上下文进行转换

#### Scenario: 数值转字符串
- **WHEN** `123` 参与字符串拼接
- **THEN** 转为字符串形式

### Requirement: null 传播
系统 SHALL 处理 null 在运算符中的传播行为。

#### Scenario: null 相等比较
- **WHEN** `null == null`
- **THEN** 返回 `true`

#### Scenario: null 参与算术
- **WHEN** `null + 1`
- **THEN** 报错或返回 null（取决于规范）

### Requirement: 布尔逻辑
系统 SHALL 正确处理 `and`、`or`、`not` 运算符。

#### Scenario: 短路求值
- **WHEN** `false and $undefined-var`
- **THEN** 返回 false，不触发未定义变量错误

#### Scenario: not 运算
- **WHEN** `not true`
- **THEN** 返回 false

#### Scenario: not 对非布尔值
- **WHEN** `not 0`
- **THEN** 返回 false（0 在 SCSS 中 truthy... 实际需确认规范）

### Requirement: 比较运算符
系统 SHALL 处理 `>`、`<`、`>=`、`<=`、`==`、`!=` 对所有可比较类型。

#### Scenario: 数值比较
- **WHEN** `5 > 3`
- **THEN** 返回 true

#### Scenario: 字符串比较
- **WHEN** `"a" < "b"`
- **THEN** 返回 true（按字典序）

#### Scenario: 带单位比较
- **WHEN** `5px > 3px`
- **THEN** 返回 true（同单位可比较）

#### Scenario: 不兼容单位比较
- **WHEN** `5px > 3%`
- **THEN** 报 incompatible units 错误

### Requirement: 除法运算符优先级
系统 SHALL 区分 `/` 作为 CSS 分隔符和 SCSS 除法运算符。

#### Scenario: 括号除法
- **WHEN** `(10 / 2)`
- **THEN** 作为除法运算得 5

#### Scenario: CSS 简写中的 /
- **WHEN** `font: 12px/1.5`
- **THEN** 保留为 CSS 简写，不做除法

### Requirement: 模运算
系统 SHALL 处理 `%` 模运算符。

#### Scenario: 整数模
- **WHEN** `10 % 3`
- **THEN** 返回 1

#### Scenario: 浮点模
- **WHEN** `10.5 % 3`
- **THEN** 返回 1.5
