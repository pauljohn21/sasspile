## ADDED Requirements

### Requirement: 伪元素语法归一化
系统 SHALL 将单冒号伪元素语法（`:before`、`:after`、`:first-line`、`:first-letter`）与对应双冒号语法（`::before`、`::after`、`::first-line`、`::first-letter`）视为语义等价。比较和统一操作 SHALL 归一化后比较。

#### Scenario: 单双冒号伪元素统一
- **WHEN** 执行 `selector.unify(":before", "::before")`
- **THEN** 结果为 `:before`（归一化为 class syntax）

#### Scenario: 不同伪元素冲突
- **WHEN** 执行 `selector.unify("::before", "::after")`
- **THEN** 结果为 `null`（不同伪元素不可统一）

#### Scenario: 伪元素与伪类区分
- **WHEN** 执行 `selector.unify(":hover", "::hover")`
- **THEN** `:hover` 是伪类、`::hover` 是伪元素，二者不同

### Requirement: 伪类链式合并
`unify_compound` SHALL 支持伪类链式合并：不同 name 的伪类 SHALL 合并为链式（`:c` + `:d` → `:c:d`），相同 name 的伪类 SHALL 后者覆盖前者（`:c(:x)` + `:c(:y)` → `:c(:y)`）。

#### Scenario: 不同名伪类链式
- **WHEN** 执行 `selector.unify(":c", ":d")`
- **THEN** 结果为 `:c:d`

#### Scenario: 同名伪类覆盖
- **WHEN** 执行 `selector.unify(":nth-child(2n)", ":nth-child(3n)")`
- **THEN** 结果为 `:nth-child(3n)`

#### Scenario: 单伪类保持
- **WHEN** 执行 `selector.unify(":c", ":c")`
- **THEN** 结果为 `:c`

### Requirement: 伪元素归一化比较
`is_super_compound` SHALL 在比较伪元素时归一化 `is_class_syntax` 字段——单冒号和双冒号语法视为相同伪元素。

#### Scenario: 归一化后伪元素匹配
- **WHEN** 判断 `:before` 是否是 `::before` 的超选择器
- **THEN** 返回 `true`（归一化后相同）
