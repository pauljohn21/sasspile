# Spec Delta

## Purpose

为现有 Directive Operations 添加响应式 media-breakpoint mixin 展开语义，使得 `@include media-breakpoint-up(md) { .class { ... } }` 正确生成 `@media (min-width: 768px) { .class { ... } }`。

## ADDED Requirements

### Requirement: @include 展开响应式 breakpoint mixin 时生成 @media 包裹
系统 SHALL 在 `@include media-breakpoint-up($breakpoint)` 调用时，将 mixin body 内的 CSS 语句包裹在对应的 `@media (min-width: ...)` 查询中。

#### Scenario: media-breakpoint-up
- **WHEN** `@include media-breakpoint-up(md) { .custom { display: block; } }`
- **THEN** 产物 SHALL 包含 `@media (min-width: 768px) { .custom { display: block; } }`

#### Scenario: media-breakpoint-down
- **WHEN** `@include media-breakpoint-down(md) { .custom { display: none; } }`
- **THEN** 产物 SHALL 包含 `@media (max-width: 767.98px) { .custom { display: none; } }`

#### Scenario: media-breakpoint-between
- **WHEN** `@include media-breakpoint-between(sm, lg) { .custom { padding: 1rem; } }`
- **THEN** 产物 SHALL 包含 `@media (min-width: 576px) and (max-width: 991.98px) { .custom { padding: 1rem; } }`

#### Scenario: media-breakpoint-only
- **WHEN** `@include media-breakpoint-only(md) { .custom { font-size: 14px; } }`
- **THEN** 产物 SHALL 包含 `@media (min-width: 768px) and (max-width: 991.98px) { .custom { font-size: 14px; } }`

### Requirement: @media 包裹内的嵌套规则保持完整
系统 SHALL 保持 @media 块内的嵌套规则结构和顺序不变。

#### Scenario: 多规则 @media
- **WHEN** `@include media-breakpoint-up(md) { .a { color: red } .b { color: blue } }`
- **THEN** 两条规则 SHALL 都在同一 @media 块内，且顺序保持不变

#### Scenario: 嵌套 @media（mobile first）
- **WHEN** 同一选择器在多个断点中被定义
- **THEN** 每个断点的规则 SHALL 独立生成，不合并
