## ADDED Requirements

### Requirement: selector.replace() 实现
MUST 实现 `selector.replace($selector, $original, $replacement)` 选择器子串替换。

#### Scenario: 基本替换
- **WHEN** 调用 `selector.replace('.a.b', '.b', '.c')`
- **THEN** 返回 `.a.c`

#### Scenario: 未找到替换源
- **WHEN** 调用 `selector.replace('.a.b', '.x', '.c')`
- **THEN** 返回原始选择器（或报错，按规范）

### Requirement: selector.nest() 列表边界
MUST 正确处理 `selector.nest($selectors...)` 传入列表作为参数的情况。

#### Scenario: 列表参数展开
- **WHEN** 调用 `selector.nest($sel1, $sel2, ...)`
- **THEN** 正确处理多参数或列表展开
