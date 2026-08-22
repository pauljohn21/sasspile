## ADDED Requirements

### Requirement: 特殊函数名序列化

系统 SHALL 正确序列化特殊函数名（calc/clamp/expression/url/element/type），包括大小写和前缀处理。

#### Scenario: calc 函数名
- **WHEN** 表达式包含 `calc(...)` 或 `CALC(...)` 或 `-calc(...)`
- **THEN** 系统 SHALL 根据 sass-spec 规则正确序列化函数名（保留原始大小写或标准化）

#### Scenario: clamp 函数名
- **WHEN** 表达式包含 `clamp(...)` 或 `CLAMP(...)` 或 `-clamp(...)`
- **THEN** 系统 SHALL 根据 sass-spec 规则正确序列化

#### Scenario: expression 函数名
- **WHEN** 表达式包含 `expression(...)` 或 `EXPRESSION(...)` 或 `-moz-binding-expression(...)`
- **THEN** 系统 SHALL 根据 sass-spec 规则正确序列化或报错

#### Scenario: url 函数名
- **WHEN** 表达式包含 `url(...)` 或 `URL(...)` 或 `-url(...)`
- **THEN** 系统 SHALL 根据 sass-spec 规则正确序列化

#### Scenario: element 函数名
- **WHEN** 表达式包含 `element(...)` 或 `ELEMENT(...)` 或 `-moz-element(...)`
- **THEN** 系统 SHALL 根据 sass-spec 规则正确序列化或报错

#### Scenario: type 函数名
- **WHEN** 表达式包含 `type(...)` 或 `TYPE(...)` 或 `-type(...)`
- **THEN** 系统 SHALL 根据 sass-spec 规则正确序列化或报错

### Requirement: 特殊函数名错误检测

系统 SHALL 检测特殊函数名在不允许的上下文中的使用。

#### Scenario: 小写无前缀 element 错误
- **WHEN** 表达式包含 `element(...)` （无供应商前缀的小写形式）
- **THEN** 系统 SHALL 报错

#### Scenario: 小写无前缀 expression 错误
- **WHEN** 表达式包含 `expression(...)` （无供应商前缀的小写形式）
- **THEN** 系统 SHALL 报错

#### Scenario: 小写无前缀 url 错误
- **WHEN** 表达式包含 `url(...)` （某些不允许的上下文中）
- **THEN** 系统 SHALL 报错

#### Scenario: 小写无前缀 type 错误
- **WHEN** 表达式包含 `type(...)` （无供应商前缀的小写形式）
- **THEN** 系统 SHALL 报错
