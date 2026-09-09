## MODIFIED Requirements

### Requirement: selector-extend NO-OP 保留空命名空间
`selector-extend()` SHALL 在无操作（no-op）时保留原始选择器的命名空间格式，包括 `Namespace::Empty`（输出 `|name` 格式）。

#### Scenario: 空命名空间无操作
- **WHEN** 调用 `selector-extend(*.c, |*.c)` 返回不操作结果
- **THEN** 输出 `|*.c` 而不是 `*.c`

### Requirement: selector-extend 格式不生成多余选择器
`selector-extend()` SHALL 只生成由 extender 产生的新选择器，不引入未预期的组合。

#### Scenario: 格式输出正确性
- **WHEN** 调用 `selector-extend()` 在格式测试用例上
- **THEN** 结果中的每个 complex selector 都来自原始 selector 或 extender 的正确组合，不包含多余项

### Requirement: selector-extend 复杂选择器统合生成所有排列
`selector-extend()` SHALL 为复杂选择器的每种有效统合方式生成对应扩展结果。

#### Scenario: parent with grandparent 全排列
- **WHEN** 调用 `selector-extend()` 在 parent/with_grandparent/complex 输入上
- **THEN** 生成 `.c .d.x .e`, `.c .f .x.g .e`, `.f .c .x.g .e` 三种组合

### Requirement: selector-extend tail combinator 不重复 compound
`selector-extend()` SHALL 在 extender 有 trailing combinator（`>`, `+`, `~`）时不重复附加 compound。

#### Scenario: trailing child combinator 正确性
- **WHEN** 调用 `selector-extend()` 在 trailing_combinator/extender/child 输入上
- **THEN** 输出 `.c.x .d, .x.e > .d`（无重复 `.d`）
