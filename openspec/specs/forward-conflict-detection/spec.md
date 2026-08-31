## MODIFIED Requirements

### Requirement: @forward 同名成员冲突检测
系统 SHALL 在多个 @forward 指令合并 exports 时检测同名成员冲突并报错。扩展覆盖 same_value、because_of_as、syntax/as 验证、私有成员访问场景。

#### Scenario: 变量冲突
- **WHEN** 两个 `@forward` 指令分别转发了同名变量 `$a`
- **THEN** 系统 报 "Two forwarded modules both define a variable named $a." 错误

#### Scenario: 函数冲突
- **WHEN** 两个 `@forward` 指令分别转发了同名函数 `c`
- **THEN** 系统 报 "Two forwarded modules both define a function named c." 错误

#### Scenario: mixin 冲突
- **WHEN** 两个 `@forward` 指令分别转发了同名 mixin `b`
- **THEN** 系统 报 "Two forwarded modules both define a mixin named b." 错误

#### Scenario: 同值变量不冲突
- **WHEN** 两个 `@forward` 指令转发了同名且同值的变量
- **THEN** 系统 不报错（允许同值冲突）

#### Scenario: 同值函数不冲突
- **WHEN** 两个 `@forward` 指令转发了同名且同实现的函数
- **THEN** 系统 不报错（允许同值冲突）

#### Scenario: 同值 mixin 不冲突
- **WHEN** 两个 `@forward` 指令转发了同名且同内容的 mixin
- **THEN** 系统 不报错（允许同值冲突）

#### Scenario: because_of_as 冲突
- **WHEN** 两个 `@forward "... as a-*"` 和 `@forward "... as b-*"` 指令转发后产生同名成员（因为 as 前缀导致冲突）
- **THEN** 系统 SHALL 报冲突错误

#### Scenario: 带前缀转发冲突
- **WHEN** 两个 `@forward "... as prefix-*"` 指令转发了同名成员（加前缀后名称相同）
- **THEN** 系统 报冲突错误

#### Scenario: 混合成员类型不冲突
- **WHEN** 两个 `@forward` 分别转发了变量 `$a` 和函数 `a`（不同类型同名）
- **THEN** 系统 不报错（不同类型不冲突）

#### Scenario: syntax as nothing 错误
- **WHEN** `@forward "..." as` 被执行（as 后无内容）
- **THEN** 系统 SHALL 报语法错误

#### Scenario: syntax as asterisk 错误
- **WHEN** `@forward "..." as *` 被执行（@forward 不允许 as *）
- **THEN** 系统 SHALL 报语法错误

#### Scenario: syntax as no star 错误
- **WHEN** `@forward "..." as prefix` 被执行（as 后无星号前缀）
- **THEN** 系统 SHALL 报语法错误

#### Scenario: 私有变量访问
- **WHEN** 模块定义 `$private-var` 且通过 `@use` 访问 `module.$private-var`
- **THEN** 系统 SHALL 报 "Private members can't be accessed from outside their modules." 错误

#### Scenario: 私有 mixin 访问
- **WHEN** 模块定义 `private-mixin`（以 `-` 或 `_` 开头）且从外部 `@include module.private-mixin`
- **THEN** 系统 SHALL 报私有成员访问错误

#### Scenario: 私有函数访问
- **WHEN** 模块定义 `private-function`（以 `-` 或 `_` 开头）且从外部调用 `module.private-function()`
- **THEN** 系统 SHALL 报私有成员访问错误

#### Scenario: with undefined 变量
- **WHEN** `@forward "..." with ($undefined: value)` 被执行且模块不定义 `$undefined`
- **THEN** 系统 SHALL 报 "Undefined variable" 错误

#### Scenario: with not_default 变量
- **WHEN** `@forward "..." with ($var: value)` 被执行且模块定义 `$var` 不带 `!default`
- **THEN** 系统 SHALL 报 "This variable was not declared with !default" 错误

#### Scenario: with through_forward show
- **WHEN** `@forward "..." with ($var: value) show $var` 组合使用
- **THEN** 系统 SHALL 根据 sass-spec 规则验证 through_forward 配置

#### Scenario: with through_forward hide
- **WHEN** `@forward "..." with ($var: value) hide $var` 组合使用
- **THEN** 系统 SHALL 根据 sass-spec 规则验证 through_forward 配置

#### Scenario: with through_forward as
- **WHEN** `@forward "..." with ($var: value) as prefix-*` 组合使用
- **THEN** 系统 SHALL 根据 sass-spec 规则验证 through_forward 配置

#### Scenario: with through_forward with
- **WHEN** `@forward "..." with ($var: value)` 后接 `@forward "..." with ($var: other)`
- **THEN** 系统 SHALL 根据 sass-spec 规则验证 through_forward 配置

#### Scenario: with multi_configuration one_file
- **WHEN** 同一文件被多次 `@use` 且 with 配置不同
- **THEN** 系统 SHALL 报 "This module was already loaded with a different configuration." 错误

#### Scenario: with multi_configuration multi_file
- **WHEN** 不同文件分别 `@use` 同一模块且 with 配置不同
- **THEN** 系统 SHALL 报多配置冲突错误

#### Scenario: with multi_configuration unconfigured_first
- **WHEN** 先 `@use` 模块不带 with，后 `@use` 同一模块带 with
- **THEN** 系统 SHALL 报多配置冲突错误

#### Scenario: with multi_configuration through_forward
- **WHEN** 通过 @forward 传递 with 配置后再被 @use with 覆盖
- **THEN** 系统 SHALL 报多配置冲突错误

#### Scenario: extend through forward 错误
- **WHEN** `@forward "..."` 后通过 extend 访问转发的成员
- **THEN** 系统 SHALL 根据 sass-spec 规则验证 extend through forward 行为
