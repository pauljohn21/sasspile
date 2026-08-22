## MODIFIED Requirements

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
