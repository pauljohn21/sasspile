## ADDED Requirements

### Requirement: :not() extend 追加语义
当选择器包含 `:not()` 伪类且 extendee 匹配 `:not()` 内部时，系统 SHALL 追加新的 `:not()` 到 compound 而非替换内部。

#### Scenario: 简单 :not() 扩展
- **WHEN** extend(":not(.c)", ".c", ".d")
- **THEN** 结果为 ":not(.c):not(.d)"

#### Scenario: :not() 扩展 extender 为列表
- **WHEN** extend(":not(.c)", ".c", ".d, .e")
- **THEN** 结果为 ":not(.c):not(.d):not(.e)"

#### Scenario: :not() 内部为复杂选择器
- **WHEN** extend(":not(.c .d)", ".d", ".e .f")
- **THEN** 结果为 ":not(.c .d):not(.c .e .f):not(.e .c .f)"

#### Scenario: :not() 内部为 compound
- **WHEN** extend(":not(.c.d)", ".c", ".e")
- **THEN** 结果为 ":not(.c.d):not(.d.e)"

#### Scenario: :not() 已包含列表
- **WHEN** extend(":not(.c, .d)", ".c", ".e")
- **THEN** 结果为 ":not(.c, .e, .d)"（添加到现有列表）

#### Scenario: :not() 内部为 :is()
- **WHEN** extend(":not(.c)", ".c", ":is(.d, .e)")
- **THEN** 结果为 ":not(.c):not(.d):not(.e)"

#### Scenario: :not() 内部为 :where()
- **WHEN** extend(":not(.c)", ".c", ":where(.d .e, .f .g)")
- **THEN** 结果为 ":not(.c):not(.d .e):not(.f .g)"

#### Scenario: :not() 内部为 :matches()
- **WHEN** extend(":not(.c)", ".c", ":matches(.d, .e)")
- **THEN** 结果为 ":not(.c):not(.d):not(.e)"

### Requirement: :not() 扩展时 extender 包含伪类
当 extender 本身包含伪类时，系统 SHALL 整体添加为新的 `:not()`。

#### Scenario: extender 包含 :is() 在 compound 内
- **WHEN** extend(":not(.c)", ".c", ".d:is(.e, .f)")
- **THEN** 结果为 ":not(.c):not(.d:is(.e, .f))"

#### Scenario: extender 包含 :matches() 在 compound 内
- **WHEN** extend(":not(.c)", ".c", ".d:matches(.e, .f)")
- **THEN** 结果为 ":not(.c):not(.d:matches(.e, .f))"

#### Scenario: extender 包含 :where() 在 compound 内
- **WHEN** extend(":not(.c)", ".c", ".d:where(.e, .f)")
- **THEN** 结果为 ":not(.c):not(.d:where(.e, .f))"
