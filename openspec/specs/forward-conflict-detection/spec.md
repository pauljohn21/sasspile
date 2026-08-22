# forward-conflict-detection Specification

## Purpose
TBD - created by archiving change spec-pass-rate-boost-2. Update Purpose after archive.
## Requirements
### Requirement: @forward 同名成员冲突检测
系统 SHALL 在多个 @forward 指令合并 exports 时检测同名成员冲突并报错。

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

#### Scenario: 带前缀转发冲突
- **WHEN** 两个 `@forward "... as prefix-*"` 指令转发了同名成员（加前缀后名称相同）
- **THEN** 系统 报冲突错误

#### Scenario: 混合成员类型不冲突
- **WHEN** 两个 `@forward` 分别转发了变量 `$a` 和函数 `a`（不同类型同名）
- **THEN** 系统 不报错（不同类型不冲突）

