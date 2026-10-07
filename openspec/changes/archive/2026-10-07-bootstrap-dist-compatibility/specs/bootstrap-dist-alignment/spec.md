# Spec Delta

## Purpose

对齐 rx-scss 编译产物与 Bootstrap 5.3.x 官方 dist CSS，确保全量 `bootstrap.scss` 编译输出逐行匹配 `dist/css/bootstrap.css`（280KB, 12048 行）。该能力作为所有 SCSS 特性补全的终极验收标准。

## ADDED Requirements

### Requirement: 全量 Bootstrap dist 逐行对齐
系统 SHALL 将 `bootstrap.scss` 编译产物与 `dist/css/bootstrap.css` 进行逐行 diff，所有 CSS 声明块（selector + properties）在参考文件中存在且在产物中存在。

#### Scenario: 编译产物覆盖所有参考 CSS 规则
- **WHEN** 编译 `bootstrap/bootstrap.scss`（expanded 模式）
- **THEN** 产物的 CSS 规则集合 SHALL 包含参考文件中 >= 99% 的规则行（允许空白/注释差异）

#### Scenario: CSS Custom Properties 完整生成
- **WHEN** 编译包含 `:root { --bs-btn-close-opacity: 0.5; }` 的 SCSS
- **THEN** 产物 SHALL 包含 `--bs-btn-close-opacity: 0.5` 且值正确

#### Scenario: Utility API 全量展开
- **WHEN** 编译 `$utilities` map 上的 `@each` 迭代
- **THEN** 产物 SHALL 包含所有 spacing utilities (`.mt-0` 到 `.mt-5`, `.mx-auto`, `.px-lg-2` 等)

#### Scenario: 响应式断点类生成
- **WHEN** 编译 `@include media-breakpoint-up(sm) { ... }` 循环内的工具类
- **THEN** 产物 SHALL 包含 `.d-sm-none`, `.d-md-block`, `.col-lg-6` 等断点前缀类

#### Scenario: Component CSS 完整生成
- **WHEN** 编译 `@import \"buttons\"` 导入链
- **THEN** 产物 SHALL 包含 `.btn { display: inline-block; font-weight: 400; ... }` 的完整声明

#### Scenario: Form 组件伪元素选择器
- **WHEN** 编译 `.form-floating > textarea ~ label::after`
- **THEN** **THEN** 产物 SHALL 保留 `::after` 伪元素和 `~` 选择器

#### Scenario: SVG Data-URI 字符串
- **WHEN** 编译 `background-image: url("data:image/svg+xml,...")`
- **THEN** 产物 SHALL 保留完整的 SVG data-URI 字符串（引号和转义不变）

### Requirement: 自动化对照测试基础设施
系统 SHALL 提供 `bootstrap_dist_check()` 函数来执行自动化 diff，输出结构化差异报告。

#### Scenario: 差异报告生成
- **WHEN** 编译产物与参考文件存在差异
- **THEN** 系统输出 SHALL 包含：missing_rules 计数、missing_properties 计数、前 50 条差异详情

#### Scenario: CI 门控通过条件
- **WHEN** 对照覆盖率 >= 99%
- **THEN** 测试 SHALL pass
- **WHEN** 对照覆盖率 < 99%
- **THEN** 测试 SHALL fail 并输出差异

### Requirement: rgba() 支持 CSS Variable 第一个参数
系统 SHALL 接受 `rgba($color, $alpha)` 第一个参数为 CSS Variable（`var(--bs-white)`），返回 `rgba(var(--bs-white-rgb), $alpha)` 格式。

#### Scenario: rgba with CSS var
- **WHEN** 调用 `rgba(#fff, 0.5)`
- **THEN** 返回 `rgba(255, 255, 255, 0.5)` (comma-separated RGB)
- **WHEN** 调用 `rgba(var(--bs-primary), 0.5)`
- **THEN** 返回 `rgba(var(--bs-primary-rgb), 0.5)` (引用 RGB 变量)

### Requirement: 颜色 channel 函数
系统 SHALL 支持 `red($color)`, `green($color)`, `blue($color)`, `alpha($color)` 通道提取函数。

#### Scenario: red channel
- **WHEN** 调用 `red(#ff0000)`
- **THEN** 返回 `255`
- **WHEN** 调用 `green(#00ff00)`
- **THEN** 返回 `0`

#### Scenario: alpha channel
- **WHEN** 调用 `alpha(rgba(0,0,0,0.5))`
- **THEN** 返回 `0.5`

