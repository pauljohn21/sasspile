## ADDED Requirements

### Requirement: 错误消息统一使用英文

系统 SHALL 在所有错误消息中使用英文，禁止中文错误消息。sass-spec 的期望输出和错误消息均为英文，中文消息必定不匹配。

#### Scenario: 不是 map 错误消息
- **WHEN** 系统检测到非 map 类型被当作 map 使用
- **THEN** 错误消息 SHALL 为 "X is not a map" 而非 "X 不是 map"

#### Scenario: 不是 string 错误消息
- **WHEN** 系统检测到非 string 类型被当作 string 使用
- **THEN** 错误消息 SHALL 为 "X is not a string" 而非 "X 不是 string"

#### Scenario: 不是 number 错误消息
- **WHEN** 系统检测到非 number 类型被当作 number 使用
- **THEN** 错误消息 SHALL 为 "X is not a number" 而非 "X 不是 number"

#### Scenario: 不是 list 错误消息
- **WHEN** 系统检测到非 list 类型被当作 list 使用
- **THEN** 错误消息 SHALL 为 "X is not a list" 而非 "X 不是 list"

#### Scenario: 参数不足错误消息
- **WHEN** 系统检测到函数参数不足
- **THEN** 错误消息 SHALL 为 "Missing argument $X." 而非中文

#### Scenario: 参数过多错误消息
- **WHEN** 系统检测到函数参数过多
- **THEN** 错误消息 SHALL 为 "Only N argument(s) allowed, but M were/was passed." 而非中文
