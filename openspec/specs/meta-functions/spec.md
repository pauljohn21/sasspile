## ADDED Requirements

### Requirement: type-of 返回正确的类型标识

`meta.type-of` MUST 返回值的精确类型名: `number`, `string`, `bool`, `color`, `list`, `map`, `null`, `calculation`, `function`, `mixin`, `argument list`。

#### Scenario: 数字类型
- **WHEN** 调用 `type-of(42)`
- **THEN** 返回 `"number"`

#### Scenario: 空列表类型
- **WHEN** 调用 `type-of(())`
- **THEN** 返回 `"list"`

#### Scenario: 匿名函数
- **WHEN** 调用 `type-of(get-function("type-of"))`
- **THEN** 返回 `"function"`

### Requirement: module-variables 列出模块内变量

`meta.module-variables` MUST 返回指定模块中所有定义的全局变量名列表。

#### Scenario: 模块变量列表
- **WHEN** 调用 `module-variables("module-name")` 在已定义变量的模块上
- **THEN** 返回变量名列表(逗号分隔字符串)

### Requirement: module-functions 列出模块内函数

`meta.module-functions` MUST 返回指定模块中所有定义的函数名和参数规格的列表。

#### Scenario: 模块函数列表
- **WHEN** 调用 `module-functions("module-name")` 在已定义函数的模块上
- **THEN** 返回函数签名列表

### Requirement: get-function 获取函数引用

`meta.get-function` MUST 返回指定名称的内建/用户函数引用。通过 `call()` 调用必须能正常执行该函数。

#### Scenario: 获取内建函数
- **WHEN** 调用 `call(get-function("type-of"), 42)`
- **THEN** 返回 `"number"`

#### Scenario: 获取用户定义函数
- **WHEN** 调用 `call(get-function("my-fn"), $x)` 在定义了 `@function my-fn` 的上下文中
- **THEN** 正常执行返回结果

#### Scenario: 不存在的函数报错
- **WHEN** 调用 `get-function("nonexistent")`
- **THEN** 返回合理的错误

### Requirement: keywords 获取 mixin 关键字参数

`meta.keywords` MUST 返回当前 mixin 调用时传入的关键字参数 map。

#### Scenario: 有 kwargs 的 mixin
- **WHEN** 在 `@mixin foo($x, $y)` 内部调用 `keywords(($x: 1, $y: 2))`
- **THEN** 返回 `$x: 1, $y: 2` 的 map

#### Scenario: 无 kwargs 的 mixin
- **WHEN** 在 `@mixin foo($x)` 内部调用 `keywords(($x: 1))`
- **THEN** 返回 `$x: 1` 的 map

### Requirement: content-exists 检查 @content 是否传入

`meta.content-exists` MUST 在 mixin 被带 `@content` 调用时返回 `true`,不带时返回 `false`。

#### Scenario: mixin with content
- **WHEN** `@include my-mixin { color: red; }` 后,在 mixin 体内调用 `content-exists()`
- **THEN** 返回 `true`

#### Scenario: mixin without content
- **WHEN** `@include my-mixin` 后,在 mixin 体内调用 `content-exists()`
- **THEN** 返回 `false`

### Requirement: inspect 返回值的调试表示

`meta.inspect` MUST 返回值的 Sass 代码字面表示形式(可用于 re-parse)。

#### Scenario: 字符串 inspect
- **WHEN** 调用 `inspect("hello")`
- **THEN** 返回 `"hello"`(带引号)

#### Scenario: map inspect
- **WHEN** 调用 `inspect((a: 1, b: 2))`
- **THEN** 返回 `(a: 1, b: 2)` 的字面表示
