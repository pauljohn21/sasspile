## MODIFIED Requirements

### Requirement: @forward 同名成员冲突检测
系统 SHALL 在 `@forward` 指令合并 exports 时检测 forwarded 表内的同名成员冲突。冲突判定基于来源路径：同一来源路径的同名成员为幂等写入不冲突；不同来源路径的同名成员即使值相同也报冲突错误。local 表中的同名成员不参与 forwarded 冲突检测。

#### Scenario: 变量冲突
- **WHEN** 两个 `@forward` 指令分别从不同模块路径转发了同名变量 `$a`
- **THEN** 系统 报 "Two forwarded modules both define a variable named $a." 错误

#### Scenario: 函数冲突
- **WHEN** 两个 `@forward` 指令分别从不同模块路径转发了同名函数 `c`
- **THEN** 系统 报 "Two forwarded modules both define a function named c." 错误

#### Scenario: mixin 冲突
- **WHEN** 两个 `@forward` 指令分别从不同模块路径转发了同名 mixin `b`
- **THEN** 系统 报 "Two forwarded modules both define a mixin named b." 错误

#### Scenario: 同来源不同值不冲突
- **WHEN** 两个 `@forward` 指令从同一模块路径转发了同名成员（值不同由 with() 配置导致）
- **THEN** 系统 不报错（同来源路径幂等写入）

#### Scenario: 不同来源同值也冲突
- **WHEN** 两个 `@forward` 指令分别从不同模块路径转发了同名且同值的变量
- **THEN** 系统 报冲突错误（冲突基于来源路径而非值比较）

#### Scenario: use as star 与 forward 同一模块不冲突
- **WHEN** 一个文件同时执行 `@use 'X' as *` 和 `@forward 'X'`（X 是同一模块路径）
- **THEN** 系统 不报错（`@use as *` 写入 local 表，`@forward` 写入 forwarded 表，不互相干扰）

#### Scenario: 带前缀转发冲突
- **WHEN** 两个 `@forward "... as prefix-*"` 指令从不同模块路径转发了同名成员（加前缀后名称相同）
- **THEN** 系统 报冲突错误

#### Scenario: 混合成员类型不冲突
- **WHEN** 两个 `@forward` 分别从不同模块路径转发了变量 `$a` 和函数 `a`（不同类型同名）
- **THEN** 系统 不报错（不同类型不冲突）
