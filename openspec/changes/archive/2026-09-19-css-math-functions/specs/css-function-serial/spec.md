# css-function-serial Delta Specification

## Purpose
修改 CSS 函数名序列化规则——vendor prefix 函数名（如 `-A-CALC`）规范化为小写。

## MODIFIED Requirements

### Requirement: vendor prefix 函数名规范化
系统 SHALL 对 vendor prefix CSS 函数名进行大小写规范化：保留 prefix 连字符，函数名部分转为小写。

#### Scenario: uppercase calc prefix
- **WHEN** 表达式 `-A-CALC(...)` 序列化
- **THEN** 输出为 `-a-calc(...)`（函数名部分小写）

#### Scenario: uppercase element prefix
- **WHEN** 表达式 `-C-ELEMENT(...)` 序列化
- **THEN** 输出为 `-c-element(...)`

#### Scenario: uppercase expression prefix
- **WHEN** 表达式 `-C-EXPRESSION(...)` 序列化
- **THEN** 输出为 `-c-expression(...)`

#### Scenario: mixed case unprefixed calc
- **WHEN** 表达式 `CaLc(1px)` 序列化
- **THEN** 输出为 `calc(1px)`（全小写）

#### Scenario: mixed case unprefixed url
- **WHEN** 表达式 `URL(http://...)` 序列化
- **THEN** 输出为 `url(http://...)`

### Requirement: CSS function 内注释处理
CSS 函数参数中的注释 SHALL 按 CSS 规范处理：行注释截断、块注释移除或保留单个空格。

#### Scenario: 行注释参数内
- **WHEN** 表达式 `calc(// comment\n c)` 序列化
- **THEN** 输出为 `calc( c)`（行注释 + 换行 → 单空格）

#### Scenario: 块注释参数内
- **WHEN** 表达式 `calc(c /**/)` 序列化
- **THEN** 输出为 `calc(c /**/)` 或 `calc(c )`（块注释保留或单空格）
