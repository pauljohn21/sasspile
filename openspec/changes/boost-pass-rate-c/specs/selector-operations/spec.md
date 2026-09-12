## ADDED Requirements

### Requirement: selector.unify 遵循 Sass 规范
`selector.unify()` 函数 SHALL 按照 Sass 规范返回两个选择器的统一形式。

#### Scenario: compound 选择器统一
- **WHEN** 调用 `selector-unify('.a', '.b')`
- **THEN** 返回 SHALL 为规范化的统一选择器（如 `.a.b`）

#### Scenario: 复杂选择器统一
- **WHEN** 调用 `selector-unify('a.b', 'c.d')`
- **THEN** 返回 SHALL 正确合并 compound 部分

### Requirement: selector.extend 正确扩展
`selector.extend()` 函数 SHALL 按照 Sass 规范执行选择器扩展。

#### Scenario: simple extend
- **WHEN** 调用 `selector-extend('.a', '.b', '.c')`
- **THEN** 返回 SHALL 正确替换扩展关系

### Requirement: selector.is_superselector 正确检测
`selector.is-superselector()` 函数 SHALL 按照 Sass 规范检测超选择器关系。

#### Scenario: 直接超选择器
- **WHEN** 调用 `selector-is-superselector('a', 'a.b')`
- **THEN** 返回 SHALL 为 `true`

#### Scenario: 非超选择器
- **WHEN** 调用 `selector-is-superselector('a.b', 'a')`
- **THEN** 返回 SHALL 为 `false`
