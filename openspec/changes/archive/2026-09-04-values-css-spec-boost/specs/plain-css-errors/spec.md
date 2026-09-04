## MODIFIED Requirements

### Requirement: plain CSS 变量限制
系统 SHALL 在 plain CSS 模式中检测 Sass 变量使用并报错，包括 `calc()` 内部的变量引用。

#### Scenario: 变量引用
- **WHEN** 在 `.css` 文件的声明值中引用 `$variable`
- **THEN** 系统 报 "Sass variables aren't allowed in plain CSS." 错误

#### Scenario: calc 内部变量引用
- **WHEN** 在 `.css` 文件中使用 `calc($var)`
- **THEN** 系统 报 "Sass variables aren't allowed in plain CSS." 错误

#### Scenario: 父选择器后缀
- **WHEN** 在 `.css` 文件中使用 `&-suffix` 父选择器后缀
- **THEN** 系统 报 "Parent selectors can't have suffixes in plain CSS." 错误

#### Scenario: 占位选择器
- **WHEN** 在 `.css` 文件中使用 `%placeholder` 选择器
- **THEN** 系统 报 "Placeholder selectors aren't allowed in plain CSS." 错误

#### Scenario: 顶级前导组合器
- **WHEN** 在 `.css` 文件中使用 `> .child` 或 `+ .sibling` 顶级前导组合器
- **THEN** 系统 报 "Top-level leading combinators aren't allowed in plain CSS." 错误

### Requirement: plain CSS interpolation 限制
系统 SHALL 在 plain CSS 模式中检测不允许位置的插值并报错，包括 `calc()` 内部的插值。

#### Scenario: 选择器中的插值
- **WHEN** 在 `.css` 文件中的选择器位置使用 `#{...}` 插值
- **THEN** 系统 报 "Interpolation isn't allowed in plain CSS." 错误

#### Scenario: 属性名中的插值
- **WHEN** 在 `.css` 文件中的属性名位置使用 `#{...}` 插值
- **THEN** 系统 报 "Interpolation isn't allowed in plain CSS." 错误

#### Scenario: calc 内部插值
- **WHEN** 在 `.css` 文件中使用 `calc(#{1px})`
- **THEN** 系统 报 "Interpolation isn't allowed in plain CSS." 错误

## ADDED Requirements

### Requirement: plain CSS 命名空间函数限制
系统 SHALL 在 plain CSS 模式中检测命名空间函数调用（如 `c.d()`）并报错。

#### Scenario: calc 内部命名空间函数
- **WHEN** 在 `.css` 文件中使用 `calc(c.d())`
- **THEN** 系统 报 "Module namespaces aren't allowed in plain CSS." 错误

### Requirement: plain CSS 父选择器限制
系统 SHALL 在 plain CSS 模式中检测声明值中的父选择器 `&` 并报错。

#### Scenario: 声明值中的父选择器
- **WHEN** 在 `.css` 文件的声明值中使用 `&`
- **THEN** 系统 报 "The parent selector isn't allowed in plain CSS." 错误

### Requirement: plain CSS 内建函数限制
系统 SHALL 在 plain CSS 模式中检测 Sass 内建函数调用（如 `index()`）并报错。

#### Scenario: 声明值中的内建函数
- **WHEN** 在 `.css` 文件的声明值中使用 `index(1 2 3, 1)`
- **THEN** 系统报错

### Requirement: plain CSS map/list 限制
系统 SHALL 在 plain CSS 模式中检测 map 和 list 字面量并报错。

#### Scenario: map 字面量
- **WHEN** 在 `.css` 文件中使用 `(y: z)`
- **THEN** 系统报错

#### Scenario: 空 list 字面量
- **WHEN** 在 `.css` 文件中使用 `()`
- **THEN** 系统报错
