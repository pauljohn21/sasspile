# map-advanced Specification

## ADDED Requirements

### Requirement: map.remove

 SHALL 删除 map 中指定 key，返回新 map。若 key 不存在则无变化。

#### Scenario: remove existing key
- **WHEN** 输入为 `map-remove((a: 1, b: 2), a)`
- **THEN** 返回 `(b: 2)`

#### Scenario: remove non-existing key no-op
- **WHEN** 输入为 `map-remove((a: 1, b: 2), c)`
- **THEN** 返回原 map

### Requirement: map.deep-merge

 SHALL 递归合并两个 map，子 map 也合并。

#### Scenario: deep-merge overlapping keys
- **WHEN** 输入为 `map-deep-merge((a: (b: 1, c: 2)), (a: (b: 3, d: 4)))`
- **THEN** 返回 `(a: (b: 3, c: 2, d: 4))`

### Requirement: map.deep-remove

 SHALL 从嵌套 map 中删除 key。

#### Scenario: deep remove nested
- **WHEN** 输入为 `map-deep-remove((a: (b: 1, c: 2)), a, b)`
- **THEN** 返回 `(a: (c: 2))`
