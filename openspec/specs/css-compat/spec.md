# css-compat Specification

## Purpose
TBD - created by archiving change feature-completeness-boost. Update Purpose after archive.
## Requirements
### Requirement: @supports 查询语法
系统 SHALL 支持 `@supports` 条件规则——包括属性值查询、逻辑运算符（and/or/not）。

#### Scenario: 属性值查询
- **WHEN** `@supports (display: grid) { ... }`
- **THEN** 规则在该特性可用时应用

#### Scenario: not 查询
- **WHEN** `@supports not (display: grid) { ... }`
- **THEN** 规则在该特性不可用时应用

#### Scenario: and/or 组合
- **WHEN** `@supports (display: flex) and (transform: rotate(45deg)) { ... }`
- **THEN** 两个特性均可用时应用

### Requirement: @media 嵌套和逻辑
系统 SHALL 支持 `@media` 查询嵌套在规则内，以及复杂 media type/feature 组合。

#### Scenario: 规则内嵌套
- **WHEN** `@media (min-width: 768px)` 嵌套在 `.class { ... }` 内
- **THEN** 媒体查询提升到外层，包裹原规则

#### Scenario: 范围语法
- **WHEN** `@media (width >= 768px) { ... }`
- **THEN** 解析范围表达式

### Requirement: 自定义属性（--*）
系统 SHALL 支持 CSS 自定义属性的定义和使用（`var()` 函数）。

#### Scenario: 自定义属性定义
- **WHEN** `--primary: blue`
- **THEN** 输出 `--primary: blue`

#### Scenario: var() 默认值
- **WHEN** `color: var(--primary, red)`
- **THEN** 输出 `color: var(--primary, red)`（保留原样）

### Requirement: CSS 函数嵌套
系统 SHALL 正确处理 `calc()`、`min()`、`max()`、`clamp()` 嵌套。

#### Scenario: calc 嵌套
- **WHEN** `calc(100% - 20px)`
- **THEN** 保留为 CSS 原生 calc 表达式

#### Scenario: min/max/clamp
- **WHEN** `min(10px, 5%)`、`clamp(10px, 5%, 50px)`
- **THEN** 原样保留输出

### Requirement: @layer 规则
系统 SHALL 支持 CSS Cascade Layers `@layer` 规则。

#### Scenario: 声明层
- **WHEN** `@layer utilities { ... }`
- **THEN** 输出保持 `@layer` 结构

#### Scenario: 层排序
- **WHEN** `@layer base, components, utilities;`
- **THEN** 声明层顺序

### Requirement: @container 查询
系统 SHALL 支持 Container Queries `@container` 规则。

#### Scenario: 容器查询
- **WHEN** `@container (min-width: 400px) { ... }`
- **THEN** 输出保持 @container 结构

### Requirement: @scope 规则
系统 SHALL 支持 CSS Scoping `@scope` 规则。

#### Scenario: 作用域样式
- **WHEN** `@scope (.card) to (.footer) { ... }`
- **THEN** 输出保持 @scope 结构

