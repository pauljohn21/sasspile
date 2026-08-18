## ADDED Requirements

### Requirement: @extend 传递性解析

系统 SHALL 实现 `@extend` 的传递性解析：如果 A extends B 且 B extends C，则 A 也 extends C。

#### Scenario: 简单传递性

- **GIVEN** `in-other-extender {@extend in-other-extendee}` 和 `in-input {@extend in-other-extender}`
- **WHEN** 系统构建 extend 依赖图并执行传递性传播
- **THEN** `in-input` MUST 被视为也 extends `in-other-extendee`
- **AND** 规则 `in-other-extendee {x: y}` 的选择器 MUST 变为 `in-other-extendee, in-other-extender, in-input`

#### Scenario: 多级传递

- **GIVEN** A extends B，B extends C，C extends D
- **WHEN** 系统执行传递性 BFS
- **THEN** A MUST 被视为 extends D
- **AND** 规则 `D {x: y}` MUST 包含 A、B、C、D 四个选择器

### Requirement: 循环检测与截断

系统 SHALL 检测 `@extend` 循环引用（A extends B 且 B extends A），避免无限递归。

#### Scenario: 直接循环

- **GIVEN** `.foo {@extend .bar}` 和 `.bar {@extend .foo}`
- **WHEN** 系统执行传递性 BFS
- **THEN** 系统 MUST 检测到循环并停止递归
- **AND** 系统 MUST NOT 无限递归或 panic
- **AND** 输出 MUST 包含 `.foo, .bar`（每个选择器出现一次）

#### Scenario: 多元素循环

- **GIVEN** A extends B，B extends C，C extends A
- **WHEN** 系统执行传递性解析
- **THEN** 系统 MUST 正确检测三元循环
- **AND** 每个选择器 MUST 在结果中只出现一次

#### Scenario: extend 结果可被 extend

- **GIVEN** `:not(.c) {x: y}`，`.a {@extend :not(.b)}`，`.b {@extend .c}`
- **WHEN** 系统应用 extends 并传递
- **THEN** `:not(.c)` 的 extend 结果 `:not(.c):not(.b)` MUST 自身可被 extend
- **AND** 最终输出 MUST 为 `:not(.c):not(.b), .a:not(.c) {x: y}`

### Requirement: 顺序无关性

系统 SHALL 保证 `@extend` 的传递性解析结果与 extend 声明顺序无关。

#### Scenario: 不同声明顺序产生相同结果

- **GIVEN** 6 种不同顺序的 extend 链（A→B→C 循环，6 种排列）
- **WHEN** 系统分别编译 6 种顺序
- **THEN** 每种顺序的输出 MUST 在语义上等价（选择器集合相同）
- **AND** 选择器列表的顺序可能不同但集合 MUST 相同
