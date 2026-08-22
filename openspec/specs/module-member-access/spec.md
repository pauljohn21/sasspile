## Purpose

defines sasspile compiler specs.

## Requirements



### Requirement: 模块成员变量访问
系统 SHALL 支持通过 `$namespace.variablename` 语法访问 `@use` 模块导出的成员变量。

#### Scenario: 访问模块变量
- **WHEN** 执行 `@use "theme" as t;` 后访问 `$t.color`
- **THEN** 系统 返回 `theme` 模块中导出的 `color` 变量值

#### Scenario: 访问链式模块变量
- **WHEN** 模块 A `@forward` 模块 B 的变量，用户 `@use A as a` 后访问 `$a.var`
- **THEN** 系统 正确返回通过 forward 链传递的变量值

### Requirement: 模块变量在 Env 中的查找
系统 SHALL 在变量查找时检测变量名中的 `.` 分隔符，并从对应命名空间的 `vars` 中获取值。

#### Scenario: 命名空间变量查找
- **WHEN** 变量名为 `ns.name` 且 `env.namespaces` 中存在 `ns` 模块
- **THEN** 系统 从 `ns` 模块的 `vars` 中查找 `name` 并返回值

#### Scenario: 命名空间不存在
- **WHEN** 变量名为 `ns.name` 但 `env.namespaces` 中不存在 `ns`
- **THEN** 系统 报 "Undefined variable: $ns.name" 错误

### Requirement: 模块变量在 @forward 中的传递
系统 SHALL 在 `@forward` 指令中将上游模块的变量传递到下游模块的导出中。

#### Scenario: forward 变量
- **WHEN** 模块 B 执行 `@forward "A"`
- **THEN** 模块 A 的所有公开变量通过 B 的命名空间可访问
