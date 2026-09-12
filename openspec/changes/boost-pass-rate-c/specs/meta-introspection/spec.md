## ADDED Requirements

### Requirement: function_exists 正确检测
`meta.function-exists()` 函数 SHALL 在当前作用域及全局作用域检测函数是否存在。

#### Scenario: 全局函数存在
- **WHEN** 调用 `function-exists('rgb')`
- **THEN** 返回 SHALL 为 `true`

#### Scenario: 用户定义函数
- **WHEN** 在当前模块定义 `@function foo() {}` 后调用 `function-exists('foo')`
- **THEN** 返回 SHALL 为 `true`

### Requirement: variable_exists 正确检测
`meta.variable-exists()` 函数 SHALL 在正确的作用域链中检测变量。

#### Scenario: 局部变量
- **WHEN** 在函数内部定义 `$x` 后调用 `variable-exists('x')`
- **THEN** 返回 SHALL 为 `true`

### Requirement: module_functions 内省
`meta.module-functions()` 函数 SHALL 列出指定模块的所有公开函数。

#### Scenario: sass:color 模块
- **WHEN** 调用 `module-functions('color')`
- **THEN** 返回 SHALL 包含 color 模块的所有公开函数名
