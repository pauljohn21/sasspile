## ADDED Requirements

### Requirement: @use with 配置错误检测

系统 SHALL 在 @use with 配置中检测未定义变量、非默认值引用、namespace 错误、嵌套配置、重复变量、多配置冲突并报错。

#### Scenario: with undefined 变量
- **WHEN** `@use "module" with ($undefined: value)` 被执行且模块不定义 `$undefined`
- **THEN** 系统 SHALL 报 "Undefined variable" 错误

#### Scenario: with not_default 变量
- **WHEN** `@use "module" with ($var: value)` 被执行且模块定义 `$var` 不带 `!default`
- **THEN** 系统 SHALL 报 "This variable was not declared with !default" 错误

#### Scenario: with namespace 错误
- **WHEN** `@use "sass:math" with ($pi: 3)` 被执行（内建模块不接受 with 配置）
- **THEN** 系统 SHALL 报错

#### Scenario: with 嵌套配置错误
- **WHEN** 模块 A 通过 @use 引入模块 B（带 with 配置），模块 A 自身被 @use with 覆盖
- **THEN** 系统 SHALL 根据 sass-spec 规则验证嵌套配置合法性

#### Scenario: with conflict
- **WHEN** `@use "module" with ($var: a, $var: b)` 被执行（重复变量配置）
- **THEN** 系统 SHALL 报 "The same variable may only be configured once." 错误

#### Scenario: with invalid_expression error
- **WHEN** `@use "module" with ($var: 1 + )` 被执行（配置值表达式无效）
- **THEN** 系统 SHALL 报表达式语法错误

#### Scenario: with core_module
- **WHEN** `@use "sass:math" with ($var: value)` 被执行
- **THEN** 系统 SHALL 报 "Built-in modules can't be configured." 错误

#### Scenario: with multi_configuration one_file
- **WHEN** 同一文件中多次 `@use "module" with (...)` 且配置不同
- **THEN** 系统 SHALL 报 "This module was already loaded with a different configuration." 错误

#### Scenario: with multi_configuration multi_file
- **WHEN** 不同文件分别 `@use "module" with (...)` 且配置不同
- **THEN** 系统 SHALL 报多配置冲突错误

#### Scenario: with multi_configuration unconfigured_first
- **WHEN** 先 `@use "module"` 不带 with，后 `@use "module" with (...)`
- **THEN** 系统 SHALL 报多配置冲突错误

#### Scenario: with multi_configuration through_forward
- **WHEN** 通过 @forward 传递 with 配置后再被 @use with 覆盖
- **THEN** 系统 SHALL 报多配置冲突错误

#### Scenario: with repeated_variable
- **WHEN** `@use "module" with ($var: a, $var: b)` 被执行
- **THEN** 系统 SHALL 报重复变量配置错误

#### Scenario: with through_forward show
- **WHEN** `@forward "..." with ($var: value) show $var` 被执行
- **THEN** 系统 SHALL 根据 sass-spec 规则验证

#### Scenario: with through_forward hide
- **WHEN** `@forward "..." with ($var: value) hide $var` 被执行
- **THEN** 系统 SHALL 根据 sass-spec 规则验证

#### Scenario: with through_forward as
- **WHEN** `@forward "..." with ($var: value) as prefix-*` 被执行
- **THEN** 系统 SHALL 根据 sass-spec 规则验证

#### Scenario: with through_forward with
- **WHEN** `@forward "..." with ($var: value)` 后接 `@forward "..." with ($var: other)`
- **THEN** 系统 SHALL 根据 sass-spec 规则验证

#### Scenario: with private different
- **WHEN** `@use "module" with ($-private: value)` 被执行且模块定义了 `$-private` 带 `!default`
- **THEN** 系统 SHALL 根据 sass-spec 规则验证私有变量配置行为
