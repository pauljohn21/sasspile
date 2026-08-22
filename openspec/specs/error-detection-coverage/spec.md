## Purpose

定义 sasspile 编译器在表达式解析、selector 函数、map 类型检查、模块冲突和 plain CSS 模式中的错误检测能力。
## Requirements
### Requirement: 表达式语法错误检测
系统 SHALL 在表达式解析中检测无效语法并报错，而非静默跳过。

#### Scenario: not 后无有效表达式
- **WHEN** 解析 `not` 后不跟有效表达式（如 `not not`）
- **THEN** 系统 报语法错误

#### Scenario: and/or 后无有效表达式
- **WHEN** 解析 `a and` 或 `a or` 后不跟有效表达式
- **THEN** 系统 报语法错误

#### Scenario: 空括号
- **WHEN** 解析 `()`
- **THEN** 系统 报语法错误

#### Scenario: or 前无有效表达式
- **WHEN** 解析 `or b`（or 前无操作数）
- **THEN** 系统 报语法错误

### Requirement: selector 函数错误检测
系统 SHALL 在 selector 函数中检测无效输入并报错。

#### Scenario: append 无效选择器类型
- **WHEN** 调用 `selector-append(123, ".b")`
- **THEN** 系统 报类型错误

#### Scenario: append 无效组合器
- **WHEN** 调用 `selector-append("> .a", ".b")`
- **THEN** 系统 报选择器错误

### Requirement: map 类型检查
系统 SHALL 在 map 函数中验证参数类型，对非 map 输入报错。

#### Scenario: deep_merge 非 map 参数
- **WHEN** 调用 `map-deep-merge(1, (a: 1))`
- **THEN** 系统 报 "$map: 1 is not a map" 错误

#### Scenario: 重复键检测
- **WHEN** map 字面量中存在重复键
- **THEN** 系统 报 "Duplicate key" 错误

### Requirement: @use/@forward conflict 检测
系统 SHALL 在模块加载时检测同名成员 conflict 并报错。

#### Scenario: 变量 conflict
- **WHEN** 两个 `@forward` 导出同名变量
- **THEN** 系统 报 "conflict" 错误

#### Scenario: 函数 conflict
- **WHEN** 两个 `@forward` 导出同名函数
- **THEN** 系统 报 "conflict" 错误

#### Scenario: 同值 conflict
- **WHEN** 两个 `@forward` 导出同名且同值的成员
- **THEN** 系统 不报错（允许同值冲突）

### Requirement: plain CSS 限制
系统 SHALL 在 plain CSS 模式中检测不允许的操作并报错。系统 SHALL 在以下场景中检测并报错：sass() 函数调用、@use/@forward/@include/@function/@mixin at-rule、选择器和属性名中的插值 `#{...}`、声明值中的运算符（`+`/`-`/`*`/`/`）、Sass 变量引用 `$var`、父选择器后缀 `&-suffix`、占位选择器 `%placeholder`、顶级前导组合器（`> ` 或 `+ ` 开头）。错误信息格式为 "<X> isn't allowed in plain CSS"。

#### Scenario: sass() 在 plain CSS
- **WHEN** 在 plain CSS 模式中调用 `sass()`
- **THEN** 系统 报 "sass() conditions aren't allowed in plain CSS" 错误

#### Scenario: @use 在 plain CSS
- **WHEN** 在 `.css` 文件中使用 `@use "sass:math"`
- **THEN** 系统 报 "This at-rule isn't allowed in plain CSS." 错误

#### Scenario: @include 在 plain CSS
- **WHEN** 在 `.css` 文件中使用 `@include mixin-name`
- **THEN** 系统 报 "This at-rule isn't allowed in plain CSS." 错误

#### Scenario: 插值在 plain CSS
- **WHEN** 在 `.css` 文件中的选择器位置使用 `#{...}` 插值
- **THEN** 系统 报 "Interpolation isn't allowed in plain CSS." 错误

#### Scenario: 运算符在声明值中
- **WHEN** 在 `.css` 文件的声明值中使用 `+` 运算符
- **THEN** 系统 报 "Operators aren't allowed in plain CSS." 错误

#### Scenario: Sass 变量引用
- **WHEN** 在 `.css` 文件的声明值中引用 `$variable`
- **THEN** 系统 报 "Sass variables aren't allowed in plain CSS." 错误

#### Scenario: 父选择器后缀
- **WHEN** 在 `.css` 文件中使用 `&-suffix` 父选择器后缀
- **THEN** 系统 报 "Parent selectors can't have suffixes in plain CSS." 错误

#### Scenario: 占位选择器
- **WHEN** 在 `.css` 文件中使用 `%placeholder` 选择器
- **THEN** 系统 报 "Placeholder selectors aren't allowed in plain CSS." 错误

#### Scenario: 顶级前导组合器
- **WHEN** 在 `.css` 文件中使用 `> .child` 顶级前导组合器
- **THEN** 系统 报 "Top-level leading combinators aren't allowed in plain CSS." 错误

