## ADDED Requirements

### Requirement: CSS custom property --value 序列化
系统 SHALL 对 Element Plus 定义的 CSS 变量（如 `--el-color-primary`）序列化值与 EP dist 一致。

#### Scenario: Hex 颜色值
- **WHEN** `--el-color-primary: #409eff`
- **THEN** 输出 `#409eff`（不转换为 rgb 或其他格式）

#### Scenario: CSS var() 引用保持
- **WHEN** 值包含 `var(--el-color-primary)`
- **THEN** 保持 var() 引用不展开（除非有 fallback 展开逻辑）

#### Scenario: zero-value unit
- **WHEN** `margin: 0` vs `margin: 0px`
- **THEN** 零值不带单位（与 EP dist 一致）
