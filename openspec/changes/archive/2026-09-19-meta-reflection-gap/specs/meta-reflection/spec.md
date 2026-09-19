# meta-reflection Delta Specification

## MODIFIED Requirements

### Requirement: get-function 反射
系统 SHALL 支持 `get-function($name, $module: null)` 内建函数，返回指定名称的函数引用。
查找路径 SHALL 覆盖：local scope → 所有 namespace 模块 → 内建函数表。

#### Scenario: 获取本地已定义函数
- **WHEN** 调用 `get-function("my-fn")` 且 `my-fn` 在本地 scope 已定义
- **THEN** 返回可调用的函数值，可用于 `call()`

#### Scenario: 获取 namespace 模块的函数
- **WHEN** 通过 `@use 'module_a'` 导入了模块，且模块定义了函数 `c`
- **THEN** `get-function("c")` 能正确找到并返回该函数的 FunctionRef
- **AND** `get-function("d")` 同理能找到 namespace 中的函数

#### Scenario: 函数不存在时报错
- **WHEN** 调用 `get-function("nonexistent")`
- **THEN** 不返回 null，而是报错 "Undefined function: nonexistent."

#### Scenario: 2-arg 模块限定查找
- **WHEN** 调用 `get-function("foo", "a")` 且 namespace "a" 存在
- **THEN** 在 namespace "a" 的函数表中查找 foo

#### Scenario: 模块不存在时报错
- **WHEN** 调用 `get-function("foo", "a")` 且 namespace "a" 不存在
- **THEN** 报错 "There is no module with namespace \"a\"."

#### Scenario: 非字符串 $name 报错
- **WHEN** 调用 `get-function(2px)` 传入非字符串
- **THEN** 报错 "$name: 2px is not a string."

#### Scenario: 非字符串 $module 报错
- **WHEN** 调用 `get-function("foo", 1)` 传入非字符串 module
- **THEN** 报错 "$module: 1 is not a string."

#### Scenario: 参数过多报错
- **WHEN** 调用 `get-function("a", "b", "c", "d")`
- **THEN** 报错提示只允许 2 个参数但传入了 4 个

#### Scenario: 无参数报错
- **WHEN** 调用 `get-function()`
- **THEN** 报错 "Missing argument $name."

#### Scenario: CSS 模式
- **WHEN** 调用 `get-function("rgb", $css: true)`
- **THEN** 返回 CSS 原生函数引用

### Requirement: load-css 动态加载
系统 SHALL 支持 `meta.load-css($module, $with: null)` 内建函数，在运行时动态加载模块 CSS。
同时 SHALL 支持裸名 `load-css(...)` 调用（当通过 `@use 'sass:meta'` 导入后）。

#### Scenario: 通过 meta.load-css 调用
- **WHEN** 调用 `@include meta.load-css("theme")`
- **THEN** 将 `theme` 模块的 CSS 输出到当前位置

#### Scenario: 通过裸名 load-css 调用
- **WHEN** 使用 `@use 'sass:meta'` 后调用 `@include load-css("theme")`
- **THEN** 将 `theme` 模块的 CSS 输出到当前位置

#### Scenario: 带配置加载
- **WHEN** 调用 `load-css("theme", $with: (primary: red))`
- **THEN** 使用指定配置加载主题模块
