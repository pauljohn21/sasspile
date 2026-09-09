## ADDED Requirements

### Requirement: extend NO-OP 命名空间感知
`is_more_specific_than` SHALL 在选择器扩展的 NO-OP 检测中考虑命名空间兼容性。当 extendee 的命名空间与 extender 的命名空间不兼容时， SHALL NOT 判定为 NO-OP。

#### Scenario: 命名空间不兼容不触发 NO-OP
- **WHEN** 执行 `selector.extend("c|*.d", ".d", "e|*")`
- **THEN** 结果为 `"c|*.d"`（命名空间冲突 → NO-OP，保留原始）

#### Scenario: 兼容命名空间正常扩展
- **WHEN** 执行 `selector.extend(".a.b", ".a", ".c")`
- **THEN** 结果为 `".a.b, .c.b"`（扩展发生）

### Requirement: extend compounds_conflict 命名空间感知
`compounds_conflict` SHALL 在比较 remaining simples 与 extender compound 的 Type 时考虑命名空间。不同命名空间的 Type 视为冲突。

#### Scenario: 命名空间 Type 冲突
- **WHEN** remaining 包含 `Type { namespace: Namespace::Explicit("a"), name: "x" }` 且 extender 包含 `Type { namespace: Namespace::Explicit("b"), name: "x" }`
- **THEN** 判定为冲突

#### Scenario: 无命名空间 Type 一致
- **WHEN** remaining 和 extender 包含同名无命名空间 Type
- **THEN** 不判定为冲突

### Requirement: extend 伪元素冲突检测
`compounds_conflict` SHALL 在比较伪元素时归一化 `is_class_syntax`——单冒号和双冒号语法视为相同伪元素。

#### Scenario: 伪元素归一化后不冲突
- **WHEN** remaining 包含 `:before`（class syntax）且 extender 包含 `::before`（element syntax）
- **THEN** 不判定为冲突（归一化后相同）

## MODIFIED Requirements

### Requirement: extend_complex NO-OP 条件
`extend_complex` 的 NO-OP 检测 SHALL 增加条件：当 extender 的命名空间与 selector 中对应位置的命名空间不兼容时，不触发 NO-OP（允许扩展发生）。

#### Scenario: 命名空间不兼容允许扩展
- **WHEN** 执行 `selector.extend("c.d", ".d", "e")` 其中 e 的命名空间与 c 不兼容
- **THEN** 扩展发生（不满足 NO-OP 条件）
