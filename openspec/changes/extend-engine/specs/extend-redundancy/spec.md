## ADDED Requirements

### Requirement: 冗余选择器消除

系统 SHALL 在应用 extends 后检测并移除被包含（superselector）的选择器，避免输出冗余选择器。

#### Scenario: 超集选择器消除子集

- **GIVEN** 选择器列表 `.foo.bar, .x, .y`，其中 `.foo` 是 `.foo.bar` 的 superselector
- **WHEN** 系统执行冗余消除
- **THEN** 系统 MUST 保留 `.foo.bar`（更具体的）
- **AND** 系统 MUST NOT 移除 `.x` 和 `.y`（它们不是 `.foo.bar` 的子集）

#### Scenario: 重复选择器合并

- **GIVEN** extender `.a .b` extends `.e`，`.a.a` 被生成
- **WHEN** 系统检测到 `.a.a` 中的重复
- **THEN** 系统 MUST 简化为 `.a`
- **AND** 输出 MUST NOT 包含 `.a.a`

#### Scenario: 自扩展不改变选择器

- **GIVEN** `.c, .a .b .c, .a .c .b {@extend .c}`
- **WHEN** 系统应用 extend 并检查冗余
- **THEN** 输出 MUST 保持 `.c, .a .b .c, .a .c .b` 不变
- **AND** 系统 MUST NOT 添加额外选择器（已有 .c 在列表中）

### Requirement: Placeholder 移除

系统 SHALL 在应用 extends 后从输出中移除所有未被任何非 placeholder 规则引用的 placeholder 选择器（`%foo`）。

#### Scenario: placeholder 被扩展后消失

- **GIVEN** `%foo {a: b}` 和 `.bar {@extend %foo}`
- **WHEN** 系统应用 extends
- **THEN** 输出 MUST 为 `.bar {a: b}`
- **AND** 输出 MUST NOT 包含 `%foo`（placeholder 被移除）

#### Scenario: 未被引用的 placeholder 不输出

- **GIVEN** `%unused {a: b}`（无任何 `@extend %unused`）
- **WHEN** 系统序列化 CSS
- **THEN** 输出 MUST NOT 包含 `%unused` 或其声明

#### Scenario: placeholder compound 混合

- **GIVEN** `%in-other.a {x: y}` 和 `.b {@extend %in-other}`
- **WHEN** 系统应用 extend 并移除 placeholder
- **THEN** 输出 MUST 为 `.b.a {x: y}` 或等价形式
- **AND** `%in-other` 部分 MUST 被移除，`.a` 部分必须保留
