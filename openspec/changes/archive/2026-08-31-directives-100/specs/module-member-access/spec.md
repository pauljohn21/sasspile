## ADDED Requirements

### Requirement: @use/@forward 变量遮蔽

系统 SHALL 正确处理 @forward 链中的变量遮蔽（through_forward）行为。

#### Scenario: shadowed variable through_forward
- **WHEN** 模块 A 定义 `$var: 1`，模块 B 执行 `@forward "A"` 后定义 `$var: 2`，用户 `@use "B"` 后访问 `$var`
- **THEN** 系统 SHALL 返回模块 B 的 `$var` 值（遮蔽上游）

#### Scenario: shadowed nested global through_forward
- **WHEN** 模块 A 定义 `$var: 1`（global），模块 B 执行 `@forward "A"` 后在嵌套作用域定义 `$var: 2`
- **THEN** 系统 SHALL 根据 sass-spec 规则处理嵌套全局变量遮蔽

#### Scenario: shadowed nested local through_forward
- **WHEN** 模块 A 定义 `$var: 1`，模块 B 执行 `@forward "A"` 后在嵌套作用域定义 `$var: 2`（local）
- **THEN** 系统 SHALL 根据 sass-spec 规则处理嵌套局部变量遮蔽

### Requirement: @use/@forward 变量覆盖

系统 SHALL 正确处理 @use/@forward 中的变量覆盖（override）行为。

#### Scenario: override variable
- **WHEN** 模块 A 定义 `$var: 1`，用户 `@use "A" with ($var: 2)` 后 `@forward "A"` 中 `$var` 被覆盖
- **THEN** 系统 SHALL 根据 sass-spec 规则处理变量覆盖

#### Scenario: override mixin
- **WHEN** 模块 A 定义 mixin `m`，通过 @forward 覆盖
- **THEN** 系统 SHALL 根据 sass-spec 规则处理 mixin 覆盖

#### Scenario: override function
- **WHEN** 模块 A 定义 function `f`，通过 @forward 覆盖
- **THEN** 系统 SHALL 根据 sass-spec 规则处理 function 覆盖

### Requirement: @use/@forward 优先级

系统 SHALL 正确处理 @use/@forward 中的优先级（precedence）差异。

#### Scenario: precedence top_level
- **WHEN** 多个 @use/@forward 在顶层定义同名变量
- **THEN** 系统 SHALL 根据 sass-spec 规则处理顶层优先级

#### Scenario: precedence nested
- **WHEN** 多个 @use/@forward 在嵌套作用域定义同名变量
- **THEN** 系统 SHALL 根据 sass-spec 规则处理嵌套优先级

### Requirement: @use with non_overridable

系统 SHALL 检测 @use with 中不可覆盖的变量。

#### Scenario: non_overridable 变量
- **WHEN** 模块定义 `$var: value`（非 `!default`），且 `@use "module" with ($var: other)` 被执行
- **THEN** 系统 SHALL 报 "This variable was not declared with !default" 错误

### Requirement: dash_insensitive 变量访问

系统 SHALL 支持变量名中 `-` 和 `_` 不敏感匹配。

#### Scenario: dash_insensitive 访问
- **WHEN** 模块定义 `$my-var`，用户访问 `$my_var`
- **THEN** 系统 SHALL 正确返回变量值

### Requirement: variable_exists 检查

系统 SHALL 正确处理 variable_exists 检查。

#### Scenario: variable_exists
- **WHEN** `variable-exists(€var)` 被调用
- **THEN** 系统 SHALL 根据 sass-spec 规则返回变量是否存在
