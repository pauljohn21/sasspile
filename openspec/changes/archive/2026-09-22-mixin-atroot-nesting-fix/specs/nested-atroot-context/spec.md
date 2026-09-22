## ADDED Requirements

### Requirement: Nested at-root selector context inheritance
当 `@at-root` mixin 内部嵌套调用另一个 `@at-root` mixin 时，系统 SHALL 确保内层 mixin 生成的选择器继承外层 mixin 的 `@at-root` 选择器上下文，生成正确的组合选择器。

#### Scenario: BEM m() 嵌套 e() — single level
- **WHEN** `@include m(large) { @include e(header) { margin-bottom: 20px; } }` 在外层 `.el-descriptions { ... }` 内调用
- **THEN** 输出选择器为 `.el-descriptions--large .el-descriptions__header { margin-bottom: 20px; }`
- **AND** 不是 `.el-descriptions__header { margin-bottom: 20px; }`（缺少前缀）

#### Scenario: BEM m() 嵌套 e() — with declarations
- **WHEN** 外层 `@content` 同时包含声明和嵌套 `@include`
- **THEN** 声明 `@include m(large)` 位置正确输出 `.el-descriptions--large { font-size: 14px; }`
- **AND** 嵌套 `@include e(header)` 正确输出 `.el-descriptions--large .el-descriptions__header { ... }`

#### Scenario: Multiple size variants
- **WHEN** `@each $size in (large, small)` 循环调用 `@include m($size)`
- **THEN** 每个 size 的 `@include e(header)` 选择器前缀正确（`.el-descriptions--large` / `.el-descriptions--small`）
- **AND** 不同 size 的同名元素选择器不混淆

### Requirement: AtRootDirect preserves nested child structure
当 `exec_mixin` 处理含 `@at-root` 的 mixin body 时，系统 SHALL 保留 `AtRootDirect` 节点的嵌套子节点结构，而非将所有子节点扁平化为兄弟节点。

#### Scenario: AtRootDirect with child rules
- **WHEN** `@mixin m(large)` body 包含 `@at-root { &--large { @content } }` 且 `@content` 含 `@include e(header)`
- **THEN** exec_mixin 输出 `AtRootDirect(Rule("&--large", children=[Rule(".descriptions__header", ...)]))`
- **AND** 不输出 `[AtRootDirect(Rule("&--large")), AtRootDirect(Rule(".descriptions__header")), ...]` 扁平结构

#### Scenario: Declaration vs Rule separation
- **WHEN** `@at-root` 内部同时包含 Declaration 和 Rule 节点
- **THEN** Declaration 保持原样（不包裹在 AtRootDirect 中）
- **AND** Rule 节点包裹在 AtRootDirect 中并保留嵌套层级

### Requirement: Recursive child selector combination
当 `RuleBuilder::push` 处理含 `&` 的 `AtRootDirect` 节点时，系统 SHALL 递归地将子节点的选择器与组合后的外层选择器组合。

#### Scenario: AtRootDirect with & selector and child Rules
- **WHEN** `AtRootDirect(Rule("&--large", children=[Rule(".header", ...)]))` 被 push 到 selector=".el-descriptions" 的 RuleBuilder
- **THEN** 外层选择器组合为 `.el-descriptions--large`
- **AND** 子节点选择器组合为 `.el-descriptions--large .header`
- **AND** 孙节点（如有）继续递归组合

#### Scenario: AtRootDirect with non-& selector
- **WHEN** `AtRootDirect(Rule(".foo", children=[Rule(".bar", ...)]))` selector 不含 `&`
- **THEN** 直接 push 不组合（维持现有行为）

### Requirement: EP BEM ordering preserved
重构后的 `exec_mixin`  SHALL 保留 EP BEM mixin 链的源码位置排序语义——`@include` 处生成的规则位于源码位置，而非 RuleBuilder 默认的固定位置。

#### Scenario: Mixin call ordering
- **WHEN** 多个 `@include` 在源码中顺序出现
- **THEN** 输出 CSS 中规则的顺序与源码顺序一致
- **AND** 不因子节点组合改变原有排序

## MODIFIED Requirements

（无现有能力修改）

## REMOVED Requirements

（无）
