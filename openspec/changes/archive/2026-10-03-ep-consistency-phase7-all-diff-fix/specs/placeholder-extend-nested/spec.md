## ADDED Requirements

### Requirement: Nested rule placeholder extend
当规则内嵌另一规则且内层规则 `@extend %ph` 时，系统 SHALL 正确将 %ph 声明提升到正确层级，而非顶层。

#### Scenario: Nested single placeholder extend
- **WHEN** 外层 `.a { @extend %x; .b { @extend %y; } }`
- **THEN** `.a` 获得 %x 声明，`.b` 获得 %y 声明，两层级各自正确

#### Scenario: Nested multiple extenders
- **WHEN** 嵌套规则内有 `@extend %size; @extend %flex;`
- **THEN** 所有 %placeholder 声明在对应嵌套层级合并到 extender 选择器

### Requirement: Compound selector chain placeholder extend
当 %placeholder 定义在复合选择器链（如 `.a .b`）内且被 extend 时，系统 SHALL 保留完整选择器链上下文。

#### Scenario: Deep nested extend in description/form components
- **WHEN** `descriptions.scss` 中 `.el-descriptions__item @extend %complex-ph`
- **THEN** extend 后的声明应用保持 `.el-descriptions__item` 祖先链

### Requirement: Multiple placeholder interaction
当同一规则 extend 多个 %placeholder 且这些 placeholder 之间也存在 extend 关系时，系统 SHALL 保证声明合并顺序与 dart-sass 一致。

#### Scenario: Checkbox/Radio with chained extends
- **WHEN** `@extend %shared; @extend %specific;` 且 `%specific` 内部含 `@extend %shared`
- **THEN** 声明按正确 cascading 顺序输出，无重复

## ADDED Requirements

### Requirement: Placeholder extend in mixin with nested context
当 mixin 内 `@extend %ph` 且该 mixin 在嵌套规则中被 include 时，系统 SHALL 确保 extend 条目传播到正确的作用域层级（基于 content_env 计算的增量）。

#### Scenario: Form-item with nested mixin extend
- **WHEN** `form-item.scss` 中 mixin `@include` 嵌套在 `.el-form-item__content { ... }` 内，mixin 含 `@extend %shared-style`
- **THEN** extend 传播仅在当前嵌套层级新增，不污染外层
