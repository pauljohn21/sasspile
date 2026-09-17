# meta-advanced Specification

## ADDED Requirements

### Requirement: meta.module-variables

 SHALL 返回当前模块的所有变量名 list。

#### Scenario: at least one variable
- **WHEN** 输入 `@use "sass:meta"; $x: 1; a {b: meta.module-variables()}`
- **THEN** 输出包含变量名 `x`

### Requirement: meta.module-functions

 SHALL 返回当前模块的所有函数名 list。

#### Scenario: returns function names
- **WHEN** 输入包含 `meta.module-functions()`
- **THEN** 输出非空函数名 list

### Requirement: meta.keywords

 SHALL 接受带 rest args 的函数返回关键词 list。

#### Scenario: keywords
- **WHEN** 输入包含 `meta.keywords($args...)`
- **THEN** 返回关键字 map

### Requirement: meta.calc-args / meta.calc-name

 SHALL 解析 calc/表达 的函数名或参数。

### Requirement: meta.get-function

 SHALL 按名字返回 function 引用，以支持高阶函数 `(meta.get-function(name))(args)`。

#### Scenario: call via get-function
- **WHEN** 输入含 `call(get-function("rgb"), 255, 0, 0)`
- **THEN** 输出 `#ff0000`
