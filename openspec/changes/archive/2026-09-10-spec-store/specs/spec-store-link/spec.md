## ADDED Requirements

### Requirement: CodeGraph 调用链桥接
系统 SHALL 给定函数名，输出 CodeGraph 调用链 + 关联 spec cases 的桥接视图。

#### Scenario: 正向查找（代码 → spec）
- **WHEN** 运行 `spec_store link --function math.sin`
- **THEN** 系统输出：(1) 该函数的 spec case 列表及状态，(2) CodeGraph callers 调用链

#### Scenario: 反向查找（spec → 代码）
- **WHEN** 运行 `spec_store link --case core_functions/math/sin/deg-to-rad`
- **THEN** 系统输出该 case 涉及的 CodeGraph 符号和调用路径

#### Scenario: 修复影响预估
- **WHEN** 运行 `spec_store link --function math.sin --impact`
- **THEN** 系统输出修复该函数会影响的所有 case 列表和预估通过数
