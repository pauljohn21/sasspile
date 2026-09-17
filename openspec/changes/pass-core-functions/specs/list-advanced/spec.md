# list-advanced Specification

## ADDED Requirements

### Requirement: list.set-nth

 SHALL 返回新 list，其中 `$n` (1-based) 元素被 `$value` 替换。

#### Scenario: set-nth 1
- **WHEN** 输入为 `a {b: list.set-nth(a b c, 1, x)}`
- **THEN** 输出形如 `x, b, c`

### Requirement: list.zip

 SHALL 将多个 list 合并为一个形如 `(a 1, b 2, c 3)` 的 list。

#### Scenario: zip two lists
- **WHEN** 输入为 `a {b: list.zip(a b c, 1 2 3)}`
- **THEN** 输出形如 `a 1, b 2, c 3`

### Requirement: list.is-bracketed

#### Scenario: bracketed list
- **WHEN** 输入为 `a {b: list.is-bracketed([a b])}`
- **THEN** 输出 `true`

#### Scenario: unbracketed list
- **WHEN** 输入为 `a {b: list.is-bracketed(a b)}`
- **THEN** 输出 `false`
