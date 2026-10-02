## ADDED Requirements

### Requirement: 伪元素与父选择器 compound 拼接
系统 SHALL 在 `@at-root` 上下文中正确拼接 `#{$parent}::before` / `#{$parent}::after`，保持为单一 compound 选择器。

#### Scenario: 双冒号伪元素
- **WHEN** 规则使用 `::before` 或 `::after` 作为 compound 后缀
- **THEN** 输出 `.parent::before` 而非 `.parent, ::before`

#### Scenario: 单冒号伪元素（CSS2 向后兼容）
- **WHEN** 规则使用 `:before` 或 `:after` 语法
- **THEN** 规范化输出 `: before` → `:before`（无前导空格）

#### Scenario: Mixin 生成的伪元素列表
- **WHEN** @at-root 生成 `sel::before, sel::after { ... }` 选择器列表
- **THEN** 每个列表项正确保持 compound 关系，逗号仅分隔不同选择器而非拆分 compound

### Requirement: 非伪元素选择器不触发 compound 逻辑
系统 SHALL 仅在 child **以 parent 为前缀**（parent + separator）时触发 compound 拼接。独立选择器保持 descendant 组合。

#### Scenario: 纯类选择器 descendant
- **WHEN** parent = `.a`, child = `.b`（无共享前缀）
- **THEN** 输出 `.a .b { ... }`（descendant combinator，保留空格）

#### Scenario: Attribute 选择器 compound
- **WHEN** parent = `.a`, child = `.a[href]`（attribute extension）
- **THEN** 输出 `.a[href] { ... }`（无 descendant 空格）
