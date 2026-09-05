## ADDED Requirements

### Requirement: @font-face 解析
MUST 解析 `@font-face { ... }` 规则及其Descriptor（font-family、src、font-weight、font-style 等）。

#### Scenario: 基本 font-face
- **WHEN** 输入包含 `@font-face { font-family: 'MyFont'; src: url('font.woff'); }`
- **THEN** 输出保留 @font-face 块

### Requirement: @font-face 内嵌规则
MUST 支持 @font-face 位于父选择器内部时正常输出。

#### Scenario: 嵌套 font-face
- **WHEN** @font-face 定义在外层规则内部
- **THEN** 正确序列化，可选择提升到根级别或保持嵌套
