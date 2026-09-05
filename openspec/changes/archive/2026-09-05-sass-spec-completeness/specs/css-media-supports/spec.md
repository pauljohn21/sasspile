## ADDED Requirements

### Requirement: @supports 嵌套格式
MUST 正确输出 `@supports` 规则，保持括号内声明格式和逻辑操作符（and/or/not）。

#### Scenario: 基本 supports
- **WHEN** 输入包含 `@supports (display: grid) { .a { display: grid; } }`
- **THEN** 输出保留 @supports 条件语法

#### Scenario: supports 逻辑操作
- **WHEN** 输入包含 `@supports not (display: grid) { ... }`
- **THEN** 保留 not/and/or 逻辑

### Requirement: @media 提升与合并
MUST 将 @media 从嵌套选择器中正确提升（bubbling），并支持查询合并。

#### Scenario: media 提升
- **WHEN** `.a { @media (min-width: 768px) { color: red; } }`
- **THEN** @media 提升到外层，内部仅保留 .a 规则

#### Scenario: media 嵌套
- **WHEN** @media 嵌套在另一个 @media 内部
- **THEN** 正确合并为单个 @media 或保持嵌套（按规范）
