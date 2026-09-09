## MODIFIED Requirements

### Requirement: is_super_compound 伪元素归一化
`is_super_compound` SHALL 在比较伪元素时归一化 `is_class_syntax`——单冒号和双冒号语法视为相同伪元素。

#### Scenario: 单双冒号伪元素视为相同
- **WHEN** 判断 `:before` 是否是 `::before` 的超选择器
- **THEN** 返回 `true`

### Requirement: is_super_compound 伪元素必须精确匹配
`is_super_compound` SHALL 要求 super 中的伪元素必须在 sub 中存在（归一化后比较）。sub 中有但 super 中没有的伪元素不影响判断。

#### Scenario: sub 有伪元素 super 无
- **WHEN** 判断 `"c"` 是否是 `"c::d"` 的超选择器
- **THEN** 返回 `false`（sub 有伪元素但 super 无，super 更宽泛但伪元素目标不同）

#### Scenario: super 有伪元素 sub 也有（归一化后相同）
- **WHEN** 判断 `"::d"` 是否是 `"c::d"` 的超选择器
- **THEN** 返回 `true`

### Requirement: is_super_compound 伪类子集判断
`is_super_compound` SHALL 要求 super 中的每个伪类都在 sub 中存在（name 相同，arg 可不同——arg 更宽泛的 super）。

#### Scenario: super 伪类是 sub 伪类的超集
- **WHEN** 判断 `":nth-child(2n+1)"` 是否是 `":nth-child(2n+1):hover"` 的超选择器
- **THEN** 返回 `false`（sub 有 :hover 但 super 无）
