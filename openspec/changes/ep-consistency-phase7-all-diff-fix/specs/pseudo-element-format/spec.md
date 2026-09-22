## ADDED Requirements

### Requirement: Pseudo-element whitespace normalization
系统 SHALL 在规范化模式下输出 `:before` `/ `:after` `/ `:first-line` `/ `:first-letter`（单冒号伪元素），无前导空格。

#### Scenario: Single colon pseudo-element
- **WHEN** SCSS 中或 extend 扩展后生成 `:before`
- **THEN** 规范化 CSS 输出 `:before`（非 `: before`）

#### Scenario: Extend-injected pseudo-element
- **WHEN** `%placeholder` 含 &:hover 声明且被 extend
- **THEN** 传播的选择器无多余空格问题

### Requirement: Double-colon pseudo-element format
对于 CSS3+ 伪元素（`::before`, `::after`, `::placeholder` 等），系统 SHALL 输出双冒号格式。

#### Scenario: CSS3 pseudo-element output
- **WHEN** 序列化 `PseudoElement::Before`
- **THEN** 规范化输出 `::before`

### Requirement: Pseudo-class chaining
当伪类链式出现如 `:not(.x):hover` 时，系统 SHALL 正确组合无多余空格。

#### Scenario: Chained pseudo-classes
- **WHEN** 规则使用 `:not(.el-is-block):hover`
- **THEN** 输出 `:not(.el-is-block):hover`，not 内部括号无多余空格
