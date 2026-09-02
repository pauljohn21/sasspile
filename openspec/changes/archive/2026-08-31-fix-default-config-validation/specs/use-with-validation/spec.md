## MODIFIED Requirements

### Requirement: @use with 未定义变量检测

系统 SHALL 在 `@use ... with ($var: value)` 中检测目标模块未定义 `$var` 或 `$var` 未声明 `!default` 的情况，并报错。验证 SHALL 在模块加载完成后执行（运行时消费跟踪），而非加载前静态 AST 检查。

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

#### Scenario: @forward 裸转发配置
- **WHEN** `@use "used" with ($a: configured)` 被执行，且 `_used.scss` 包含 `@forward "forwarded"`（无 with），`_forwarded.scss` 包含 `$a: original !default`
- **THEN** 系统 SHALL 将 `$a` 配置通过 pending_config 传播到 `_forwarded.scss`，且验证 SHALL 通过

#### Scenario: 多跳 @forward 链配置
- **WHEN** `@use "used" with ($a: configured)` 被执行，且 `_used.scss` 包含 `@forward "midstream"`，`_midstream.scss` 包含 `@forward "upstream"`，`_upstream.scss` 包含 `$a: original !default`
- **THEN** 系统 SHALL 将 `$a` 配置穿透多层 `@forward` 到达 `!default` 声明，且验证 SHALL 通过

#### Scenario: @forward with !default 中间覆盖
- **WHEN** `@use "used" with ($a: from input)` 被执行，且 `_used.scss` 包含 `@forward "forwarded" with ($a: from used !default)`，`_forwarded.scss` 包含 `$a: from forwarded !default`
- **THEN** 系统 SHALL 让 `from input` 优先（因为 `@forward with !default` 不覆盖上游配置），输出 `from input`

#### Scenario: @forward as 前缀配置
- **WHEN** `@use "used" with ($b-a: configured)` 被执行，且 `_used.scss` 包含 `@forward "forwarded" as b-*`，`_forwarded.scss` 包含 `$a: original !default`
- **THEN** 系统 SHALL 将 `$b-a` 映射到 `$a` 后传播，且验证 SHALL 通过

#### Scenario: @forward show/hide 过滤配置
- **WHEN** `@use "used" with ($a: configured)` 被执行，且 `_used.scss` 包含 `@forward "forwarded" show $a`（或 `hide $b`），`_forwarded.scss` 包含 `$a: original !default`
- **THEN** 系统 SHALL 验证 `$a` 通过（show/hide 不影响配置传播，只影响成员可见性）

#### Scenario: 分布式变量验证
- **WHEN** `@use "mod" with ($a: x, $b: y, $missing: z)` 被执行，且 `_index.scss` 包含 `@forward "./a/a"; @forward "./b/b"`，`_a/_variables.scss` 包含 `$a: default !default`，`_b/_variables.scss` 包含 `$b: default !default`
- **THEN** 系统 SHALL 验证 `$a` 和 `$b` 通过，但报错 `$missing` 未声明 `!default`

#### Scenario: @forward + 本地 !default 混合
- **WHEN** `@use "used" with ($a: x, $b: y)` 被执行，且 `_used.scss` 包含 `@forward "forwarded" with ($b: from used !default)` 和 `$a: from used !default`，`_forwarded.scss` 包含 `$b: from forwarded !default`
- **THEN** 系统 SHALL 验证 `$a`（本地 !default）和 `$b`（转发 !default）均通过
