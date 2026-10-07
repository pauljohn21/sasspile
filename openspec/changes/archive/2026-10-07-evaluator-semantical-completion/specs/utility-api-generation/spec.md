# Spec Delta

## Purpose

从嵌套 SCSS map 结构完整生成 Bootstrap Utility API 类（spacing、display、flex、sizing），包括响应式断点变体。该能力是 `bootstrap-dist-alignment` 覆盖率从 22% 提升到 60%+ 的核心路径。

## ADDED Requirements

### Requirement: 系统 SHALL 正确迭代三层嵌套 `$utilities` map
系统 SHALL 迭代 `$utilities` map 的顶层 keys（如 `spacing`、`display`）、第二层 keys（如 `margin`、`padding`）和第三层 values（CSS property + values map），为每个叶节点生成对应的 CSS 类。

#### Scenario: 三层 @each 迭代
- **WHEN** `$utilities` 包含 `{ spacing: { margin: { responsive: true, property: margin, values: { 0: 0, 1: 0.25rem, 5: 3rem } } } }`
- **THEN** 编译生成 `.m-0 { margin: 0 }`、`.m-1 { margin: 0.25rem }`、`.m-5 { margin: 3rem }`
- **WHEN** values 包含 `null` 键（代表无前缀）
- **THEN** 生成 `.g-0 { gap: 0 }` 形式（无前缀变体）

#### Scenario: responsive 标志触发媒体查询包裹
- **WHEN** spacing value 中 `responsive: true` 且包含断点 map
- **THEN** 每个 spacing 类 SHALL 被 `@media (min-width: ...)` 包裹，生成 `.m-sm-0 { margin: 0 }` 等断点变体

### Requirement: 系统 SHALL 生成 spacing utility 类
系统 SHALL 根据 spacing 配置 `{ property: margin/padding, class: m/p, values: {...} }` 生成简写类。

#### Scenario: margin spacing
- **WHEN** 编译 `map-get($utilities, "spacing")` 中的 margin 配置
- **THEN** 生成 `.mt-1 { margin-top: 0.25rem }`、`.mx-2 { margin-left: 0.5rem; margin-right: 0.5rem }`、`.m-3 { margin: 0.75rem }`

#### Scenario: padding spacing
- **WHEN** 编译 padding 配置
- **THEN** 生成 `.p-1 { padding: 0.25rem }`、`.py-2 { padding-top: 0.5rem; padding-bottom: 0.5rem }`

### Requirement: 系统 SHALL 生成 display utility 类
系统 SHALL 生成 `.d-none`、`.d-block`、`.d-flex` 等进行响应式变体。

#### Scenario: display basic
- **WHEN** 编译 `$utilities` 中 display 配置
- **THEN** 生成 `.d-none { display: none }`、`.d-block { display: block }`、`.d-flex { display: flex }` 等

#### Scenario: display responsive
- **WHEN** display 配置 `responsive: true`
- **THEN** 生成 `.d-sm-none`、`.d-md-block`、`.d-lg-flex` 等所有断点前缀变体

### Requirement: 系统 SHALL 处理 map 值的 CSS property 简写展开
系统 SHALL 将 `margin-top` 简写展开为完整的 CSS 属性名（`margin-top: value`）。

#### Scenario: 单属性 property
- **WHEN** `property` 字段为 `"margin"`
- **THEN** 生成 `margin: value`

#### Scenario: 多属性 property（shorthand）
- **WHEN** `property` 字段为 `["margin-top", "margin-bottom"]`
- **THEN** 生成两条声明：`margin-top: value; margin-bottom: value`

### Requirement: 系统 SHALL 处理 `mx-auto` 特殊值
系统 SHALL 正确处理 `auto` 值生成 `.mx-auto { margin-left: auto; margin-right: auto }`。

#### Scenario: mx-auto
- **WHEN** mx values 包含 `"auto": auto`
- **THEN** 生成 `.mx-auto { margin-left: auto; margin-right: auto }`

### Requirement: 系统 SHALL 处理 `!important` 标记
系统 SHALL 对标记为 `important: true` 的属性在声明末尾添加 `!important`。

#### Scenario: important true
- **WHEN** utility 配置 `important: true`
- **THEN** 每个声明 SHALL 以 `!important` 结尾，如 `.m-0 { margin: 0 !important }`
