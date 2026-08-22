## ADDED Requirements

### Requirement: use+import 交互模块加载

系统 SHALL 正确处理 @use 和 @import 组合使用时的模块加载和 CSS 输出顺序。

#### Scenario: use_into_use_and_import_into_use
- **WHEN** 文件 A @use 文件 B（B 中也有 @use），同时文件 A @import 文件 C（C 中也有 @use）
- **THEN** 系统 SHALL 正确输出所有模块的 CSS，不丢失输出

#### Scenario: use_into_use_and_import_into_import
- **WHEN** 文件 A @use 文件 B（B 中有 @use），同时文件 A @import 文件 C（C 中也有 @import）
- **THEN** 系统 SHALL 正确输出所有模块的 CSS

#### Scenario: use_into_use_and_use_into_import
- **WHEN** 文件 A @use 文件 B（B 中有 @use），同时文件 A @use 文件 C（C 中有 @import）
- **THEN** 系统 SHALL 正确输出所有模块的 CSS

#### Scenario: use_into_use_and_use_into_import_into_use
- **WHEN** 文件 A @use 文件 B（B 中有 @use），同时文件 A @use 文件 C（C 中有 @import），C 中有 @use
- **THEN** 系统 SHALL 正确输出所有模块的 CSS

#### Scenario: use_and_import_into_diamond_extend
- **WHEN** 文件 A 同时 @use 和 @import 形成钻石依赖，且包含 @extend
- **THEN** 系统 SHALL 正确合并 @extend 关系并输出 CSS

#### Scenario: isolated_through_import
- **WHEN** @import 加载的文件中包含 @use，且 @use 的模块不应传播到外层
- **THEN** 系统 SHALL 隔离 @import 内的 @use 模块作用域

### Requirement: CSS import 输出位置

系统 SHALL 正确处理 CSS @import 在 @use/@import 交互场景中的输出位置。

#### Scenario: css_import_above_rule
- **WHEN** CSS @import 出现在 CSS 规则上方，且存在 @use 引用
- **THEN** 系统 SHALL 按 sass-spec 规则输出 CSS @import 的正确位置

#### Scenario: css_import_below_rule
- **WHEN** CSS @import 出现在 CSS 规则下方，且存在 @use 引用
- **THEN** 系统 SHALL 按 sass-spec 规则输出 CSS @import 的正确位置

### Requirement: escaped 文件名处理

系统 SHALL 正确处理包含转义字符的 @use/@import/@forward URL。

#### Scenario: escaped URL
- **WHEN** `@use "module\ name"` 被执行（URL 包含转义空格）
- **THEN** 系统 SHALL 正确解析文件路径

### Requirement: null 模块处理

系统 SHALL 正确处理 @use/@forward 中的 null 场景。

#### Scenario: null through_forward
- **WHEN** `@forward "module" with ($var: null)` 被执行
- **THEN** 系统 SHALL 根据 sass-spec 规则处理 null 值传递

#### Scenario: null 模块引用
- **WHEN** `@use "module" with ($var: null)` 被执行
- **THEN** 系统 SHALL 根据 sass-spec 规则处理 null 值配置

### Requirement: 重复 import 处理

系统 SHALL 正确处理同一文件被多次 @import 的场景。

#### Scenario: import_twice with_change
- **WHEN** 同一文件被多次 @import，且文件中变量值在两次导入之间发生变化
- **THEN** 系统 SHALL 根据 sass-spec 规则处理变量变化

#### Scenario: import_twice still_changes_in_same_file
- **WHEN** 同一文件被多次 @import，且变化发生在同一文件内
- **THEN** 系统 SHALL 根据 sass-spec 规则处理

### Requirement: midstream_definition 处理

系统 SHALL 正确处理 @import 中间流定义的场景。

#### Scenario: midstream_definition with_config
- **WHEN** @import 文件中间流定义变量，且带有 with 配置
- **THEN** 系统 SHALL 根据 sass-spec 规则处理中间流定义

#### Scenario: midstream_definition no_config
- **WHEN** @import 文件中间流定义变量，不带 with 配置
- **THEN** 系统 SHALL 根据 sass-spec 规则处理中间流定义

### Requirement: prefixed_as 处理

系统 SHALL 正确处理 @forward 的 `as` 前缀。

#### Scenario: prefixed_as
- **WHEN** `@forward "module" as prefix-*` 被执行
- **THEN** 系统 SHALL 根据 sass-spec 规则正确添加前缀到所有导出成员

### Requirement: 嵌套 @规则输出

系统 SHALL 正确处理 @import 内嵌套 @at-rule 的 CSS 输出。

