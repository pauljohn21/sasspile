# meta-reflection Specification

## Purpose
TBD - created by archiving change feature-completeness-boost. Update Purpose after archive.
## Requirements
### Requirement: get-function 反射
系统 SHALL 支持 `get-function($name, $css: false)` 内建函数，返回指定名称的函数引用。

#### Scenario: 获取已定义函数
- **WHEN** 调用 `get-function("my-fn")` 且 `my-fn` 已定义
- **THEN** 返回可调用的函数值，可用于 `call()`

#### Scenario: 函数不存在
- **WHEN** 调用 `get-function("nonexistent")`
- **THEN** 返回 `null`（不报错）

#### Scenario: CSS 模式
- **WHEN** 调用 `get-function("rgb", $css: true)`
- **THEN** 返回 CSS 原生函数引用

### Requirement: get-mixin 反射
系统 SHALL 支持 `get-mixin($name)` 内建函数，返回指定名称的 mixin 引用。

#### Scenario: 获取已定义 mixin
- **WHEN** 调用 `get-mixin("my-mixin")` 且 `my-mixin` 已定义
- **THEN** 返回可调用的 mixin 值

#### Scenario: mixin 不存在
- **WHEN** 调用 `get-mixin("nonexistent")`
- **THEN** 返回 `null`

### Requirement: module-variables 反射
系统 SHALL 支持 `module-variables($module)` 内建函数，返回指定模块的所有变量名映射。

#### Scenario: 获取模块变量列表
- **WHEN** 调用 `module-variables("utils")` 且 `utils` 已通过 @use 加载
- **THEN** 返回该模块所有 `@use` 导出变量的 Map

### Requirement: module-functions 反射
系统 SHALL 支持 `module-functions($module)` 内建函数，返回指定模块的所有函数名。

#### Scenario: 获取模块函数列表
- **WHEN** 调用 `module-functions("sass:math")`
- **THEN** 返回 math 模块所有可用函数名列表

### Requirement: module-mixins 反射
系统 SHALL 支持 `module-mixins($module)` 内建函数，返回指定模块的所有 mixin 名。

#### Scenario: 获取模块 mixin 列表
- **WHEN** 调用 `module-mixins("utils")`
- **THEN** 返回该模块所有可用 mixin 名列表

### Requirement: load-css 动态加载
系统 SHALL 支持 `load-css($module, $with: null)` 内建函数，在运行时动态加载模块 CSS。

#### Scenario: 动态加载模块
- **WHEN** 调用 `load-css("theme")`
- **THEN** 将 `theme` 模块的 CSS 输出到当前位置

#### Scenario: 带配置加载
- **WHEN** 调用 `load-css("theme", $with: (primary: red))`
- **THEN** 使用指定配置加载主题模块

### Requirement: apply 函数
系统 SHALL 支持 `apply($function, $args...)` 内建函数，按函数引用动态调用。

#### Scenario: 按引用调用函数
- **WHEN** 调用 `apply($fn, 1, 2)` 其中 `$fn` 来自 `get-function()`
- **THEN** 等价于直接调用 `$fn(1, 2)`

### Requirement: keywords 函数
系统 SHALL 支持 `keywords($args)` 内建函数，返回 mixin 调用时的命名参数 Map。

#### Scenario: 获取命名参数
- **WHEN** mixin 被调用为 `include foo($a: 1, $b: 2)` 时调用 `keywords($args)`
- **THEN** 返回 `($a: 1, $b: 2)` 映射

### Requirement: feature-exists 检测
系统 SHALL 支持 `feature-exists($feature)` 内建函数，检测指定 Sass 特性是否可用。

#### Scenario: 已知特性
- **WHEN** 调用 `feature-exists("global-variable-shadowing")`
- **THEN** 返回 `true`

#### Scenario: 未知特性
- **WHEN** 调用 `feature-exists("unknown-feature")`
- **THEN** 返回 `false`

### Requirement: content-exists 检测
系统 SHALL 支持 `content-exists()` 内建函数，检测当前 mixin 是否通过 `@content` 传递了内容块。

#### Scenario: 有 @content
- **WHEN** mixin 被 `@include` 且带有 `{...}` 块
- **THEN** `content-exists()` 返回 `true`

#### Scenario: 无 @content
- **WHEN** mixin 被 `@include` 无块
- **THEN** `content-exists()` 返回 `false`

