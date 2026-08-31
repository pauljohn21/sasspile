## Purpose

defines sasspile compiler specs.
## Requirements
### Requirement: meta.load-css mixin
系统 SHALL 提供 `meta.load-css($module, $with: ())` mixin，动态加载指定模块的 CSS 并注入当前上下文。系统 SHALL 支持通过模块名字符串或模块命名空间引用加载模块，并在加载时正确传递 `$with` 配置 map。

#### Scenario: 加载模块 CSS
- **WHEN** 调用 `@include meta.load-css("rounded-corners")`
- **THEN** 系统 加载 `rounded-corners` 模块并将其 CSS 输出注入当前位置

#### Scenario: 带 with 配置加载
- **WHEN** 调用 `@include meta.load-css("theme", $with: (color: blue))`
- **THEN** 系统 使用 `color: blue` 配置加载 `theme` 模块并注入 CSS

#### Scenario: 模块限定调用
- **WHEN** 调用 `@include meta.load-css(...)`
- **THEN** 系统 正确识别 `meta.load-css` 为 mixin 调用而非函数调用

#### Scenario: 不存在的模块
- **WHEN** 调用 `@include meta.load-css("nonexistent")`
- **THEN** 系统 报 "Can't find stylesheet to import: nonexistent" 错误

### Requirement: meta.get-mixin 函数
系统 SHALL 提供 `meta.get-mixin($name, $module: null)` 函数，返回 mixin 引用值。

#### Scenario: 获取全局 mixin
- **WHEN** 调用 `meta.get-mixin("my-mixin")`
- **THEN** 系统 返回 `MixinRef` 值，包含 mixin 名称和 null 模块

#### Scenario: 获取模块 mixin
- **WHEN** 调用 `meta.get-mixin("my-mixin", $module: $my-module)`
- **THEN** 系统 返回 `MixinRef` 值，包含 mixin 名称和模块来源

### Requirement: meta.apply mixin
系统 SHALL 提供 `meta.apply($mixin, $args...)` mixin，动态调用 mixin 引用。系统 SHALL 验证 `$mixin` 参数为 MixinRef 类型，对非 mixin 引用报类型错误。

#### Scenario: 调用 mixin 引用
- **WHEN** 调用 `@include meta.apply(meta.get-mixin("my-mixin"))`
- **THEN** 系统 执行 `my-mixin` mixin

#### Scenario: 带参数调用
- **WHEN** 调用 `@include meta.apply(meta.get-mixin("greet"), "Hello")`
- **THEN** 系统 使用参数 `"Hello"` 执行 `greet` mixin

#### Scenario: 带 content 调用
- **WHEN** 调用 `@include meta.apply($mixin) { color: red; }`
- **THEN** 系统 将 `@content` 传递给被调用的 mixin

#### Scenario: 非 mixin 引用参数
- **WHEN** 调用 `@include meta.apply(2px)`
- **THEN** 系统 报 "$mixin: 2px is not a mixin reference." 错误

#### Scenario: 缺少参数
- **WHEN** 调用 `@include meta.apply()` 无参数
- **THEN** 系统 报 "Missing argument $mixin." 错误

### Requirement: meta.module-functions 反射
系统 SHALL 提供 `meta.module-functions($module)` 函数，返回模块中所有公开函数的 map。

#### Scenario: 获取模块函数列表
- **WHEN** 调用 `meta.module-functions($m)` 其中 `$m` 是通过 `@use` 获得的模块引用
- **THEN** 系统 返回 `(function-name: function-ref, ...)` 形式的 map

### Requirement: meta.module-mixins 反射
系统 SHALL 提供 `meta.module-mixins($module)` 函数，返回模块中所有公开 mixin 的 map。

#### Scenario: 获取模块 mixin 列表
- **WHEN** 调用 `meta.module-mixins($m)`
- **THEN** 系统 返回 `(mixin-name: mixin-ref, ...)` 形式的 map

### Requirement: meta.module-variables 反射
系统 SHALL 提供 `meta.module-variables($module)` 函数，返回模块中所有公开变量的 map。

#### Scenario: 获取模块变量列表
- **WHEN** 调用 `meta.module-variables($m)`
- **THEN** 系统 返回 `(variable-name: value, ...)` 形式的 map

