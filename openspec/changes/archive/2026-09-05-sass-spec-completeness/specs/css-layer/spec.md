## ADDED Requirements

### Requirement: @layer 声明语法
MUST 解析 `@layer` 的无块声明语法（`@layer name, name2;`）用于声明层顺序。

#### Scenario: layer 顺序声明
- **WHEN** 输入包含 `@layer base, utilities;`
- **THEN** 输出保留 layer 声明

### Requirement: @layer 块语法
MUST 解析 `@layer name { ... }` 块语法，将内部规则归入命名层。

#### Scenario: layer 块规则
- **WHEN** 输入包含 `@layer utilities { .padding-sm { padding: 0.5rem; } }`
- **THEN** 输出保留 layer 块结构

#### Scenario: 匿名 layer
- **WHEN** 输入包含 `@layer { .rule { color: red; } }`
- **THEN** 输出保留匿名 layer 块

#### Scenario: 嵌套 layer
- **WHEN** layer 定义在另一个 layer 内部
- **THEN** 正确处理 layer 嵌套
