## ADDED Requirements

### Requirement: string.str-insert 模块限定名可调用

`string.str-insert($string, $insert, $index)` SHALL 通过模块限定名正确调用，不 SHALL 报 "Undefined function"。

#### Scenario: string.str-insert 基本调用

- **WHEN** 调用 `string.str-insert("Hello world", " Universe", 6)`
- **THEN** 返回 `"Hello Universe world"`

#### Scenario: str-insert 全局名调用

- **WHEN** 调用 `str-insert("abcd", "X", 2)`
- **THEN** 返回 `"aXbcd"`

### Requirement: str-index 参数类型强制转换

`str-index($string, $substring)` SHALL 接受可转换为字符串的参数，不 SHALL 因 number 参数报 "$string is not a string"。

#### Scenario: str-index 数字参数

- **WHEN** 调用 `str-index(1, "1")`
- **THEN** 返回 `1`（数字 1 转为字符串 "1" 后匹配）

### Requirement: utils.a mixin/function 解析

callable spec 中的 `utils.a` 模块函数和 mixin SHALL 通过 `@use` 正确解析和调用。

#### Scenario: utils.a 函数调用

- **WHEN** 模块定义了 `utils.a()` 函数并通过 `@use` 导入
- **THEN** 调用 `utils.a()` 返回正确结果，不报 "Undefined function: utils.a"

#### Scenario: utils.a mixin 调用

- **WHEN** 模块定义了 `utils.a()` mixin 并通过 `@use` 导入
- **THEN** 调用 `@include utils.a()` 正确执行，不报 "Undefined mixin: utils.a"

### Requirement: Calc 与字符串拼接运算

`+` 运算符 SHALL 支持 Calc 值与字符串/数字的拼接，不 SHALL 报 "Unsupported + operation"。

#### Scenario: Calc + Number 拼接

- **WHEN** 调用 `calc(100% - 10px) + 20px`
- **THEN** 返回 `calc(100% - 10px + 20px)`

#### Scenario: String + Calc 拼接

- **WHEN** 调用 `"prefix " + calc(100% - 10px)`
- **THEN** 返回 `"prefix calc(100% - 10px)"`