### Requirement: shade-color / tint-color 混合函数
系统 SHALL 实现 `shade-color($color, $weight)` 和 `tint-color($color, $weight)` 函数，按权重与 black/white 混合。

#### Scenario: shade-color
- **WHEN** 调用 `shade-color(#0d6efd, 15%)`
- **THEN** 返回颜色加深 15% 后的值

#### Scenario: tint-color
- **WHEN** 调用 `tint-color(#0d6efd, 15%)`
- **THEN** 返回颜色加白 15% 后的值

#### Scenario: 与 mix() 等价
- **WHEN** 调用 `shade-color(#fff, 20%)`
- **THEN** 等价于 `mix(#000, #fff, 20%)`
- **WHEN** 调用 `tint-color(#000, 20%)`
- **THEN** 等价于 `mix(#fff, #000, 20%)`

### Requirement: to-rgb / to-rgb-list 转换
系统 SHALL 支持 `to-rgb($color)` 返回 `R, G, B` comma-separated 格式字符串，用于 CSS Custom Properties。

#### Scenario: to-rgb hex color
- **WHEN** 调用 `to-rgb(#0d6efd)`
- **THEN** 返回 `"13, 110, 253"` (字符串)

#### Scenario: to-rgb named color
- **WHEN** 调用 `to-rgb(red)`
- **THEN** 返回 `"255, 0, 0"`

#### Scenario: 用于 RGB variable
- **WHEN** 声明 `--bs-primary-rgb: #{to-rgb(#0d6efd)};`
- **THEN** 产物 SHALL 包含 `--bs-primary-rgb: 13, 110, 253;`

### Requirement: 字符串替换函数
系统 SHALL 实现 `str-replace($string, $search, $replacement)` 全局替换函数。

#### Scenario: 基本替换
- **WHEN** 调用 `str-replace("foobar", "foo", "baz")`
- **THEN** 返回 `"bazbar"`
- **WHEN** 调用 `str-replace("a,b,c", ",", "%2C")`
- **THEN** 返回 `"a%2Cb%2Cc"`

#### Scenario: SVG 内容替换
- **WHEN** 调用 `str-replace($svg, "%23", "#")`
- **THEN** 返回替换后的字符串

### Requirement: 双变量 @each map 迭代
系统 SHALL 支持 `@each $key, $value in $map` 双变量形式。

#### Scenario: 双变量迭代
- **WHEN** 编译 `@each $key, $value in $utilities`
- **THEN** `$key` 和 `$value` 分别在各自迭代中绑定正确

#### Scenario: map 键值对访问
- **WHEN** 编译 `@each $color, $value in $theme-colors`
- **THEN** `$color` 绑定键名（如 `primary`），`$value` 绑定色值（如 `#0d6efd`）

### Requirement: @content 占位符替换
系统 SHALL 在 `@include mixin-name { content }` 时将 mixin body 内的 `@content` 替换为调用方传入的内容块。

#### Scenario: 基本 content 替换
- **WHEN** `@mixin test { div { @content; } }` 后跟 `@include test { color: red; }`
- **THEN** 产物 SHALL 包含 `div { color: red; }`（`@content` 被替换）

#### Scenario: responsive 断点 mixin
- **WHEN** `@include media-breakpoint-up(md) { .custom { display: block; } }`
- **THEN** 产物 SHALL 包含 `@media (min-width: 768px) { .custom { display: block; } }`

### Requirement: map 函数补全
系统 SHALL 实现完整的 map 函数集包括 `map-keys()`, `map-values()`, `map-get()`, `map-has-key()`, `map-merge()`。

#### Scenario: map-keys
- **WHEN** 调用 `map-keys(("a": 1, "b": 2))`
- **THEN** 返回列表 `("a", "b")`

#### Scenario: map-values
- **WHEN** 调用 `map-values(("a": 1, "b": 2))`
- **THEN** 返回列表 `(1, 2)`

#### Scenario: map-has-key nested
- **WHEN** 调用 `map-has-key($utilities, "margin")`
- **THEN** 返回 `true` (嵌套 map 检查)

### Requirement: list 函数补全
系统 SHALL 实现 `list-separator()`, `list-length()`, `list-nth()`, `list-join()`, `list-append()`, `list-zip()`。

#### Scenario: list-separator
- **WHEN** 调用 `list-separator(1px solid red)`
- **THEN** 返回 `"space"`

#### Scenario: list-zip
- **WHEN** 调用 `list-zip(a b c, 1 2 3)`
- **THEN** 返回 `((a 1), (b 2), (c 3))`
