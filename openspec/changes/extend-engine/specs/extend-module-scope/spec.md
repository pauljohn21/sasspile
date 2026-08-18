## ADDED Requirements

### Requirement: 模块 ID 与依赖图

系统 SHALL 为每次 `@use` 进入的模块分配唯一 `ModuleId`，并维护模块依赖图记录 `@use` 关系。

#### Scenario: 模块 ID 分配

- **WHEN** `eval_use_rule` 进入一个新模块
- **THEN** 系统 MUST 分配一个新的唯一 `ModuleId`
- **AND** 该 `ModuleId` MUST 被压入 `ExtensionStore` 的模块栈
- **AND** 当前模块内的所有 `@extend` MUST 使用此 `ModuleId` 标记来源

#### Scenario: 模块依赖图构建

- **WHEN** 模块 M1 `@use` 模块 M2
- **THEN** 系统 MUST 在 `module_graph` 中记录边 M1 → M2
- **AND** `downstream(M1)` MUST 包含 M2 及 M2 递归 `@use` 的所有模块

### Requirement: extend 作用域隔离

系统 SHALL 根据 `Extension` 的 `module_id` 和 CSS 规则的来源模块，确定 extend 是否可影响该规则。

#### Scenario: 兄弟模块 extend 隔离

- **GIVEN** `input.scss @use "left"; @use "right"`，left 和 right 互不 `@use`
- **WHEN** left 中有 `@extend right-extendee !optional`
- **THEN** 该 extend MUST NOT 影响 right 的 CSS 规则
- **AND** 因为 `!optional`，该 extend MUST 静默失败（不报错）

#### Scenario: 下游模块 extend 生效

- **GIVEN** `input.scss @use "other"`，`_other.scss` 中有 `@extend in-input`
- **WHEN** other 模块的 extend 检查能否影响 input 的规则
- **THEN** 系统 MUST 允许此 extend（other 是 input 的下游，input @use other）
- **AND** `in-input {x: y}` 的选择器 MUST 变为 `in-input, in-other`

#### Scenario: diamond 模式 extend 合并

- **GIVEN** `input @use "left"; @use "right"`，left 和 right 都 `@use "shared"`，left 和 right 都 `@extend in-shared`
- **WHEN** left 和 right 的 extends 检查能否影响 shared 的规则
- **THEN** 两者 MUST 都被允许（left/right 都是 shared 的下游）
- **AND** `in-shared {x: y}` 的选择器 MUST 变为 `in-shared, right-extendee, left-extendee`

#### Scenario: 私有 placeholder 跨模块

- **GIVEN** `input @use "other"`，`_other.scss` 有 `%-in-other {x: y}` 和 `in-other {@extend %-in-other}`
- **WHEN** input 中有 `@extend %-in-other !optional`
- **THEN** 该 extend MUST 静默失败（`%-` 前缀的 placeholder 只在定义模块内可见）
- **AND** 输出 MUST 只包含 `in-other {x: y}`（不带 input 的选择器）
