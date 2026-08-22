## ADDED Requirements

### Requirement: @use with 未定义变量检测

系统 SHALL 在 `@use ... with ($var: value)` 中检测目标模块未定义 `$var` 的情况，并报错。

#### Scenario: 未定义变量
- **WHEN** 模块 `_mixin.scss` 不定义 `$color` 变量，且 `@use "_mixin" with ($color: red)` 被执行
- **THEN** 系统 SHALL 报错，错误消息包含 `$color` 变量名和模块路径

#### Scenario: 非默认值引用
- **WHEN** 模块定义 `$color: blue`（非 `!default`），且 `@use "_mixin" with ($color: red)` 被执行
- **THEN** 系统 SHALL 报错，错误消息说明该变量不是 `!default` 变量

#### Scenario: namespace 错误
- **WHEN** `@use "sass:math" with ($pi: 3)` 被执行（内建模块不接受 with 配置）
- **THEN** 系统 SHALL 报错，错误消息说明内建模块不接受配置

#### Scenario: 嵌套配置验证
- **WHEN** 模块 A 通过 `@use` 引入模块 B，模块 B 的 `@use` 带 with 配置，但模块 A 也尝试通过 `@use B with (...)` 覆盖
- **THEN** 系统 SHALL 根据 sass-spec 规则验证嵌套配置的合法性

#### Scenario: 重复变量配置
- **WHEN** `@use "_mixin" with ($color: red, $color: blue)` 被执行（同一变量配置两次）
- **THEN** 系统 SHALL 报错，错误消息说明重复配置

#### Scenario: 多配置冲突
- **WHEN** 同一模块被多次 `@use` 且 with 配置不同
- **THEN** 系统 SHALL 报错，错误消息说明多配置冲突

#### Scenario: with 配置通过 @forward 传递
- **WHEN** `@forward "_mixin" with ($color: red)` 和 `@use "_other" with ($color: blue)` 组合使用
- **THEN** 系统 SHALL 根据 sass-spec 规则验证 through_forward 配置合法性
