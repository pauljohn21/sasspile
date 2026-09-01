## ADDED Requirements

### Requirement: 数值精度和格式化对齐

数值输出 SHALL 匹配 sass-spec 的精度和格式化规则，包括 infinity、特殊单位、科学计数法。

#### Scenario: infinity 多单位

- **WHEN** 输入涉及 `infinity` 带多单位运算
- **THEN** 输出格式匹配 sass-spec 期望

#### Scenario: 短 hex 颜色

- **WHEN** 输入 `color: #f00;`
- **THEN** 输出 `color: #f00;`（不展开为 `#ff0000`）

#### Scenario: 长 hex 颜色

- **WHEN** 输入 `color: #ff0000;`
- **THEN** 输出 `color: #ff0000;`（不缩短为 `#f00`）

### Requirement: 选择器排序对齐

逗号分隔的选择器 SHALL 按 sass-spec 的排序规则输出，不 SHALL 重新排序。

#### Scenario: 简单选择器顺序

- **WHEN** 输入 `c, b, a { color: red; }`
- **THEN** 输出保持 `c, b, a` 顺序

### Requirement: @media / @supports 合并规则

相同 query 的 @media/@supports 块 SHALL 合并为一个块，不同 query 的块 SHALL 保持独立。

#### Scenario: 相同 @media 合并

- **WHEN** 输入两个 `@media (min-width: 100px)` 块
- **THEN** 输出合并为一个 `@media (min-width: 100px)` 块

#### Scenario: 不同 @media 不合并

- **WHEN** 输入 `@media (min-width: 100px)` 和 `@media (min-width: 200px)`
- **THEN** 输出为两个独立的 @media 块

### Requirement: map.has-key 和 map.deep-remove 输出正确

`map.has-key` SHALL 返回正确的布尔值，`map.deep-remove` SHALL 正确移除嵌套键。

#### Scenario: has-key 存在的键

- **WHEN** 调用 `map-has-key(("a": 1), "a")`
- **THEN** 返回 `true`

#### Scenario: deep-remove 不存在的键

- **WHEN** 调用 `map-deep-remove(("a": ("b": 1)), "c")`
- **THEN** 返回原始 map 不变