#### Scenario: nested at_rule keyframes
- **WHEN** @import 文件中包含 `@keyframes` 规则
- **THEN** 系统 SHALL 正确输出嵌套 @keyframes CSS

#### Scenario: nested at_rule childless
- **WHEN** @import 文件中包含无子节点的 @at-rule
- **THEN** 系统 SHALL 正确输出嵌套 @at-rule CSS

#### Scenario: nested at_rule declaration_child
- **WHEN** @import 文件中包含声明子节点的 @at-rule
- **THEN** 系统 SHALL 正确输出嵌套 @at-rule CSS

#### Scenario: nested at_rule rule_child
- **WHEN** @import 文件中包含规则子节点的 @at-rule
- **THEN** 系统 SHALL 正确输出嵌套 @at-rule CSS

#### Scenario: nested with_comment
- **WHEN** @import 文件中包含注释和嵌套 @规则
- **THEN** 系统 SHALL 正确输出注释和 @规则 CSS

### Requirement: 空白处理

系统 SHALL 正确处理 @import 修饰符参数中的空白。

#### Scenario: whitespace modifier args after_open_paren
- **WHEN** `@import "url" (modifier)` 中 `(modifier` 后有额外空白
- **THEN** 系统 SHALL 根据 sass-spec 规则正确处理

#### Scenario: whitespace modifier args before_close_paren
- **WHEN** `@import "url" (modifier)` 中 `modifier)` 前有额外空白
- **THEN** 系统 SHALL 根据 sass-spec 规则正确处理

### Requirement: 注释处理

系统 SHALL 正确处理 @import 修饰符参数中的注释。

#### Scenario: comment modifier args after_open_paren loud
- **WHEN** `@import "url" (/* comment */modifier)` 中 `(modifier` 后有注释
- **THEN** 系统 SHALL 根据 sass-spec 规则正确处理

#### Scenario: comment modifier args before_close_paren loud
- **WHEN** `@import "url" (modifier/* comment */)` 中 `modifier)` 前有注释
- **THEN** 系统 SHALL 根据 sass-spec 规则正确处理

### Requirement: CSS import after style rule

系统 SHALL 正确处理 CSS @import 出现在 style rule 之后的场景。

#### Scenario: css_import_after_style_rule
- **WHEN** CSS @import 出现在 CSS 规则之后
- **THEN** 系统 SHALL 根据 sass-spec 规则正确输出 CSS @import 位置

### Requirement: member inaccessible

系统 SHALL 正确处理 @import 中的 member 不可访问场景。

#### Scenario: member inaccessible nested function
- **WHEN** @import 文件中嵌套函数不可访问
- **THEN** 系统 SHALL 根据 sass-spec 规则处理或报错

### Requirement: for 循环声明内输出

系统 SHALL 正确处理 @for 在声明内使用时的输出格式。

#### Scenario: for in_declaration
- **WHEN** `@for` 循环出现在 CSS 声明内部
- **THEN** 系统 SHALL 根据 sass-spec 规则正确输出生成的声明

### Requirement: at_root 输出

系统 SHALL 正确处理 @at-root 指令的输出格式。

#### Scenario: at_root 基本输出
- **WHEN** `@at-root` 被执行
- **THEN** 系统 SHALL 根据 sass-spec 规则正确输出 CSS

#### Scenario: at_root with_query
- **WHEN** `@at-root (selector: value)` 带查询参数
- **THEN** 系统 SHALL 根据 sass-spec 规则正确输出

#### Scenario: at_root with_media
- **WHEN** `@at-root (media: ...)` 带媒体查询
- **THEN** 系统 SHALL 根据 sass-spec 规则正确输出

#### Scenario: at_root with_supports
- **WHEN** `@at-root (supports: ...)` 带 supports 查询
- **THEN** 系统 SHALL 根据 sass-spec 规则正确输出

### Requirement: if 分支输出

系统 SHALL 正确处理 @if/@else if/@else 分支逻辑。

#### Scenario: if 基本分支
- **WHEN** `@if true { ... }` 被执行
- **THEN** 系统 SHALL 根据 sass-spec 规则正确输出

#### Scenario: if else if 分支
- **WHEN** `@if false { ... } @else if true { ... }` 被执行
- **THEN** 系统 SHALL 根据 sass-spec 规则正确输出

#### Scenario: if else 分支
- **WHEN** `@if false { ... } @else { ... }` 被执行
- **THEN** 系统 SHALL 根据 sass-spec 规则正确输出

### Requirement: default 值处理

系统 SHALL 正确处理 @for 循环中的 default 值。

#### Scenario: for default
- **WHEN** `@for $i from 1 through 5` 使用默认值
- **THEN** 系统 SHALL 根据 sass-spec 规则正确输出
