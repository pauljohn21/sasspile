## MODIFIED Requirements

### Requirement: 模块变量在 Env 中的查找
系统 SHALL 在变量查找时检测变量名中的 `.` 分隔符，并从对应命名空间的成员中获取值。系统 SHALL 维护 local 和 forwarded 双层成员表：local 表存储当前文件定义和 `@use as *` 导入的成员（当前文件可见），forwarded 表存储 `@forward` 导出的成员（当前文件不可见，只传递给下游）。查找时只查 local 表。

#### Scenario: 命名空间变量查找
- **WHEN** 变量名为 `ns.name` 且 `env.namespaces` 中存在 `ns` 模块
- **THEN** 系统 从 `ns` 模块的成员中查找 `name` 并返回值

#### Scenario: 命名空间不存在
- **WHEN** 变量名为 `ns.name` 但 `env.namespaces` 中不存在 `ns`
- **THEN** 系统 报 "Undefined variable: $ns.name" 错误

#### Scenario: forwarded 成员在当前文件不可见
- **WHEN** 文件执行 `@forward "other"` 后直接访问 `$c`（`other` 模块定义了 `$c`）
- **THEN** 系统 报 "Undefined variable" 错误（forwarded 成员不在 local 表中，不可见）

#### Scenario: use as star 导入的成员在当前文件可见
- **WHEN** 文件执行 `@use "other" as *` 后访问 `$c`（`other` 模块定义了 `$c`）
- **THEN** 系统 返回 `$c` 的值（`@use as *` 写入 local 表，可见）

### Requirement: 模块变量在 @forward 中的传递
系统 SHALL 在 `@forward` 指令中将上游模块的成员传递到下游模块的 forwarded 成员表中。传递时 local 成员优先于 forwarded 成员（local 遮蔽 forwarded）。

#### Scenario: forward 变量
- **WHEN** 模块 B 执行 `@forward "A"`
- **THEN** 模块 A 的成员通过 B 的 forwarded 表传递，下游 `@use B` 可通过命名空间访问

#### Scenario: local 遮蔽 forwarded
- **WHEN** 模块 B 同时执行 `@forward "A"`（A 定义 `$c: upstream`）和 `$c: midstream`（local 定义）
- **THEN** 下游 `@use B` 访问 `B.$c` 时返回 `midstream`（local 遮蔽 forwarded）

#### Scenario: local 成员不泄漏到 forwarded
- **WHEN** 模块 B 执行 `@use 'A' as *`（A 的函数写入 B 的 local 表），然后 `@forward 'C'`
- **THEN** 模块 B 的 forwarded 表不包含 A 的函数（`@use as *` 写入 local，不参与 forward 传递）

### Requirement: @forward show/hide 过滤
系统 SHALL 在 `@forward` 指令中支持 `show` 和 `hide` 子句过滤转发的成员。

#### Scenario: show 过滤
- **WHEN** `@forward "upstream" show $c` 且 upstream 定义了 `$c` 和 `$d`
- **THEN** 只有 `$c` 被转发，`$d` 不被转发

#### Scenario: hide 过滤
- **WHEN** `@forward "upstream" hide b` 且 upstream 定义了 mixin `a` 和 `b`
- **THEN** `a` 被转发，`b` 不被转发
