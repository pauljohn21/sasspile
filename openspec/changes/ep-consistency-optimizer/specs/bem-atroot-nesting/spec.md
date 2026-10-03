## ADDED Requirements

### Requirement: BEM 嵌套选择器作用域在 @at-root 链中正确解析
当 `@mixin e()` 在 `@at-root` 上下文内链式嵌套调用时（如 `@include e(list) { @include e(item) {} }`），内层 mixin 捕获的 `&`  SHALL 解析为当前 `@at-root` body 的选择器，而非外层 parent 选择器。

#### Scenario: 双层 BEM 嵌套与修饰符组合
- **WHEN** SCSS 包含 `.block { &--modifier { @include e(list) { @include e(item) { padding: 0; } } } }`
- **THEN** 输出 SHALL 为 `.block--modifier .block__list .block__item { padding: 0; }`（三层完整链）

#### Scenario: 单层 BEM 嵌套不受影响
- **WHEN** SCSS 包含 `.block { @include e(item) { color: red; } }`（无嵌套 e()）
- **THEN** 输出 SHALL 为 `.block__item { color: red; }`（单层不变）

#### Scenario: @mixin m() 修饰符后接 @mixin e() 元素
- **WHEN** SCSS 包含 `.block { @include m(modifier) { @include e(item) { color: red; } } }`
- **THEN** 输出 SHALL 为 `.block--modifier .block__item { color: red; }`（m() 修饰符 + e() 元素）

#### Scenario: 三层 BEM 极深嵌套
- **WHEN** SCSS 包含 `.block { &--mod { @include e(a) { @include e(b) { @include e(c) { color: red; } } } } }`
- **THEN** 输出 SHALL 为 `.block--mod .block__a .block__b .block__c { color: red; }`（四层完整链）

### Requirement: AtRootDirect 在 flatten 阶段传递正确 parent selector
当 `CssNode::AtRootDirect` 通过 `nest_rule_in_children` 或直接 push 到 result 时，其内部子节点的 parent selector SHALL 为其自身的 selector 与外层 parent selector的组合，而非仅使用外层 selector。

#### Scenario: AtRootDirect 嵌套 Rule 的 parent 计算
- **WHEN** 一个 `AtRootDirect(Rule(".a__list"))` 内嵌套 `AtRootDirect(Rule(".a__list .a__item"))`，外层 parent 为 `.a--horizontal`
- **THEN** 内层 Rule 的最终 selector SHALL 为 `.a--horizontal .a__list .a__item`

#### Scenario: AtRootDirect 内无嵌套时保持原 selector
- **WHEN** `AtRootDirect(Rule(".a__list"))` 无内层嵌套
- **THEN** 输出 selector SHALL 为 `.a--horizontal .a__list`

### Requirement: 不破坏现有非 BEM 嵌套规则
普通嵌套规则（非 `@mixin e()` BEM 模式）的行为 SHALL 保持不变。

#### Scenario: 普通类选择器嵌套
- **WHEN** SCSS 包含 `.parent { .child { color: red; } }`
- **THEN** 输出 SHALL 为 `.parent .child { color: red; }`（标准后代组合）

#### Scenario: & 父引用基本用法
- **WHEN** SCSS 包含 `.a { &--b { color: red; } }`（sass-spec basic 模式）
- **THEN** 输出 SHALL 为 `.a--b { color: red; }`（父引用替换）

#### Scenario: @at-root 语义不变
- **WHEN** SCSS 包含 `.a { @at-root .b { color: red; } }`
- **THEN** 输出 SHALL 为 `.b { color: red; }`（@at-root 脱离父选择器）
