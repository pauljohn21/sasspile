# selector-advanced Specification

## ADDED Requirements

### Requirement: selector.parse

 SHALL 返回 selector 的 parsed 表示（list-of-strings 形式）。

#### Scenario: parse simple class
- **WHEN** 输入为 `a {b: selector.parse(".foo")}`
- **THEN** 输出形如 `.foo`（内部 list 序列化的字符串形式）

### Requirement: selector.extend

 SHALL 按 CSS extend 规则扩展 selector。

#### Scenario: extend
- **WHEN** 输入为 `selector.extend(".a .b", ".b", ".c")`
- **THEN** 返回 `.a .c`

### Requirement: selector.replace

 SHALL 替换 selector 中的某一部分（支持 `$source` 与 `$target`）。

#### Scenario: replace
- **WHEN** 输入为 `selector.replace(".a.b", ".b", ".c")`
- **THEN** 返回 `.a.c`

### Requirement: selector.unify

 SHALL 合并两个 selector（若可合并）。

#### Scenario: unify compatible
- **WHEN** 输入为 `selector.unify(".a", ".b")`
- **THEN** 返回 `.a.b`

#### Scenario: unify incompatible returns null
- **WHEN** 输入为 `selector.unify(".a", ".b .c")`
- **THEN** 返回 null

### Requirement: selector.is-superselector

 SHALL 判断第一个 selector 是否覆盖第二个。

#### Scenario: is-superselector true
- **WHEN** 输入为 `selector.is-superselector(".a", ".a.b")`
- **THEN** 返回 `true`

### Requirement: selector.simple-selectors

 SHALL 将复合 selector 拆分为简单 selector list。

#### Scenario: simple-selectors
- **WHEN** 输入为 `selector.simple-selectors(".a.b")`
- **THEN** 返回形如 `.a, .b`
