## ADDED Requirements

### Requirement: @container 查询解析
MUST 解析 `@container name (condition) { ... }` 容器查询规则。

#### Scenario: 基本 container
- **WHEN** 输入包含 `@container (min-width: 700px) { .item { color: red; } }`
- **THEN** 输出保留 container 查询结构

#### Scenario: 命名 container
- **WHEN** 输入包含 `@container sidebar (min-width: 700px) { ... }`
- **THEN** 保留容器名
