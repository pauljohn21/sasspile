## MODIFIED Requirements

### Requirement: color.scale() 算法
MUST 使用 sass-spec 规范的 scale 算法——基于当前值与极值之间的距离按比例调整，而非线性插值。

#### Scenario: scale 计算符合规范
- **WHEN** 调用符合 sass-spec 的 scale 用例
- **THEN** 返回精确匹配的期望输出值

### Requirement: color.change() 通道 clamp
MUST 在设置新值后将通道 clamp 到有效范围（0-255 for RGB, 0-1 for alpha）。

#### Scenario: 超出范围 clamp
- **WHEN** change 操作导致值超出有效范围
- **THEN** 自动 clamp 到边界值
