## ADDED Requirements

### Requirement: calc-size() 参数保留
`calc-size()` 函数 SHALL 在 sasspile 中作为保留函数处理，不执行编译时求值。

#### Scenario: calc-size 基础保留
- **WHEN** 输入为 `calc-size(auto, 20px)`
- **THEN** 输出保持 `calc-size(auto, 20px)` 原样

#### Scenario: calc-size 含 calc 表达式
- **WHEN** 输入为 `calc-size(auto, calc(10px * 2))`
- **THEN** 输出保留 calc-size 结构

### Requirement: calc-size 参数校验错误传递
当 `calc-size()` 参数明显不合法时，SHALL 透传保留而不是产生内部错误。

#### Scenario: calc-size 零参数（保留错误）
- **WHEN** 输入为 `calc-size()`
- **THEN** 系统原样保留或传递编译错误，不产生 panic

#### Scenario: calc-size 过多参数（保留错误）
- **WHEN** 输入为 `calc-size(auto, 1px, 2px)`
- **THEN** 系统原样保留或传递编译错误
