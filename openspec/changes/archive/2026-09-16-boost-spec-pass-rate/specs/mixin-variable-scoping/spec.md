## ADDED Requirements

### Requirement: Mixin 参数通过局部作用域链注入
Mixin `@include` 调用时，参数 MUST 绑定到局部作用域栈，查找优先级高于全局变量，实现词法作用域语义。

#### Scenario: 参数正确替换
- **WHEN** SCSS 源码为 `@mixin pad($x) { padding: $x; } .box { @include pad(10px); }`
- **THEN** 输出 MUST 包含 `padding: 10px;`

#### Scenario: 多个参数空格分隔
- **WHEN** SCSS: `@mixin dual($a, $b) { margin: $a $b; } .box { @include dual(5px 10px); }`
- **THEN** args 应为 `["5px", "10px"]`，输出 MUST 为 `margin: 5px 10px;`

#### Scenario: 参数含默认值
- **WHEN** SCSS: `@mixin shadow($blur: 5px) { box-shadow: 0 0 $blur; } .box { @include shadow; }`
- **THEN** MUST 使用默认值，输出 `box-shadow: 0 0 5px;`

### Requirement: 嵌套 mixin 作用域隔离
嵌套 `@include` MUST 创建独立作用域帧，内层 mixin 的局部变量 MUST NOT 泄漏到外层或全局。

#### Scenario: 嵌套 mixin 隔离
- **WHEN** SCSS: `@mixin a($x) { width: $x; } @mixin b($x) { @include a($x); height: $x; } .box { @include b(10px); }`
- **THEN** `.box` MUST 展开为 `width: 10px; height: 10px;`

#### Scenario: 参数作用域不泄漏
- **WHEN** SCSS: `$x: global; @mixin m($x) { content: $x; } .box { @include m(local); }`
- **THEN** mixin body 中 `$x` MUST 为 `local`，全局 `$x` MUST 保持 `global`

### Requirement: MixinCall 参数列表拆分
parse 层 MUST 正确拆分 `@include name(arglist)` 的参数列表，支持空格分隔与逗号分隔，尊重括号嵌套与引号。

#### Scenario: 空格分隔
- **WHEN** `@include dual(5px 10px)`
- **THEN** `args` MUST 为 `["5px", "10px"]`

#### Scenario: 逗号分隔
- **WHEN** `@include dual(5px, 10px)`
- **THEN** `args` MUST 为 `["5px", "10px"]`

#### Scenario: 括号嵌套
- **WHEN** `@include fn(a (b, c) d)`
- **THEN** MUST 正确识别嵌套括号，args 为 `["a (b, c) d"]`
