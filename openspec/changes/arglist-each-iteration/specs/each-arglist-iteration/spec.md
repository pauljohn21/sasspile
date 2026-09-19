# spec: @each 迭代 ArgList

## 概述

`@each $item in $list` 中的 `$list` 为 `ArgList` 时，应与 `List` 行为一致，
逐个产出元素。

## 行为规则

### 规则 1: ArgList 与 List 等价迭代

- **Given**: `@each $item in $list` 中 `$list` 求值为 `Value::ArgList([a, b, c], ...)`
- **Then**: `$item` 依次取值 `a`, `b`, `c`

### 规则 2: multi-var ArgList of pairs

- **Given**: `@each $k, $v in $list` 中 `$list` 为 `ArgList` 且元素为 2-element List
- **Then**: 每轮 `$k` / `$v` 解构对应位置的元素

### 规则 3: 已有行为不变

- `Value::List`、`Value::Map` 的现有行为不受影响
- `Value::ArgList` 作为其他上下文（非 `@each`）使用时不受影响
