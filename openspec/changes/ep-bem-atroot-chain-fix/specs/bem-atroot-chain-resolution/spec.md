## ADDED Requirements

### Requirement: `&` 引用在 nested BEM mixin 中正确展开
当 `e()` mixin 在复合选择器块（`&.block--mod` 或 `when(state)` 内部）被调用时，`@at-root { & { __elem { @content } } }` 中的 `&` SHALL 展开为 mixin 调用时刻的完整父选择器链（包含 block + 累积 modifier/state）。

#### Scenario: e() 在 &.block--mod 内
- **WHEN** SCSS 为 `b(block) { &.block--mod { e(elem) { color: red; } } }`
- **THEN** 输出 `.el-block.el-block--mod .el-block__elem{color:red}`（正确单前缀，无重复）

#### Scenario: e() 在 when(state) 内
- **WHEN** SCSS 为 `b(block) { @include when(active) { e(elem) { color: green; } } }`
- **THEN** 输出 `.el-block.is-active .el-block__elem{color:green}`（正确包含 `.is-active`）

#### Scenario: 嵌套 when → e
- **WHEN** SCSS 为 `b(block) { when(disabled) { when(focus) { e(input) { } } } }`
- **THEN** 输出 `.el-block.is-disabled.is-focus .el-block__input{...}`（完整链保留）

### Requirement: AtRoot 输出不被外层规则重复嵌套
当 `e()` mixin 生成的 AtRoot 节点已经包含完整父选择器链时，外层 `nest_rule_in_children` 或 `combine_selectors` 不应再次累加父前缀。

#### Scenario: wrapper-skip 路径已有完整链
- **WHEN** `eval_at_root` 的 wrapper-skip 分支已处理 `e()` 包装 Rule 并注入 selector_chain
- **THEN** AtRoot 输出节点的子 Rules 选择器 SHALL 已包含完整父链，外层 `nest_rule_in_children` 检测到后 SKIP combine

#### Scenario: 非 wrapper 路径 e() 产出的 AtRoot
- **WHEN** mixin body 直接展开 `@at-root { #{$selector} { #{$currentSelector} { @content } } }`（无 wrapper Rule）
- **THEN** `$selector` 中的 `&` SHALL 被替换为 env 的完整 current_selector（包含 block + modifier）

### Requirement: m() modifier mixin 在嵌套上下文中保持链完整
当 `m()` mixin 在嵌套规则内部调用时，`$selector: &` 捕获的引用 SHALL 包含完整父链。

#### Scenario: m() 在 e() 内
- **WHEN** SCSS 为 `b(block) { e(elem) { m(big) { font-size: 20px; } } }`
- **THEN** 输出 `.el-block__element.el-block__element--big{font-size:20px}`（正确）

#### Scenario: m() 在 &.block--mod 内
- **WHEN** SCSS 为 `b(block) { &.block--mod { m(active) { } } }`
- **THEN** modifier 选择器 SHALL 基于完整链 `.el-block.el-block--mod` 展开

### Requirement: sass-spec 兼容性保持
修改 `eval_at_root` 的 `&` 展开逻辑时，不破坏 sass-spec 现有通过测试。

#### Scenario: 普通 @at-root 无 mixin
- **WHEN** SCSS 为 `.foo { @at-root .bar { color: red; } }`
- **THEN** 输出 `.bar{color:red}`（@at-root 提升至根，不继承父）

#### Scenario: @at-root 带 &
- **WHEN** SCSS 为 `.foo { @at-root &.bar { color: red; } }`
- **THEN** 输出 `.foo.bar{color:red}`
