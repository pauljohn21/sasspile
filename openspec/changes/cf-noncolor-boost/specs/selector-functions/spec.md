## ADDED Requirements

### Requirement: selector-nest 多选择器嵌套展开

`selector-nest` 函数 MUST 将多个父选择器按需展开为嵌套格式。当父选择器列表包含多个以逗号分隔的选择器时, MUST 对每个父选择器单独执行嵌套,并以逗号连接结果。

#### Scenario: 单父选择器嵌套
- **WHEN** 调用 `selector-nest(".a", ".b .c")`
- **THEN** 返回 `".b .c .a"` (等价于 SCSS `{.b .c { &-prefix ... }}` 展开)

#### Scenario: 多父选择器嵌套
- **WHEN** 调用 `selector-nest(".a", ".b, .c")`
- **THEN** 返回 `".b .a, .c .a"` (每个父选择器独立嵌套)

#### Scenario: & 符号插值
- **WHEN** 调用 包含 `&` 的父选择器,如 `selector-nest("a", "&.b")`
- **THEN** `&` 被替换为被嵌套选择器

### Requirement: selector-merge 选择器智能合并

`selector-merge` 函数 MUST 将两个选择器以最合理方式合并。MUST 处理简单组合、后代组合、伪类选择和属性选择器的合并场景。

#### Scenario: 简单合并
- **WHEN** 调用 `selector-merge(".a", ".b")`
- **THEN** 返回可组合的选择器形式

#### Scenario: 不兼容选择器报错或回退
- **WHEN** 调用 `selector-merge(".a", "#b")` (类型不兼容)
- **THEN** 返回合理回退或报错

### Requirement: selector-extend 扩展解析

`selector-extend` 函数 MUST 模拟 `@extend` 的选择器解析和扩展行为, 返回扩展后的选择器列表。

#### Scenario: 基本 extend
- **WHEN** 调用 `selector-extend(".a", ".b", ".c")`
- **THEN** 返回将 `.b` 替换为 `.c` 的选择器

### Requirement: selector-parse 选择器解析为 AST

`selector-parse` 函数 MUST 将选择器字符串解析为可检查/操作的数据结构(match sass-spec 期望的 list/map 格式)。

#### Scenario: 简单 class 选择器
- **WHEN** 调用 `selector-parse(".a")`
- **THEN** 返回表示该选择器的结构化列表

#### Scenario: 复合选择器
- **WHEN** 调用 `selector-parse(".a > .b")`
- **THEN** 返回带 combinator 的嵌套结构

### Requirement: selector-unify 选择器合并

`selector-unify` 函数 MUST 将两个选择器"统一"为一个共同的选择器,类似 `@at-root` 中的选择器提升。

#### Scenario: 统一两个简单选择器
- **WHEN** 调用 `selector-unify(".a", ".b")`
- **THEN** 返回能匹配两者的选择器

### Requirement: is-superselector 选择器超集判断

`is-superselector` 函数 MUST 判断第一个选择器是否是第二个的"超集"匹配(即第一个匹配的所有元素都被第二个匹配)。

#### Scenario: 超集判断为真
- **WHEN** 调用 `is-superselector("a", "a.foo")`
- **THEN** 返回 `true` (因为 `a` 匹配所有 `a` 元素,包括 `a.foo`)

#### Scenario: 超集判断为假
- **WHEN** 调用 `is-superselector("a.foo", "a")`
- **THEN** 返回 `false`

### Requirement: simple-selectors 简单选择器分解

`simple-selectors` 函数 MUST 将复合选择器分解为"简单选择器"组件列表。

#### Scenario: 分解复合选择器
- **WHEN** 调用 `simple-selectors("a.foo#bar")`
- **THEN** 返回表示 `a`, `.foo`, `#bar` 的列表

### Requirement: selector-replace 选择器模式替换

`selector-replace` 函数 MUST 将选择器中匹配模式的部分替换为新内容。

#### Scenario: 基本替换
- **WHEN** 调用 `selector-replace(".a.b", ".a", ".c")`
- **THEN** 返回 `.c.b`
